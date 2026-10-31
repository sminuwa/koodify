//! Resend-backed payer receipt emails.
//!
//! HTML / CSS live under `crates/koodify/templates/emails/` and are loaded at
//! compile time via `include_str!` — keep presentation out of this module.

use resend_rs::types::{CreateAttachment, CreateEmailBaseOptions};
use resend_rs::Resend;
use tracing::{info, warn};

use crate::config::Config;

const RECEIPT_HTML: &str = include_str!("../../templates/emails/receipt.html");
const RECEIPT_CSS: &str = include_str!("../../templates/emails/receipt.css");
const LOGO_PNG: &[u8] = include_bytes!("../../templates/emails/koodify-logo.png");
/// Inline CID referenced by `receipt.html` (`src="cid:koodify-logo"`).
const LOGO_CID: &str = "koodify-logo";

#[derive(Debug, Clone)]
pub struct ReceiptDetail {
    pub label: String,
    pub value: String,
    pub mono: bool,
}

#[derive(Debug, Clone)]
pub struct ReceiptEmail {
    pub to: String,
    pub payer_name: Option<String>,
    pub amount_label: String,
    pub currency: String,
    pub reference: String,
    pub paid_at: Option<String>,
    pub business_name: Option<String>,
    pub narration: Option<String>,
    pub sender_bank: Option<String>,
    pub sender_account: Option<String>,
    pub sender_name: Option<String>,
    /// Extra merchant `customParam` rows (already humanized labels).
    pub custom_params: Vec<(String, String)>,
}

