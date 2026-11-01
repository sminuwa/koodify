//! Adaptive Bank/DVA Routing — temporary/dynamic VA bank selection with sticky
//! preference + measured failover.
//!
//! Safety rules:
//! - Explicit `bankType` from the client is never overridden.
//! - Default candidate pool is `gtBank` only (current production behavior).
//! - Extra banks are opt-in via `VA_BANK_CANDIDATES` once verified with Zainpay.
//! - Preference only flips after enough samples and a clear score margin.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tracing::{debug, info, warn};

const DEFAULT_STICKY: Duration = Duration::from_secs(10 * 60);
const MIN_SAMPLES_TO_SWITCH: u64 = 3;
const SWITCH_MARGIN: f64 = 0.12; // challenger must beat preferred by ~12%

#[derive(Debug, Clone)]
struct BankStats {
    successes: u64,
    failures: u64,
    /// Exponential moving average of attempt latency (ms).
    latency_ema_ms: f64,
}

impl Default for BankStats {
    fn default() -> Self {
        Self {
            successes: 0,
            failures: 0,
            latency_ema_ms: 800.0,
        }
    }
}

impl BankStats {
    fn samples(&self) -> u64 {
        self.successes.saturating_add(self.failures)
    }

    fn success_rate(&self) -> f64 {
        let n = self.samples();
        if n == 0 {
            return 0.85; // optimistic prior so unused banks get a try
        }
        self.successes as f64 / n as f64
    }

    /// Higher is better. Blends reliability with latency.
    fn score(&self) -> f64 {
        let rate = self.success_rate();
        let latency_penalty = (self.latency_ema_ms / 1000.0).clamp(0.05, 8.0);
        rate / latency_penalty
    }

    fn observe(&mut self, ok: bool, latency: Duration) {
        let ms = latency.as_secs_f64() * 1000.0;
        if self.samples() == 0 {
            self.latency_ema_ms = ms;
        } else {
            // Mild EMA — recent attempts matter, but don't thrash.
            self.latency_ema_ms = self.latency_ema_ms * 0.7 + ms * 0.3;
        }
        if ok {
            self.successes = self.successes.saturating_add(1);
        } else {
            self.failures = self.failures.saturating_add(1);
        }
    }
}

#[derive(Debug)]
struct Inner {
    preferred: String,
    preferred_until: Instant,
    stats: HashMap<String, BankStats>,
}

/// Shared Adaptive Bank/DVA router for temporary/dynamic VA issuance.
#[derive(Debug)]
pub struct BankRouter {
    candidates: Vec<String>,
    sticky: Duration,
    inner: Mutex<Inner>,
}

impl BankRouter {
    pub fn new(candidates: Vec<String>, sticky: Duration) -> Self {
        let mut cleaned = Vec::new();
        for c in candidates {
            let t = c.trim().trim_matches('"').to_string();
            if t.is_empty() {
                continue;
            }
            if !cleaned.iter().any(|x| x == &t) {
                cleaned.push(t);
            }
        }
        if cleaned.is_empty() {
            cleaned.push("gtBank".into());
        }
        let preferred = cleaned[0].clone();
        Self {
            candidates: cleaned,
            sticky: if sticky.is_zero() {
                DEFAULT_STICKY
            } else {
                sticky
            },
            inner: Mutex::new(Inner {
                preferred: preferred.clone(),
                preferred_until: Instant::now() + DEFAULT_STICKY,
                stats: HashMap::new(),
            }),
        }
    }

    pub fn from_config(candidates: &[String], sticky_secs: u64) -> Self {
        Self::new(
            candidates.to_vec(),
            Duration::from_secs(sticky_secs.max(60)),
        )
    }

    pub fn candidates(&self) -> &[String] {
        &self.candidates
    }

    pub fn preferred(&self) -> String {
        self.inner
            .lock()
            .map(|g| g.preferred.clone())
            .unwrap_or_else(|_| self.candidates[0].clone())
    }

