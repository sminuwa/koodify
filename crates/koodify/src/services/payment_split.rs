//! Zainbox Payment Split — validate plans, persist runs, redistribute after deposit.
//!
//! Collect full amount on primary zainbox, then internal VA/ISA → secondary ISA transfers.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use tracing::{info, warn};
use client::FundsTransferRequest;

use crate::db;
use crate::db::payment_splits::NewSplitLeg;
use crate::state::AppState;

pub const MAX_SHARES: usize = 10;
const MAX_TRANSFER_ATTEMPTS: u32 = 2;
const ZAIN_BANK_CODE: &str = "090976";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentSplitInput {
    #[serde(default)]
    pub primary_zainbox: Option<String>,
    pub shares: Vec<ShareInput>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareInput {
    pub zainbox_code: String,
    #[serde(default)]
    pub percent: Option<f64>,
    #[serde(default)]
    pub amount: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedLeg {
    pub zainbox_code: String,
    pub role: &'static str,
    pub amount_kobo: i64,
    pub share_percent: Option<f64>,
    pub share_amount: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedPlan {
    pub primary_zainbox: String,
    pub checkout_amount: String,
    pub total_kobo: i64,
    pub legs: Vec<ResolvedLeg>,
    pub snapshot: Value,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SplitValidateError {
    #[error("{0}")]
    Message(String),
}

impl SplitValidateError {
    fn msg(s: impl Into<String>) -> Self {
        Self::Message(s.into())
    }
}

/// Peel `paymentSplit` / `splitPlanId` off the upstream body into metadata.
pub fn peel_split_fields(body: &mut Value, metadata: &mut Value) {
    let Some(obj) = body.as_object_mut() else {
        return;
    };

    let split = obj.remove("paymentSplit").or_else(|| obj.remove("payment_split"));
    let plan_id = obj
        .remove("splitPlanId")
        .or_else(|| obj.remove("split_plan_id"));

    let meta = ensure_object(metadata);

    if let Some(v) = split {
        if !v.is_null() {
            meta.insert("paymentSplit".into(), v);
        }
    }
    if let Some(v) = plan_id {
        if let Some(s) = v.as_str().map(str::trim).filter(|s| !s.is_empty()) {
            meta.insert("splitPlanId".into(), Value::String(s.to_string()));
        } else if !v.is_null() {
            meta.insert("splitPlanId".into(), v);
        }
    }
}

fn ensure_object(metadata: &mut Value) -> &mut Map<String, Value> {
    if !metadata.is_object() {
        *metadata = Value::Object(Map::new());
    }
    metadata.as_object_mut().expect("object")
}

/// Convert Naira amount string (e.g. `"10000"` / `"10000.50"`) to integer kobo.
pub fn naira_to_kobo(amount: &str) -> Result<i64, SplitValidateError> {
    let t = amount.trim().replace(',', "");
    if t.is_empty() {
        return Err(SplitValidateError::msg("amount is empty"));
    }
    // Reject scientific notation / junk.
    if t.chars().any(|c| !(c.is_ascii_digit() || c == '.')) {
        return Err(SplitValidateError::msg(format!(
            "invalid amount `{amount}`"
        )));
    }
    let parts: Vec<&str> = t.split('.').collect();
    if parts.len() > 2 {
        return Err(SplitValidateError::msg(format!(
            "invalid amount `{amount}`"
        )));
    }
    let whole: i64 = parts[0]
        .parse()
        .map_err(|_| SplitValidateError::msg(format!("invalid amount `{amount}`")))?;
    if whole < 0 {
        return Err(SplitValidateError::msg("amount must be positive"));
    }
    let frac_kobo: i64 = if parts.len() == 2 {
        let mut frac = parts[1].to_string();
        if frac.len() > 2 {
            return Err(SplitValidateError::msg(format!(
                "amount `{amount}` has more than 2 decimal places"
            )));
        }
        while frac.len() < 2 {
            frac.push('0');
        }
        frac.parse()
            .map_err(|_| SplitValidateError::msg(format!("invalid amount `{amount}`")))?
    } else {
        0
    };
    whole
        .checked_mul(100)
        .and_then(|v| v.checked_add(frac_kobo))
        .ok_or_else(|| SplitValidateError::msg("amount overflow"))
}

/// Resolve + validate a plan against the checkout amount and primary zainbox.
pub fn resolve_plan(
    primary_zainbox: &str,
    checkout_amount: &str,
    input: &PaymentSplitInput,
) -> Result<ResolvedPlan, SplitValidateError> {
    if let Some(ref p) = input.primary_zainbox {
        let p = p.trim();
        if !p.is_empty() && p != primary_zainbox {
            return Err(SplitValidateError::msg(format!(
                "paymentSplit.primaryZainbox `{p}` must match checkout zainbox `{primary_zainbox}`"
            )));
        }
    }

    if input.shares.is_empty() {
        return Err(SplitValidateError::msg(
            "paymentSplit.shares must contain at least one share",
        ));
    }
    if input.shares.len() > MAX_SHARES {
        return Err(SplitValidateError::msg(format!(
            "paymentSplit.shares cannot exceed {MAX_SHARES} legs"
        )));
    }

    let total_kobo = naira_to_kobo(checkout_amount)?;
    if total_kobo <= 0 {
        return Err(SplitValidateError::msg("checkout amount must be > 0"));
    }

    // Pass 1: validate shape + allocate fixed amounts.
    // Pass 2: percent legs take a share of whatever **remains** after fixed amounts
    // (e.g. ₦200 + ₦50 + percent:100 → remainder ₦750). Percents must sum to 100
    // of that remainder pool.
    let mut seen = std::collections::HashSet::new();
    let mut fixed_legs: Vec<ResolvedLeg> = Vec::new();
    let mut percent_shares: Vec<(String, f64)> = Vec::new();
    let mut fixed_sum: i64 = 0;

    for share in &input.shares {
        let code = share.zainbox_code.trim();
        if code.is_empty() {
            return Err(SplitValidateError::msg(
                "each share requires a non-empty zainboxCode",
            ));
        }
        if !seen.insert(code.to_string()) {
            return Err(SplitValidateError::msg(format!(
                "duplicate zainboxCode `{code}` in paymentSplit.shares"
            )));
        }

        let has_pct = share.percent.is_some();
        let has_amt = share
            .amount
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        if has_pct == has_amt {
            return Err(SplitValidateError::msg(format!(
                "share for `{code}` must set either percent or amount, not both/neither"
            )));
        }

        let role = if code == primary_zainbox {
            "keep"
        } else {
            "transfer"
        };

        if has_amt {
            let amt = share.amount.as_ref().unwrap().trim();
            let kobo = naira_to_kobo(amt)?;
            if kobo <= 0 {
                return Err(SplitValidateError::msg(format!(
                    "amount for `{code}` must be > 0"
                )));
            }
            fixed_sum = fixed_sum
                .checked_add(kobo)
                .ok_or_else(|| SplitValidateError::msg("share amounts overflow"))?;
            fixed_legs.push(ResolvedLeg {
                zainbox_code: code.to_string(),
                role,
                amount_kobo: kobo,
                share_percent: None,
                share_amount: Some(amt.to_string()),
            });
        } else {
            let pct = share.percent.unwrap();
            if !pct.is_finite() || pct <= 0.0 || pct > 100.0 {
                return Err(SplitValidateError::msg(format!(
                    "percent for `{code}` must be in (0, 100]"
                )));
            }
            percent_shares.push((code.to_string(), pct));
        }
    }

    if fixed_sum > total_kobo {
        return Err(SplitValidateError::msg(format!(
            "fixed share total {fixed_sum} kobo exceeds checkout {total_kobo} kobo"
        )));
    }

    let remainder = total_kobo - fixed_sum;
    if percent_shares.is_empty() {
        if remainder != 0 {
            return Err(SplitValidateError::msg(format!(
                "fixed shares leave {remainder} kobo unallocated — add a percent:100 remainder leg (usually primary)"
            )));
        }
    } else {
        let pct_sum: f64 = percent_shares.iter().map(|(_, p)| *p).sum();
        if (pct_sum - 100.0).abs() > 0.001 {
            return Err(SplitValidateError::msg(format!(
                "percent shares must total 100% of the remaining amount after fixed shares (got {pct_sum}%)"
            )));
        }
        if remainder <= 0 {
            return Err(SplitValidateError::msg(
                "nothing left for percent shares after fixed amounts — reduce fixed shares or drop percent legs",
            ));
        }
    }

    let mut percent_legs: Vec<ResolvedLeg> = Vec::with_capacity(percent_shares.len());
    let mut percent_allocated: i64 = 0;
    for (code, pct) in &percent_shares {
        let raw = (*pct / 100.0) * (remainder as f64);
        let kobo = raw.floor() as i64;
        if kobo <= 0 && *pct < 100.0 {
            return Err(SplitValidateError::msg(format!(
                "percent for `{code}` rounds to zero kobo of the remaining {remainder}"
            )));
        }
        percent_allocated = percent_allocated
            .checked_add(kobo)
            .ok_or_else(|| SplitValidateError::msg("share amounts overflow"))?;
        let role = if code == primary_zainbox {
            "keep"
        } else {
            "transfer"
        };
        percent_legs.push(ResolvedLeg {
            zainbox_code: code.clone(),
            role,
            amount_kobo: kobo,
            share_percent: Some(*pct),
            share_amount: None,
        });
    }

    // Flooring leftover within the remainder pool → primary percent leg if present,
    // else the first percent leg.
    let pct_leftover = remainder - percent_allocated;
    if pct_leftover > 0 && !percent_legs.is_empty() {
        let idx = percent_legs
            .iter()
            .position(|l| l.zainbox_code == primary_zainbox)
            .unwrap_or(0);
        percent_legs[idx].amount_kobo += pct_leftover;
    }

    // Preserve input order: interleave by walking shares again.
    let mut computed: Vec<ResolvedLeg> = Vec::with_capacity(input.shares.len());
    let mut fi = 0usize;
    let mut pi = 0usize;
    for share in &input.shares {
        let has_amt = share
            .amount
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        if has_amt {
            computed.push(fixed_legs[fi].clone());
            fi += 1;
        } else {
            computed.push(percent_legs[pi].clone());
            pi += 1;
        }
    }

    if !computed.iter().any(|l| l.zainbox_code == primary_zainbox) {
        return Err(SplitValidateError::msg(
            "paymentSplit.shares must include the primary (collector) zainbox",
        ));
    }

    if !computed.iter().any(|l| l.role == "transfer") {
        return Err(SplitValidateError::msg(
            "paymentSplit must include at least one secondary zainbox transfer leg",
        ));
    }

    let sum: i64 = computed.iter().map(|l| l.amount_kobo).sum();
    if sum != total_kobo {
        return Err(SplitValidateError::msg(format!(
            "internal split math error: legs sum {sum} != total {total_kobo}"
        )));
    }

    let snapshot = json!({
        "primaryZainbox": primary_zainbox,
        "checkoutAmount": checkout_amount,
        "totalKobo": total_kobo,
        "shares": input.shares,
        "legs": computed.iter().map(|l| json!({
            "zainboxCode": l.zainbox_code,
            "role": l.role,
            "amountKobo": l.amount_kobo,
            "percent": l.share_percent,
            "amount": l.share_amount,
        })).collect::<Vec<_>>(),
    });

    Ok(ResolvedPlan {
        primary_zainbox: primary_zainbox.to_string(),
        checkout_amount: checkout_amount.to_string(),
        total_kobo,
        legs: computed,
        snapshot,
    })
}

/// Validate referenced zainboxes exist and are active in the local cache.
pub async fn assert_boxes_active(
    pool: &sqlx::PgPool,
    plan: &ResolvedPlan,
) -> Result<(), SplitValidateError> {
    for leg in &plan.legs {
        match db::zainboxes::get_by_code(pool, &leg.zainbox_code).await {
            Ok(Some(z)) if z.is_active => {}
            Ok(Some(_)) => {
                return Err(SplitValidateError::msg(format!(
                    "zainbox `{}` is inactive",
                    leg.zainbox_code
                )));
            }
            Ok(None) => {
                return Err(SplitValidateError::msg(format!(
                    "unknown zainboxCode `{}` — seed/sync zainboxes first",
                    leg.zainbox_code
                )));
            }
            Err(err) => {
                warn!(error = %err, "zainbox lookup failed during split validate");
                return Err(SplitValidateError::msg(
                    "failed to verify zainbox codes for paymentSplit",
                ));
            }
        }
    }
    Ok(())
}

/// Build a resolved plan from init body metadata + checkout fields.
pub async fn plan_from_checkout(
    state: &AppState,
    primary_zainbox: &str,
    checkout_amount: &str,
    metadata: &Value,
) -> Result<Option<ResolvedPlan>, SplitValidateError> {
    let plan_id = metadata
        .get("splitPlanId")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let inline = metadata.get("paymentSplit").cloned();

    if plan_id.is_none() && inline.as_ref().is_none_or(|v| v.is_null()) {
        return Ok(None);
    }

    let input = if let Some(id) = plan_id {
        let stored = db::split_plans::get(&state.pool, id)
            .await
            .map_err(|e| {
                warn!(error = %e, split_plan_id = id, "split plan lookup failed");
                SplitValidateError::msg("failed to load splitPlanId")
            })?;
        let Some(stored) = stored else {
            return Err(SplitValidateError::msg(format!(
                "unknown splitPlanId `{id}`"
            )));
        };
        if !stored.is_active {
            return Err(SplitValidateError::msg(format!(
                "splitPlanId `{id}` is inactive"
            )));
        }
        if stored.primary_zainbox != primary_zainbox {
            return Err(SplitValidateError::msg(format!(
                "splitPlanId `{id}` belongs to `{}`, not checkout zainbox `{primary_zainbox}`",
                stored.primary_zainbox
            )));
        }
        let shares: Vec<ShareInput> = serde_json::from_value(stored.shares.clone())
            .map_err(|e| SplitValidateError::msg(format!("split plan shares invalid: {e}")))?;
        PaymentSplitInput {
            primary_zainbox: Some(stored.primary_zainbox),
            shares,
        }
    } else {
        serde_json::from_value(inline.unwrap()).map_err(|e| {
            SplitValidateError::msg(format!("invalid paymentSplit object: {e}"))
        })?
    };

    let plan = resolve_plan(primary_zainbox, checkout_amount, &input)?;
    assert_boxes_active(&state.pool, &plan).await?;
    Ok(Some(plan))
}

pub async fn persist_pending(
    pool: &sqlx::PgPool,
    txn_ref: &str,
    plan: &ResolvedPlan,
) -> Result<(), sqlx::Error> {
    let legs: Vec<NewSplitLeg<'_>> = plan
        .legs
        .iter()
        .enumerate()
        .map(|(i, l)| NewSplitLeg {
            leg_index: i as i32,
            zainbox_code: &l.zainbox_code,
            role: l.role,
            amount_kobo: l.amount_kobo,
            share_percent: l.share_percent,
            share_amount: l.share_amount.as_deref(),
        })
        .collect();

    db::payment_splits::insert_pending(
        pool,
        txn_ref,
        &plan.primary_zainbox,
        &plan.checkout_amount,
        plan.total_kobo,
        &plan.snapshot,
        &legs,
    )
    .await?;
    Ok(())
}

/// After payment success: claim + execute secondary transfers.
pub async fn maybe_run_on_success(
    state: &AppState,
    txn_ref: &str,
    response: &Value,
    deposit_account: Option<&str>,
    deposit_bank_code: Option<&str>,
) {
    if !is_payment_success(response) {
        return;
    }

    if let (Some(acct), Some(code)) = (deposit_account, deposit_bank_code) {
        let _ = db::payment_splits::set_source(&state.pool, txn_ref, acct, code).await;
    } else if let Some(acct) = deposit_account {
        // Infer bank from Zainpay list later; store account early.
        let _ = db::payment_splits::set_source(&state.pool, txn_ref, acct, "").await;
    }

    let claimed = match db::payment_splits::claim_pending(&state.pool, txn_ref).await {
        Ok(c) => c,
        Err(err) => {
            warn!(error = %err, txn_ref, "failed to claim payment split");
            return;
        }
    };
    let Some(split) = claimed else {
        return;
    };

    info!(
        txn_ref,
        primary = %split.primary_zainbox,
        "payment split claimed — executing legs"
    );

    if let Err(err) = execute_split(state, &split).await {
        warn!(error = %err, txn_ref, "payment split execution failed");
        let _ = db::payment_splits::mark_finished(
            &state.pool,
            txn_ref,
            "failed",
            Some(&err),
        )
        .await;
    }
}

fn is_payment_success(response: &Value) -> bool {
    let candidates = [
        response.pointer("/data/paymentStatus"),
        response.get("paymentStatus"),
        response.pointer("/data/status"),
        response.get("status"),
        response.pointer("/data/txnStatus"),
        response.pointer("/data/transactionStatus"),
    ];
    for c in candidates.into_iter().flatten() {
        let status = match c {
            Value::String(s) => s.to_ascii_lowercase(),
            Value::Bool(true) => "success".into(),
            _ => continue,
        };
        if matches!(
            status.as_str(),
            "success" | "successful" | "paid" | "completed" | "00"
        ) {
            return true;
        }
    }
    // Some envelopes only set top-level code on verify-style payloads.
    response
        .get("code")
        .and_then(|v| v.as_str())
        .is_some_and(|c| c == "00")
        && response
            .pointer("/data/paymentStatus")
            .or_else(|| response.pointer("/data/depositedAmount"))
            .is_some()
}

/// Admin / recovery: run (or retry) a split without waiting for another status poll.
pub async fn force_run(
    state: &AppState,
    txn_ref: &str,
) -> Result<Value, String> {
    let claimed = db::payment_splits::claim_for_run(&state.pool, txn_ref, true)
        .await
        .map_err(|e| e.to_string())?;
    let Some(split) = claimed else {
        let existing = db::payment_splits::get(&state.pool, txn_ref)
            .await
            .map_err(|e| e.to_string())?;
        return match existing {
            None => Err(format!("no payment split for txnRef `{txn_ref}`")),
            Some(s) if s.status == "success" => {
                let summary = db::payment_splits::summary_json(&state.pool, txn_ref)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(json!({
                    "message": "split already completed",
                    "summary": summary,
                }))
            }
            Some(s) if s.status == "running" => Err(
                "split is already running — wait a moment and fetch status".into(),
            ),
            Some(s) => Err(format!(
                "cannot run split in status `{}`",
                s.status
            )),
        };
    };

    info!(txn_ref, "payment split force-run claimed");
    if let Err(err) = execute_split(state, &split).await {
        warn!(error = %err, txn_ref, "payment split force-run failed");
        let _ = db::payment_splits::mark_finished(
            &state.pool,
            txn_ref,
            "failed",
            Some(&err),
        )
        .await;
    }

    db::payment_splits::summary_json(&state.pool, txn_ref)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "split disappeared after run".into())
}

async fn execute_split(state: &AppState, split: &db::payment_splits::PaymentSplit) -> Result<(), String> {
    let legs = db::payment_splits::list_legs(&state.pool, &split.txn_ref)
        .await
        .map_err(|e| e.to_string())?;

    let (source_account, source_bank) =
        resolve_source(state, split).await.map_err(|e| e.clone())?;

    let _ = db::payment_splits::set_source(
        &state.pool,
        &split.txn_ref,
        &source_account,
        &source_bank,
    )
    .await;

    let mut any_fail = false;
    let mut any_ok_transfer = false;
    let mut last_err: Option<String> = None;

    for leg in &legs {
        if leg.status == "success" {
            if leg.role == "transfer" {
                any_ok_transfer = true;
            }
            continue;
        }

        if leg.role == "keep" {
            let _ = db::payment_splits::update_leg_result(
                &state.pool,
                leg.id,
                "success",
                None,
                None,
                None,
                None,
                None,
            )
            .await;
            continue;
        }

        if leg.amount_kobo <= 0 {
            let _ = db::payment_splits::update_leg_result(
                &state.pool,
                leg.id,
                "skipped",
                None,
                None,
                None,
                None,
                Some("zero amount"),
            )
            .await;
            continue;
        }

        match transfer_leg(state, split, leg, &source_account, &source_bank).await {
            Ok(()) => any_ok_transfer = true,
            Err(err) => {
                any_fail = true;
                last_err = Some(err);
            }
        }
    }

    let status = if any_fail && any_ok_transfer {
        "partial"
    } else if any_fail {
        "failed"
    } else {
        "success"
    };

    db::payment_splits::mark_finished(
        &state.pool,
        &split.txn_ref,
        status,
        last_err.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())?;

    if status == "failed" {
        return Err(last_err.unwrap_or_else(|| "all transfer legs failed".into()));
    }
    Ok(())
}

async fn transfer_leg(
    state: &AppState,
    split: &db::payment_splits::PaymentSplit,
    leg: &db::payment_splits::PaymentSplitLeg,
    source_account: &str,
    source_bank: &str,
) -> Result<(), String> {
    let dest = resolve_dest_isa(state, &leg.zainbox_code).await?;
    let mut last_err = String::from("transfer failed");

    for attempt in 1..=MAX_TRANSFER_ATTEMPTS {
        let transfer_ref = format!(
            "split_{}_{}_{}_{}",
            sanitize_ref(&split.txn_ref),
            leg.leg_index,
            attempt,
            chrono::Utc::now().timestamp_millis() % 1_000_000
        );

        let req = FundsTransferRequest {
            destination_account_number: dest.account.clone(),
            destination_bank_code: dest.bank_code.clone(),
            source_account_number: source_account.to_string(),
            source_bank_code: source_bank.to_string(),
            amount: leg.amount_kobo.to_string(),
            txn_ref: transfer_ref.clone(),
            zainbox_code: split.primary_zainbox.clone(),
            call_back_url: None,
            narration: Some(format!(
                "koodify split {} → {}",
                split.txn_ref, leg.zainbox_code
            )),
        };

        match state.zainpay.transfers().funds_transfer(&req).await {
            Ok(resp)
                if resp.is_success() || resp.code == client::types::codes::FUNDS_TRANSFER_QUEUED =>
            {
                // Queued (21) is acceptable for async transfers; verify best-effort.
                let payment_ref = resp
                    .data
                    .as_ref()
                    .and_then(|d| d.get("paymentRef"))
                    .and_then(|v| v.as_str())
                    .map(str::to_string);

                if resp.code == client::types::codes::FUNDS_TRANSFER_QUEUED {
                    let _ = state.zainpay.transfers().verify(&transfer_ref).await;
                }

                let _ = db::payment_splits::update_leg_result(
                    &state.pool,
                    leg.id,
                    "success",
                    Some(&dest.account),
                    Some(&dest.bank_code),
                    Some(&transfer_ref),
                    payment_ref.as_deref(),
                    None,
                )
                .await;
                info!(
                    txn_ref = %split.txn_ref,
                    leg = leg.leg_index,
                    dest = %leg.zainbox_code,
                    amount_kobo = leg.amount_kobo,
                    "split leg transferred"
                );
                return Ok(());
            }
            Ok(resp) => {
                last_err = format!(
                    "transfer rejected code={} desc={}",
                    resp.code, resp.description
                );
                warn!(
                    txn_ref = %split.txn_ref,
                    leg = leg.leg_index,
                    attempt,
                    error = %last_err,
                    "split leg transfer application failure"
                );
            }
            Err(err) => {
                last_err = err.to_string();
                warn!(
                    txn_ref = %split.txn_ref,
                    leg = leg.leg_index,
                    attempt,
                    error = %last_err,
                    "split leg transfer HTTP failure"
                );
            }
        }
    }

    let _ = db::payment_splits::update_leg_result(
        &state.pool,
        leg.id,
        "failed",
        Some(&dest.account),
        Some(&dest.bank_code),
        None,
        None,
        Some(&last_err),
    )
    .await;
    Err(last_err)
}

struct DestAcct {
    account: String,
    bank_code: String,
}

async fn resolve_dest_isa(state: &AppState, zainbox_code: &str) -> Result<DestAcct, String> {
    let resp = state
        .zainpay
        .settlements()
        .internal_account(zainbox_code)
        .await
        .map_err(|e| format!("ISA fetch failed for {zainbox_code}: {e}"))?;

    let data = resp
        .data
        .ok_or_else(|| format!("empty ISA response for {zainbox_code}"))?;

    let account = data
        .get("bankAccount")
        .or_else(|| data.get("accountNumber"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("ISA missing account for {zainbox_code}"))?
        .to_string();

    let bank_code = data
        .get("bankCode")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or_else(|| {
            data.get("bankName")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| s.chars().all(|c| c.is_ascii_digit()))
        })
        .unwrap_or(ZAIN_BANK_CODE)
        .to_string();

    Ok(DestAcct { account, bank_code })
}

async fn resolve_source(
    state: &AppState,
    split: &db::payment_splits::PaymentSplit,
) -> Result<(String, String), String> {
    if let (Some(acct), Some(code)) = (
        split.source_account.as_deref().map(str::trim).filter(|s| !s.is_empty()),
        split
            .source_bank_code
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty()),
    ) {
        return Ok((acct.to_string(), code.to_string()));
    }

    if let Some(acct) = split
        .source_account
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        // Have account without bank — look up from VA list.
        if let Ok(Some(code)) = lookup_bank_code(state, &split.primary_zainbox, acct).await {
            return Ok((acct.to_string(), code));
        }
    }

    // Prefer a funded deposit VA on the primary box.
    if let Ok(Some(pair)) =
        pick_funded_source(state, &split.primary_zainbox, split.total_kobo).await
    {
        return Ok(pair);
    }

    // Fallback: primary ISA.
    let isa = resolve_dest_isa(state, &split.primary_zainbox).await?;
    Ok((isa.account, isa.bank_code))
}

async fn lookup_bank_code(
    state: &AppState,
    zainbox_code: &str,
    account: &str,
) -> Result<Option<String>, String> {
    let resp = state
        .zainpay
        .zainbox()
        .virtual_accounts(zainbox_code)
        .await
        .map_err(|e| e.to_string())?;
    let Some(Value::Array(items)) = resp.data else {
        return Ok(None);
    };
    for item in items {
        let acct = item
            .get("bankAccount")
            .or_else(|| item.get("accountNumber"))
            .and_then(|v| v.as_str());
        if acct != Some(account) {
            continue;
        }
        let code = item
            .get("bankCode")
            .and_then(|v| v.as_str())
            .or_else(|| {
                item.get("bankName")
                    .and_then(|v| v.as_str())
                    .filter(|s| s.chars().all(|c| c.is_ascii_digit()))
            })
            .map(str::to_string);
        return Ok(code);
    }
    Ok(None)
}

async fn pick_funded_source(
    state: &AppState,
    zainbox_code: &str,
    need_kobo: i64,
) -> Result<Option<(String, String)>, String> {
    let resp = state
        .zainpay
        .zainbox()
        .accounts_balance(zainbox_code)
        .await
        .map_err(|e| e.to_string())?;

    let Some(Value::Array(items)) = resp.data else {
        return Ok(None);
    };

    let mut best: Option<(i64, String, String)> = None;
    for item in items {
        let acct = item
            .get("accountNumber")
            .or_else(|| item.get("bankAccount"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let Some(acct) = acct else { continue };

        let bank_type = item
            .get("bankType")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        // Prefer non-ISA deposit VAs when possible.
        let is_isa = bank_type.eq_ignore_ascii_case("ZainBank");

        let bal = item
            .get("balanceAmount")
            .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)))
            .unwrap_or(0);
        if bal < need_kobo {
            continue;
        }

        let code = item
            .get("bankCode")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| {
                if is_isa {
                    ZAIN_BANK_CODE.to_string()
                } else {
                    String::new()
                }
            });
        if code.is_empty() {
            continue;
        }

        let score = if is_isa { bal } else { bal + 1_000_000_000 };
        match &best {
            Some((s, _, _)) if *s >= score => {}
            _ => best = Some((score, acct.to_string(), code)),
        }
    }

    Ok(best.map(|(_, a, c)| (a, c)))
}