/// Send a branded payment receipt. No-op when `RESEND_API_KEY` is unset.
pub async fn send_payment_receipt(config: &Config, receipt: ReceiptEmail) -> anyhow::Result<()> {
    let Some(api_key) = config.resend_api_key.as_deref() else {
        warn!(
            reference = %receipt.reference,
            "RESEND_API_KEY unset — skipping payer receipt email"
        );
        return Ok(());
    };

    let to = receipt.to.trim();
    if to.is_empty() || !to.contains('@') {
        warn!(
            reference = %receipt.reference,
            "payer email missing/invalid — skipping receipt"
        );
        return Ok(());
    }

    let resend = Resend::new(api_key);
    let subject = format!(
        "Payment received · {} {}",
        receipt.currency, receipt.amount_label
    );
    let html = render_receipt_html(&receipt);
    let logo = CreateAttachment::from_content(LOGO_PNG.to_vec())
        .with_filename("koodify-logo.png")
        .with_content_type("image/png")
        .with_content_id(LOGO_CID);
    let email = CreateEmailBaseOptions::new(config.resend_from.clone(), [to.to_string()], subject)
        .with_html(&html)
        .with_attachment(logo);

    let sent = resend.emails.send(email).await.map_err(|err| {
        anyhow::anyhow!("resend send failed for {}: {err}", receipt.reference)
    })?;

    info!(
        reference = %receipt.reference,
        to,
        email_id = ?sent.id,
        "payer receipt email sent"
    );
    Ok(())
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn opt_trim(s: Option<&String>) -> Option<&str> {
    s.map(|v| v.as_str().trim()).filter(|v| !v.is_empty())
}

fn detail_rows(receipt: &ReceiptEmail) -> Vec<ReceiptDetail> {
    let mut rows = Vec::new();

    if let Some(name) = opt_trim(receipt.business_name.as_ref()) {
        rows.push(ReceiptDetail {
            label: "Business Name".into(),
            value: name.into(),
            mono: false,
        });
    }

    rows.push(ReceiptDetail {
        label: "Reference".into(),
        value: receipt.reference.clone(),
        mono: true,
    });

    if let Some(n) = opt_trim(receipt.narration.as_ref()) {
        rows.push(ReceiptDetail {
            label: "Narration".into(),
            value: n.into(),
            mono: false,
        });
    }

    let paid_at = opt_trim(receipt.paid_at.as_ref()).unwrap_or("Just now");
    rows.push(ReceiptDetail {
        label: "Date".into(),
        value: paid_at.into(),
        mono: false,
    });

    if let Some(bank) = opt_trim(receipt.sender_bank.as_ref()) {
        rows.push(ReceiptDetail {
            label: "Sender's Bank".into(),
            value: bank.into(),
            mono: false,
        });
    }

    if let Some(acct) = opt_trim(receipt.sender_account.as_ref()) {
        let value = match opt_trim(receipt.sender_name.as_ref()) {
            Some(n) => format!("{n} · {acct}"),
            None => acct.into(),
        };
        rows.push(ReceiptDetail {
            label: "Sender's Account".into(),
            value,
            mono: true,
        });
    } else if let Some(n) = opt_trim(receipt.sender_name.as_ref()) {
        rows.push(ReceiptDetail {
            label: "Sender".into(),
            value: n.into(),
            mono: false,
        });
    }

    for (label, value) in &receipt.custom_params {
        let l = label.trim();
        let v = value.trim();
        if l.is_empty() || v.is_empty() {
            continue;
        }
        rows.push(ReceiptDetail {
            label: l.into(),
            value: v.into(),
            mono: false,
        });
    }

    rows
}

fn render_detail_rows_html(rows: &[ReceiptDetail]) -> String {
    let mut out = String::new();
    let last = rows.len().saturating_sub(1);
    for (i, row) in rows.iter().enumerate() {
        let border = if i == last {
            ""
        } else {
            "border-bottom:1px solid #d5deea;"
        };
        let mono = if row.mono {
            "font-family:ui-monospace,Menlo,Consolas,monospace;"
        } else {
            ""
        };
        out.push_str(&format!(
            r#"                <tr>
                  <td class="kd-row-label" style="padding:10px 0;{border}font-size:13px;color:#334e6d;">{label}</td>
                  <td class="kd-row-value" align="right" style="padding:10px 0;{border}font-size:13px;font-weight:700;color:#061526;{mono}">{value}</td>
                </tr>
"#,
            border = border,
            label = escape(&row.label),
            value = escape(&row.value),
            mono = mono,
        ));
    }
    out
}

pub fn render_receipt_html(receipt: &ReceiptEmail) -> String {
    let name = opt_trim(receipt.payer_name.as_ref()).unwrap_or("there");
    let amount = escape(&receipt.amount_label);
    let currency = escape(&receipt.currency);
    let payer_email = escape(receipt.to.trim());
    let greeting = escape(name);
    let rows_html = render_detail_rows_html(&detail_rows(receipt));

    RECEIPT_HTML
        .replace("{{styles}}", RECEIPT_CSS)
        .replace("{{logo_src}}", &format!("cid:{LOGO_CID}"))
        .replace("{{currency}}", &currency)
        .replace("{{amount}}", &amount)
        .replace("{{payer_email}}", &payer_email)
        .replace("{{greeting}}", &greeting)
        .replace("{{detail_rows}}", &rows_html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipt_html_includes_brand_and_rich_fields() {
        let html = render_receipt_html(&ReceiptEmail {
            to: "payer@example.com".into(),
            payer_name: Some("Ada".into()),
            amount_label: "500".into(),
            currency: "NGN".into(),
            reference: "ORDER_123".into(),
            paid_at: Some("2026-10-31 14:03".into()),
            business_name: Some("Acme Stores".into()),
            narration: Some("School fees".into()),
            sender_bank: Some("GTBank".into()),
            sender_account: Some("0123456789".into()),
            sender_name: Some("Ada Lovelace".into()),
            custom_params: vec![("Student Id".into(), "STU001".into())],
        });
        assert!(html.contains("Koodify"));
        assert!(html.contains(r#"src="cid:koodify-logo""#));
        assert!(html.contains("ORDER_123"));
        assert!(html.contains("NGN 500"));
        assert!(html.contains("Hi Ada"));
        assert!(html.contains("payer@example.com"));
        assert!(html.contains("Acme Stores"));
        assert!(html.contains("School fees"));
        assert!(html.contains("GTBank"));
        assert!(html.contains("Ada Lovelace · 0123456789"));
        assert!(html.contains("Student Id"));
        assert!(html.contains("STU001"));
        assert!(html.contains("Secure bank-transfer checkout"));
        assert!(!html.contains("{{"));
        assert!(!LOGO_PNG.is_empty());
    }
}
