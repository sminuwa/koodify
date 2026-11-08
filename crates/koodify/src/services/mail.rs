//! Resend-backed payment emails (payer receipt + merchant notice).
//!
//! HTML / CSS live under `crates/koodify/templates/emails/` and are loaded at
//! compile time via `include_str!` — keep presentation out of this module.

use resend_rs::types::{CreateAttachment, CreateEmailBaseOptions};
use resend_rs::Resend;
use tracing::{info, warn};

use crate::config::Config;

const RECEIPT_HTML: &str = include_str!("../../templates/emails/receipt.html");
const MERCHANT_HTML: &str = include_str!("../../templates/emails/merchant-notice.html");
const RECEIPT_CSS: &str = include_str!("../../templates/emails/receipt.css");
const LOGO_PNG: &[u8] = include_bytes!("../../templates/emails/koodify-logo.png");
/// Inline CID referenced by templates (`src="cid:koodify-logo"`).
const LOGO_CID: &str = "koodify-logo";

#[derive(Debug, Clone)]
pub struct ReceiptDetail {
    pub label: String,
    pub value: String,
    pub mono: bool,
}

#[derive(Debug, Clone)]
pub struct ReceiptEmail {
    /// Recipient address (payer or merchant).
    pub to: String,
    pub payer_name: Option<String>,
    /// Displayed under the amount; for payer receipts usually equals `to`.
    pub payer_email: Option<String>,
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

/// Send a branded payer receipt. No-op when `RESEND_API_KEY` is unset.
pub async fn send_payment_receipt(config: &Config, receipt: ReceiptEmail) -> anyhow::Result<()> {
    send_templated(
        config,
        &receipt,
        &format!(
            "Payment received · {} {}",
            receipt.currency, receipt.amount_label
        ),
        render_receipt_html(&receipt),
        "payer receipt",
    )
    .await
}

/// Send a merchant / zainbox settlement alert. No-op when Resend is unset.
pub async fn send_merchant_notice(config: &Config, notice: ReceiptEmail) -> anyhow::Result<()> {
    send_templated(
        config,
        &notice,
        &format!(
            "Payment delivered · {} {} · {}",
            notice.currency, notice.amount_label, notice.reference
        ),
        render_merchant_notice_html(&notice),
        "merchant notice",
    )
    .await
}

/// Notify a merchant that KYC was approved and they can issue API secrets.
pub async fn send_kyc_approved(
    config: &Config,
    to: &str,
    business_name: &str,
    public_key: Option<&str>,
) -> anyhow::Result<()> {
    let name = escape(business_name.trim());
    let pk_row = public_key
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|pk| {
            format!(
                "<p style=\"margin:16px 0 0;font-size:14px;color:#334155\">Publishable key: \
                 <code style=\"background:#f1f5f9;padding:2px 6px;border-radius:4px\">{pk}</code></p>",
                pk = escape(pk)
            )
        })
        .unwrap_or_default();
    let html = format!(
        r#"<!doctype html>
<html><body style="margin:0;background:#f8fafc;font-family:Inter,Segoe UI,sans-serif;color:#0f172a">
  <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="padding:32px 16px">
    <tr><td align="center">
      <table role="presentation" width="560" cellspacing="0" cellpadding="0" style="background:#fff;border:1px solid #e2e8f0;border-radius:16px;padding:28px 32px">
        <tr><td>
          <img src="cid:koodify-logo" alt="Koodify" height="28" style="display:block;margin-bottom:20px"/>
          <h1 style="margin:0 0 8px;font-size:22px;letter-spacing:-0.02em">Your account is approved</h1>
          <p style="margin:0;font-size:15px;line-height:1.55;color:#475569">
            Hi {name}, your Koodify merchant KYC has been approved. Your payment account is being provisioned.
          </p>
          {pk_row}
          <p style="margin:18px 0 0;font-size:15px;line-height:1.55;color:#475569">
            Sign in to your merchant portal and open <strong>Credentials</strong> to generate your secret API key (<code>sk_…</code>).
            Keep that secret on your server only.
          </p>
          <p style="margin:24px 0 0;font-size:12px;color:#94a3b8">© Koodify · Account verification</p>
        </td></tr>
      </table>
    </td></tr>
  </table>
</body></html>"#,
        name = name,
        pk_row = pk_row,
    );