    /// Order to try for an auto-routed VA create: sticky preferred first, then
    /// remaining candidates by live score (best first).
    pub fn attempt_order(&self) -> Vec<String> {
        let Ok(mut g) = self.inner.lock() else {
            return self.candidates.clone();
        };

        if Instant::now() >= g.preferred_until {
            self.recompute_preferred_locked(&mut g);
            g.preferred_until = Instant::now() + self.sticky;
        }

        let mut rest: Vec<String> = self
            .candidates
            .iter()
            .filter(|b| *b != &g.preferred)
            .cloned()
            .collect();
        rest.sort_by(|a, b| {
            let sa = g.stats.get(a).cloned().unwrap_or_default().score();
            let sb = g.stats.get(b).cloned().unwrap_or_default().score();
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut order = Vec::with_capacity(self.candidates.len());
        order.push(g.preferred.clone());
        order.extend(rest);
        order
    }

    pub fn record_outcome(&self, bank: &str, ok: bool, latency: Duration) {
        let Ok(mut g) = self.inner.lock() else {
            return;
        };
        g.stats
            .entry(bank.to_string())
            .or_default()
            .observe(ok, latency);

        let stats = g.stats.get(bank).cloned().unwrap_or_default();
        debug!(
            bank,
            ok,
            latency_ms = latency.as_millis(),
            success_rate = format!("{:.2}", stats.success_rate()),
            score = format!("{:.3}", stats.score()),
            "va bank attempt recorded"
        );

        // On hard failure of the preferred bank, allow an immediate recompute
        // so the next checkout can stick to a healthier candidate sooner.
        if !ok && bank == g.preferred {
            self.recompute_preferred_locked(&mut g);
            g.preferred_until = Instant::now() + self.sticky;
        }
    }

    fn recompute_preferred_locked(&self, g: &mut Inner) {
        let current = g.preferred.clone();
        let current_stats = g.stats.get(&current).cloned().unwrap_or_default();
        let current_score = current_stats.score();

        let mut best = current.clone();
        let mut best_score = current_score;
        let mut best_samples = current_stats.samples();

        for bank in &self.candidates {
            let s = g.stats.get(bank).cloned().unwrap_or_default();
            let score = s.score();
            if score > best_score {
                best = bank.clone();
                best_score = score;
                best_samples = s.samples();
            }
        }

        if best == current {
            return;
        }

        // Hysteresis: need enough evidence and a clear margin.
        let margin_ok = best_score >= current_score * (1.0 + SWITCH_MARGIN)
            || (current_stats.samples() > 0 && current_stats.success_rate() < 0.5);
        if best_samples < MIN_SAMPLES_TO_SWITCH && current_stats.samples() >= MIN_SAMPLES_TO_SWITCH {
            return;
        }
        if !margin_ok && current_stats.samples() >= MIN_SAMPLES_TO_SWITCH {
            return;
        }

        info!(
            from = %current,
            to = %best,
            from_score = format!("{:.3}", current_score),
            to_score = format!("{:.3}", best_score),
            "va bank preference switched"
        );
        g.preferred = best;
    }
}

/// Parse `VA_BANK_CANDIDATES` style CSV. Empty / missing → `["gtBank"]`.
pub fn parse_candidates(raw: Option<&str>) -> Vec<String> {
    let Some(raw) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return vec!["gtBank".into()];
    };
    let list: Vec<String> = raw
        .split([',', ';', ' '])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if list.is_empty() {
        vec!["gtBank".into()]
    } else {
        list
    }
}

/// Warn when unknown/experimental keys are configured.
///
/// TODO(zainpay-va-banks): Replace this hardcoded `KNOWN` allowlist once Zainpay
/// ships an “available temporary/dynamic VA banks” endpoint. The `client` crate
/// should fetch/cache that list; Adaptive Bank/DVA Routing would validate (and
/// preferably *populate*) candidates from it. At that point `VA_BANK_CANDIDATES`
/// may become an optional override / preference order, not the source of truth.
pub fn warn_if_experimental(candidates: &[String]) {
    const KNOWN: &[&str] = &["gtBank", "wemaBank", "polaris", "fcmb", "zainMFB", "moniepoint"];
    for c in candidates {
        if !KNOWN.iter().any(|k| k.eq_ignore_ascii_case(c)) {
            warn!(
                bank = %c,
                "VA_BANK_CANDIDATES includes an unrecognized bank key — confirm it works with Zainpay temporary VA"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_candidates_are_gtbank_only() {
        let list = parse_candidates(None);
        assert_eq!(list, vec!["gtBank".to_string()]);
    }

    #[test]
    fn attempt_order_puts_preferred_first() {
        let router = BankRouter::new(
            vec!["gtBank".into(), "moniepoint".into()],
            Duration::from_secs(600),
        );
        let order = router.attempt_order();
        assert_eq!(order[0], "gtBank");
        assert!(order.contains(&"moniepoint".to_string()));
    }

    #[test]
    fn failures_can_promote_healthier_bank() {
        let router = BankRouter::new(
            vec!["gtBank".into(), "moniepoint".into()],
            Duration::from_secs(1),
        );

        for _ in 0..4 {
            router.record_outcome("gtBank", false, Duration::from_millis(1200));
        }
        for _ in 0..4 {
            router.record_outcome("moniepoint", true, Duration::from_millis(200));
        }

        // Preferred failure path already recomputed; order should favor moniepoint.
        let order = router.attempt_order();
        assert_eq!(order[0], "moniepoint");
    }

    #[test]
    fn parse_csv_candidates() {
        let list = parse_candidates(Some("gtBank, moniepoint"));
        assert_eq!(list, vec!["gtBank".to_string(), "moniepoint".to_string()]);
    }
}