fn sanitize_ref(txn_ref: &str) -> String {
    txn_ref
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .take(40)
        .collect()
}

/// Attach `data.paymentSplit` summary when present.
pub async fn enrich_response(pool: &sqlx::PgPool, txn_ref: &str, mut response: Value) -> Value {
    match db::payment_splits::summary_json(pool, txn_ref).await {
        Ok(Some(summary)) => {
            if let Some(Value::Object(data)) = response.get_mut("data") {
                data.insert("paymentSplit".into(), summary);
            } else if response.get("data").is_none() {
                if let Some(obj) = response.as_object_mut() {
                    obj.insert(
                        "data".into(),
                        json!({ "paymentSplit": summary }),
                    );
                }
            }
        }
        Ok(None) => {}
        Err(err) => {
            warn!(error = %err, txn_ref, "failed to load payment split summary");
        }
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    fn share_pct(code: &str, pct: f64) -> ShareInput {
        ShareInput {
            zainbox_code: code.into(),
            percent: Some(pct),
            amount: None,
        }
    }

    fn share_amt(code: &str, amount: &str) -> ShareInput {
        ShareInput {
            zainbox_code: code.into(),
            percent: None,
            amount: Some(amount.into()),
        }
    }

    #[test]
    fn naira_to_kobo_basic() {
        assert_eq!(naira_to_kobo("10000").unwrap(), 1_000_000);
        assert_eq!(naira_to_kobo("10.5").unwrap(), 1050);
        assert_eq!(naira_to_kobo("10.50").unwrap(), 1050);
        assert!(naira_to_kobo("10.555").is_err());
        assert!(naira_to_kobo("-1").is_err());
    }

    #[test]
    fn resolve_percent_only_with_leftover_to_primary() {
        let input = PaymentSplitInput {
            primary_zainbox: Some("primary".into()),
            shares: vec![
                share_pct("primary", 70.0),
                share_pct("tutor", 20.0),
                share_pct("agent", 10.0),
            ],
        };
        // No fixed amounts → remainder = full checkout; flooring leftover → primary.
        let plan = resolve_plan("primary", "100.01", &input).unwrap();
        assert_eq!(plan.total_kobo, 10001);
        let sum: i64 = plan.legs.iter().map(|l| l.amount_kobo).sum();
        assert_eq!(sum, 10001);
        let primary = plan.legs.iter().find(|l| l.zainbox_code == "primary").unwrap();
        assert_eq!(primary.role, "keep");
        assert!(plan.legs.iter().any(|l| l.role == "transfer"));
    }

    #[test]
    fn resolve_fixed_then_remainder_percent() {
        // ₦1000 deposit: ₦200 + ₦50 + 100% of remainder (₦750) on primary.
        let input = PaymentSplitInput {
            primary_zainbox: None,
            shares: vec![
                share_amt("tutor_box", "200"),
                share_amt("agent_box", "50"),
                share_pct("school_main_box", 100.0),
            ],
        };
        let plan = resolve_plan("school_main_box", "1000", &input).unwrap();
        assert_eq!(plan.total_kobo, 100_000);
        let by = |c: &str| {
            plan.legs
                .iter()
                .find(|l| l.zainbox_code == c)
                .unwrap()
                .amount_kobo
        };
        assert_eq!(by("tutor_box"), 20_000);
        assert_eq!(by("agent_box"), 5_000);
        assert_eq!(by("school_main_box"), 75_000);
    }

    #[test]
    fn resolve_mixed_example() {
        // Fixed secondaries, primary keeps 100% of whatever remains.
        let input = PaymentSplitInput {
            primary_zainbox: None,
            shares: vec![
                share_amt("tutor_box", "2000"),
                share_amt("agent_box", "1000"),
                share_pct("school_main_box", 100.0),
            ],
        };
        let plan = resolve_plan("school_main_box", "10000", &input).unwrap();
        assert_eq!(plan.total_kobo, 1_000_000);
        let by = |c: &str| {
            plan.legs
                .iter()
                .find(|l| l.zainbox_code == c)
                .unwrap()
                .amount_kobo
        };
        assert_eq!(by("school_main_box"), 700_000);
        assert_eq!(by("tutor_box"), 200_000);
        assert_eq!(by("agent_box"), 100_000);
    }

    #[test]
    fn rejects_duplicate_and_primary_mismatch() {
        let input = PaymentSplitInput {
            primary_zainbox: Some("other".into()),
            shares: vec![share_pct("a", 50.0), share_pct("b", 50.0)],
        };
        assert!(resolve_plan("primary", "1000", &input).is_err());

        let input = PaymentSplitInput {
            primary_zainbox: None,
            shares: vec![share_pct("a", 50.0), share_pct("a", 50.0)],
        };
        assert!(resolve_plan("a", "1000", &input).is_err());
    }

    #[test]
    fn rejects_over_allocate_and_missing_secondary() {
        let input = PaymentSplitInput {
            primary_zainbox: None,
            shares: vec![
                share_amt("primary", "6000"),
                share_amt("tutor", "5000"),
            ],
        };
        assert!(resolve_plan("primary", "10000", &input).is_err());

        let input = PaymentSplitInput {
            primary_zainbox: None,
            shares: vec![share_pct("primary", 100.0)],
        };
        assert!(resolve_plan("primary", "10000", &input).is_err());
    }

    #[test]
    fn peel_split_fields_strips_body() {
        let mut body = json!({
            "amount": "1000",
            "paymentSplit": { "shares": [] },
            "splitPlanId": "plan_1",
        });
        let mut metadata = json!({});
        peel_split_fields(&mut body, &mut metadata);
        assert!(body.get("paymentSplit").is_none());
        assert!(body.get("splitPlanId").is_none());
        assert!(metadata.get("paymentSplit").is_some());
        assert_eq!(metadata.get("splitPlanId").and_then(|v| v.as_str()), Some("plan_1"));
    }
}