    let mail = ReceiptEmail {
        to: to.to_string(),
        payer_name: Some(business_name.to_string()),
        payer_email: Some(to.to_string()),
        amount_label: "—".into(),
        currency: "NGN".into(),
        reference: format!("kyc-approved-{}", chrono::Utc::now().timestamp()),
        paid_at: None,
        business_name: Some(business_name.to_string()),
        narration: Some("KYC approved".into()),
        sender_bank: None,
        sender_account: None,
        sender_name: None,
        custom_params: vec![],
    };
    send_templated(
        config,
        &mail,
        "Your Koodify account is approved",
        html,
        "kyc approved",
    )
    .await
}

async fn send_templated(
    config: &Config,
    mail: &ReceiptEmail,
    subject: &str,
    html: String,
    kind: &str,
) -> anyhow::Result<()> {
    let Some(api_key) = config.resend_api_key.as_deref() else {
        warn!(
            reference = %mail.reference,
            kind,
            "RESEND_API_KEY unset — skipping email"
        );
        return Ok(());
    };

    let to = mail.to.trim();
    if to.is_empty() || !to.contains('@') {
        warn!(
            reference = %mail.reference,
            kind,
            "recipient email missing/invalid — skipping"
        );
        return Ok(());
    }

    let resend = Resend::new(api_key);
    let logo = CreateAttachment::from_content(LOGO_PNG.to_vec())
        .with_filename("koodify-logo.png")
        .with_content_type("image/png")
        .with_content_id(LOGO_CID);
    let email = CreateEmailBaseOptions::new(config.resend_from.clone(), [to.to_string()], subject)
        .with_html(&html)
        .with_attachment(logo);

    let sent = resend.emails.send(email).await.map_err(|err| {
        anyhow::anyhow!("resend {kind} failed for {}: {err}", mail.reference)
    })?;

    info!(
        reference = %mail.reference,
        to,
        kind,
        email_id = ?sent.id,
        "payment email sent"
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

fn payer_detail_rows(receipt: &ReceiptEmail) -> Vec<ReceiptDetail> {
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

    append_sender_rows(&mut rows, receipt);

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

fn merchant_detail_rows(notice: &ReceiptEmail) -> Vec<ReceiptDetail> {
    let mut rows = Vec::new();

    if let Some(name) = opt_trim(notice.payer_name.as_ref()) {
        rows.push(ReceiptDetail {
            label: "Payer".into(),
            value: name.into(),
            mono: false,
        });
    }

    if let Some(email) = opt_trim(notice.payer_email.as_ref()) {
        rows.push(ReceiptDetail {
            label: "Payer Email".into(),
            value: email.into(),
            mono: true,
        });
    }

    rows.push(ReceiptDetail {
        label: "Reference".into(),
        value: notice.reference.clone(),
        mono: true,
    });

    if let Some(n) = opt_trim(notice.narration.as_ref()) {
        rows.push(ReceiptDetail {
            label: "Narration".into(),
            value: n.into(),
            mono: false,
        });
    }

    let paid_at = opt_trim(notice.paid_at.as_ref()).unwrap_or("Just now");
    rows.push(ReceiptDetail {
        label: "Date".into(),
        value: paid_at.into(),
        mono: false,
    });

    append_sender_rows(&mut rows, notice);

    if let Some(name) = opt_trim(notice.business_name.as_ref()) {
        rows.push(ReceiptDetail {
            label: "Zainbox".into(),
            value: name.into(),
            mono: false,
        });
    }

    for (label, value) in &notice.custom_params {
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

fn append_sender_rows(rows: &mut Vec<ReceiptDetail>, mail: &ReceiptEmail) {
    if let Some(bank) = opt_trim(mail.sender_bank.as_ref()) {
        rows.push(ReceiptDetail {
            label: "Sender's Bank".into(),
            value: bank.into(),
            mono: false,
        });
    }

    if let Some(acct) = opt_trim(mail.sender_account.as_ref()) {
        let value = match opt_trim(mail.sender_name.as_ref()) {
            Some(n) => format!("{n} · {acct}"),
            None => acct.into(),
        };
        rows.push(ReceiptDetail {
            label: "Sender's Account".into(),
            value,
            mono: true,
        });
    } else if let Some(n) = opt_trim(mail.sender_name.as_ref()) {
        rows.push(ReceiptDetail {
            label: "Sender".into(),
            value: n.into(),
            mono: false,
        });
    }
}

fn render_detail_rows_html(rows: &[ReceiptDetail]) -> String {
    let mut out = String::new();
    let last = rows.len().saturating_sub(1);
    for (i, row) in rows.iter().enumerate() {
        let border = if i == last {
            "border-bottom:0;"
        } else {
            "border-bottom:1px solid #e3ebf8;"
        };
        let mono = if row.mono {
            "font-family:ui-monospace,Menlo,Consolas,monospace;"
        } else {
            ""
        };
        out.push_str(&format!(
            r#"                      <tr>
                        <td class="detail-label" style="padding:8px 4px;{border}width:36%;color:#2a4066;font-size:13px;">{label}</td>
                        <td class="detail-value" style="padding:8px 4px;{border}color:#12213e;text-align:right;font-weight:700;font-size:13px;{mono}">{value}</td>
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

fn display_payer_email(mail: &ReceiptEmail) -> String {
    opt_trim(mail.payer_email.as_ref())
        .unwrap_or_else(|| mail.to.trim())
        .to_string()
}

pub fn render_receipt_html(receipt: &ReceiptEmail) -> String {
    let name = opt_trim(receipt.payer_name.as_ref()).unwrap_or("there");
    let amount = escape(&receipt.amount_label);
    let currency = escape(&receipt.currency);
    let payer_email = escape(&display_payer_email(receipt));
    let greeting = escape(name);
    let rows_html = render_detail_rows_html(&payer_detail_rows(receipt));

    RECEIPT_HTML
        .replace("{{styles}}", RECEIPT_CSS)
        .replace("{{logo_src}}", &format!("cid:{LOGO_CID}"))
        .replace("{{currency}}", &currency)
        .replace("{{amount}}", &amount)
        .replace("{{payer_email}}", &payer_email)
        .replace("{{greeting}}", &greeting)
        .replace("{{detail_rows}}", &rows_html)
}

pub fn render_merchant_notice_html(notice: &ReceiptEmail) -> String {
    let name = opt_trim(notice.business_name.as_ref()).unwrap_or("there");
    let amount = escape(&notice.amount_label);
    let currency = escape(&notice.currency);
    let payer_email = escape(&display_payer_email(notice));
    let greeting = escape(name);
    let reference = escape(&notice.reference);
    let rows_html = render_detail_rows_html(&merchant_detail_rows(notice));

    MERCHANT_HTML
        .replace("{{styles}}", RECEIPT_CSS)
        .replace("{{logo_src}}", &format!("cid:{LOGO_CID}"))
        .replace("{{currency}}", &currency)
        .replace("{{amount}}", &amount)
        .replace("{{payer_email}}", &payer_email)
        .replace("{{greeting}}", &greeting)
        .replace("{{reference}}", &reference)
        .replace("{{detail_rows}}", &rows_html)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_mail() -> ReceiptEmail {
        ReceiptEmail {
            to: "payer@example.com".into(),
            payer_name: Some("Ada".into()),
            payer_email: Some("payer@example.com".into()),
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
        }
    }

    #[test]
    fn receipt_html_includes_brand_and_rich_fields() {
        let html = render_receipt_html(&sample_mail());
        assert!(html.contains("Koodify"));
        assert!(html.contains(r#"src="cid:koodify-logo""#));
        assert!(html.contains("ORDER_123"));
        assert!(html.contains("NGN&nbsp;500") || html.contains("NGN 500"));
        assert!(html.contains("Hi Ada"));
        assert!(html.contains("payer@example.com"));
        assert!(html.contains("Acme Stores"));
        assert!(html.contains("School fees"));
        assert!(html.contains("GTBank"));
        assert!(html.contains("Ada Lovelace · 0123456789"));
        assert!(html.contains("Student Id"));
        assert!(html.contains("STU001"));
        assert!(html.contains("Secure bank-transfer checkout"));
        assert!(html.contains("Payment Notification"));
        assert!(html.contains("Amount Paid"));
        assert!(html.contains("Bank Transfer"));
        assert!(html.contains("detail-label"));
        assert!(html.contains("stack-col"));
        assert!(html.contains("notif-text"));
        assert!(!html.contains("{{"));
        assert!(!LOGO_PNG.is_empty());
    }

    #[test]
    fn merchant_notice_html_is_ops_facing() {
        let mut notice = sample_mail();
        notice.to = "merchant@acme.test".into();
        let html = render_merchant_notice_html(&notice);
        assert!(html.contains("Payment Delivered"));
        assert!(html.contains("Amount Received"));
        assert!(html.contains("Merchant Alert"));
        assert!(html.contains("Hi Acme Stores"));
        assert!(html.contains("Payer"));
        assert!(html.contains("Ada"));
        assert!(html.contains("Payer Email"));
        assert!(html.contains("payer@example.com"));
        assert!(html.contains("ORDER_123"));
        assert!(html.contains("zainbox notification"));
        assert!(!html.contains("{{"));
        assert!(!html.contains("merchant@acme.test")); // recipient not echoed in body
    }

    /// Live sample send: `SAMPLE_RECEIPT_TO=you@example.com cargo test -p koodify --lib send_sample_receipt -- --ignored --nocapture`
    #[tokio::test]
    #[ignore = "live Resend send — set SAMPLE_RECEIPT_TO"]
    async fn send_sample_receipt() {
        let _ = dotenvy::from_filename(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.env"),
        );
        let to = std::env::var("SAMPLE_RECEIPT_TO").expect("SAMPLE_RECEIPT_TO required");
        let api_key = std::env::var("RESEND_API_KEY").expect("RESEND_API_KEY required");
        let from = std::env::var("RESEND_FROM")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "Koodify <onboarding@resend.dev>".into());

        let mut config = crate::config::Config::new(
            "127.0.0.1:0",
            "postgres://unused",
            "https://api.zainpay.ng",
            "unused",
            "unused",
        );
        config.resend_api_key = Some(api_key);
        config.resend_from = from;

        let receipt = ReceiptEmail {
            to: to.clone(),
            payer_name: Some("Sunusi".into()),
            payer_email: Some(to.clone()),
            amount_label: "500".into(),
            currency: "NGN".into(),
            reference: "SAMPLE_gtj26mi1_1793451647335".into(),
            paid_at: Some("31 Oct 2026 · 14:14".into()),
            business_name: Some("Acme Stores".into()),
            narration: Some("School fees".into()),
            sender_bank: Some("GTBank".into()),
            sender_account: Some("0123456789".into()),
            sender_name: Some("Ada Lovelace".into()),
            custom_params: vec![
                ("Student Id".into(), "STU001".into()),
                ("Payment Type".into(), "School Fees".into()),
            ],
        };

        send_payment_receipt(&config, receipt.clone())
            .await
            .expect("sample receipt send failed");
        eprintln!("sample payer receipt sent to {to}");

        let mut merchant = receipt;
        merchant.to = to.clone();
        send_merchant_notice(&config, merchant)
            .await
            .expect("sample merchant notice send failed");
        eprintln!("sample merchant notice sent to {to}");
    }
}
