//! Resend-backed payer receipt emails.

use resend_rs::types::CreateEmailBaseOptions;
use resend_rs::Resend;
use tracing::{info, warn};

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct ReceiptEmail {
    pub to: String,
    pub payer_name: Option<String>,
    pub amount_label: String,
    pub currency: String,
    pub reference: String,
    pub paid_at: Option<String>,
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
    let email = CreateEmailBaseOptions::new(config.resend_from.clone(), [to.to_string()], subject)
        .with_html(&html);

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

pub fn render_receipt_html(receipt: &ReceiptEmail) -> String {
    let name = receipt
        .payer_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("there");
    let amount = escape(&receipt.amount_label);
    let currency = escape(&receipt.currency);
    let reference = escape(&receipt.reference);
    let paid_at = receipt
        .paid_at
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(escape)
        .unwrap_or_else(|| "Just now".into());
    let greeting = escape(name);

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Payment receipt · Koodify</title>
</head>
<body style="margin:0;padding:0;background:#eef3f9;font-family:'DM Sans',Helvetica,Arial,sans-serif;color:#061526;">
  <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="background:#eef3f9;padding:32px 16px;">
    <tr>
      <td align="center">
        <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="max-width:560px;background:#ffffff;border-radius:16px;overflow:hidden;border:1px solid #c5d3e6;">
          <tr>
            <td style="background:linear-gradient(135deg,#0f57d9,#0a3a9a);padding:28px 32px;">
              <p style="margin:0;font-size:13px;font-weight:700;letter-spacing:0.12em;text-transform:uppercase;color:rgba(255,255,255,0.78);">Koodify</p>
              <h1 style="margin:10px 0 0;font-size:26px;line-height:1.2;font-weight:800;color:#ffffff;letter-spacing:-0.02em;">Payment received</h1>
            </td>
          </tr>
          <tr>
            <td style="padding:28px 32px 8px;">
              <p style="margin:0 0 16px;font-size:16px;line-height:1.55;color:#2a405c;">
                Hi {greeting},
              </p>
              <p style="margin:0 0 24px;font-size:16px;line-height:1.55;color:#2a405c;">
                We’ve confirmed your bank transfer. Here’s a quick receipt for your records.
              </p>
              <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="background:#f4f8ff;border-radius:12px;border:1px solid #d9e6ff;">
                <tr>
                  <td style="padding:20px 22px;">
                    <p style="margin:0 0 4px;font-size:12px;font-weight:700;letter-spacing:0.06em;text-transform:uppercase;color:#5b6b8c;">Amount paid</p>
                    <p style="margin:0;font-size:28px;font-weight:800;color:#0f57d9;letter-spacing:-0.02em;">{currency} {amount}</p>
                  </td>
                </tr>
              </table>
              <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="margin-top:18px;">
                <tr>
                  <td style="padding:10px 0;border-bottom:1px solid #e8eef6;font-size:13px;color:#5b6b8c;">Reference</td>
                  <td align="right" style="padding:10px 0;border-bottom:1px solid #e8eef6;font-size:13px;font-weight:700;color:#061526;font-family:ui-monospace,Menlo,Consolas,monospace;">{reference}</td>
                </tr>
                <tr>
                  <td style="padding:10px 0;font-size:13px;color:#5b6b8c;">Paid at</td>
                  <td align="right" style="padding:10px 0;font-size:13px;font-weight:600;color:#061526;">{paid_at}</td>
                </tr>
              </table>
              <p style="margin:24px 0 0;font-size:14px;line-height:1.55;color:#5b6b8c;">
                If you didn’t make this payment, reply to this email or contact the merchant who sent you the checkout link.
              </p>
            </td>
          </tr>
          <tr>
            <td style="padding:20px 32px 28px;">
              <p style="margin:0;font-size:12px;color:#8a9bb3;">
                Powered by <strong style="color:#0f57d9;">Koodify</strong> · Secure bank-transfer checkout
              </p>
            </td>
          </tr>
        </table>
      </td>
    </tr>
  </table>
</body>
</html>"##
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipt_html_includes_brand_and_reference() {
        let html = render_receipt_html(&ReceiptEmail {
            to: "payer@example.com".into(),
            payer_name: Some("Ada".into()),
            amount_label: "500".into(),
            currency: "NGN".into(),
            reference: "ORDER_123".into(),
            paid_at: Some("2026-10-31".into()),
        });
        assert!(html.contains("Koodify"));
        assert!(html.contains("ORDER_123"));
        assert!(html.contains("NGN 500"));
        assert!(html.contains("Hi Ada"));
    }
}
