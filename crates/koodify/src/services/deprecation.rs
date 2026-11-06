//! API versioning / deprecation headers (DX issue 08).
//!
//! Deprecated `/api/v1` aliases keep working until [`SUNSET_HTTP_DATE`], but
//! responses include `Deprecation`, `Sunset`, and `Link` so clients can migrate.

use axum::http::{HeaderValue, Request};
use axum::middleware::Next;
use axum::response::Response;

/// HTTP-date for RFC 8594 `Sunset` (1 May 2027 UTC).
pub const SUNSET_HTTP_DATE: &str = "Sat, 01 May 2027 00:00:00 GMT";

/// Calendar form for docs / FEATURES.
pub const SUNSET_DATE: &str = "2027-05-01";

#[derive(Debug, Clone, Copy)]
pub struct DeprecatedAlias {
    /// Human path pattern (for docs).
    pub path_pattern: &'static str,
    /// Canonical replacement path pattern.
    pub successor: &'static str,
}

/// Deprecated `/api/v1` aliases still routed today.
pub const DEPRECATED_ALIASES: &[DeprecatedAlias] = &[
    DeprecatedAlias {
        path_pattern: "/api/v1/zainbox/{zainboxCode}/activity",
        successor: "/api/v1/zainbox/{zainboxCode}/transactions",
    },
    DeprecatedAlias {
        path_pattern: "/api/v1/merchant/activity",
        successor: "/api/v1/merchant/transactions",
    },
    DeprecatedAlias {
        path_pattern: "/api/v1/virtual-account/{accountNumber}",
        successor: "/api/v1/va/{accountNumber}",
    },
    DeprecatedAlias {
        path_pattern: "/api/v1/deposit/{txnRef}",
        successor: "/api/v1/dva/verify/{txnRef}",
    },
];

/// Concrete successor with path params filled when possible.
pub fn concrete_successor(path: &str) -> Option<String> {
    let path = path.trim_end_matches('/');
    if let Some(rest) = path.strip_prefix("/api/v1/zainbox/") {
        if let Some((code, "activity")) = rest.split_once('/') {
            return Some(format!("/api/v1/zainbox/{code}/transactions"));
        }
    }
    if path == "/api/v1/merchant/activity" {
        return Some("/api/v1/merchant/transactions".into());
    }
    if let Some(account) = path.strip_prefix("/api/v1/virtual-account/") {
        if !account.is_empty() && !account.contains('/') {
            return Some(format!("/api/v1/va/{account}"));
        }
    }
    if let Some(txn) = path.strip_prefix("/api/v1/deposit/") {
        if !txn.is_empty() && !txn.contains('/') {
            return Some(format!("/api/v1/dva/verify/{txn}"));
        }
    }
    None
}

/// Axum middleware: stamp deprecation headers on known aliases.
pub async fn deprecation_headers(req: Request<axum::body::Body>, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let successor = concrete_successor(&path);
    let mut res = next.run(req).await;

    if let Some(succ) = successor {
        let headers = res.headers_mut();
        headers.insert("Deprecation", HeaderValue::from_static("true"));
        headers.insert("Sunset", HeaderValue::from_static(SUNSET_HTTP_DATE));
        if let Ok(link) = HeaderValue::from_str(&format!("<{succ}>; rel=\"successor-version\"")) {
            headers.insert("Link", link);
        }
        if let Ok(v) = HeaderValue::from_str(&succ) {
            headers.insert("X-Koodify-Successor", v);
        }
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_zainbox_activity() {
        assert_eq!(
            concrete_successor("/api/v1/zainbox/box1/activity").as_deref(),
            Some("/api/v1/zainbox/box1/transactions")
        );
    }

    #[test]
    fn matches_merchant_activity() {
        assert_eq!(
            concrete_successor("/api/v1/merchant/activity").as_deref(),
            Some("/api/v1/merchant/transactions")
        );
    }

    #[test]
    fn matches_virtual_account_and_deposit() {
        assert_eq!(
            concrete_successor("/api/v1/virtual-account/0123456789").as_deref(),
            Some("/api/v1/va/0123456789")
        );
        assert_eq!(
            concrete_successor("/api/v1/deposit/ORDER_1").as_deref(),
            Some("/api/v1/dva/verify/ORDER_1")
        );
    }

    #[test]
    fn ignores_canonical_paths() {
        assert!(concrete_successor("/api/v1/merchant/transactions").is_none());
        assert!(concrete_successor("/api/v1/dva/verify/ORDER_1").is_none());
        assert!(concrete_successor("/api/v1/activity/ORDER_1").is_none());
    }
}
