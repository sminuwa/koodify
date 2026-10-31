//! Confirm Zainpay VA → VA (or ISA → VA) via `POST /bank/transfer/v2`.
//!
//! Env:
//! - `ZAINPAY_API_KEY` (required)
//! - `ZAINPAY_BASE_URL` (default sandbox: https://sandbox.zainpay.ng)
//! - `PRIMARY_ZAINBOX` / `SECONDARY_ZAINBOX` (optional; otherwise first two from list)
//! - `PROOF_AMOUNT_KOBO` (default `100` = ₦1.00)
//! - `PROOF_EXECUTE=1` to actually transfer (default is dry-run discovery only)
//! - `PROOF_FUND=1` to call mock static fund on primary before transfer (sandbox)
//!
//! Usage:
//!   cargo run -p koodify --example va_to_va_proof
//!   PROOF_EXECUTE=1 cargo run -p koodify --example va_to_va_proof

use anyhow::{bail, Context, Result};
use client::{
    Environment, FundVirtualAccountRequest, FundsTransferRequest, ZainpayClient,
};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();

    let api_key = std::env::var("ZAINPAY_API_KEY").context("ZAINPAY_API_KEY required")?;
    let base_url = std::env::var("ZAINPAY_BASE_URL")
        .unwrap_or_else(|_| Environment::Sandbox.base_url().to_string());
    let execute = env_flag("PROOF_EXECUTE");
    let fund = env_flag("PROOF_FUND");
    let amount_kobo = std::env::var("PROOF_AMOUNT_KOBO").unwrap_or_else(|_| "100".into());

    let client = ZainpayClient::builder()
        .base_url(&base_url)
        .mock_base_url(Environment::Sandbox.mock_base_url())
        .api_key(api_key)
        .build()?;

    println!("base_url={base_url}");
    println!("execute={execute} fund={fund} amount_kobo={amount_kobo}");

    let list = client.zainbox().list(Some("active")).await?;
    println!("\n== zainbox list ==");
    println!("{}", pretty_opt(&list.data));

    let codes = resolve_codes(list.data.as_ref())?;
    let (primary_code, secondary_code) = codes;
    println!("\nprimary={primary_code}");
    println!("secondary={secondary_code}");

    let primary = load_box(&client, &primary_code).await?;
    let secondary = load_box(&client, &secondary_code).await?;

    println!("\n== primary accounts ==");
    print_accounts(&primary);
    println!("\n== secondary accounts ==");
    print_accounts(&secondary);

    let source = pick_source(&primary).context("no usable source account on primary")?;
    let dest = pick_dest(&secondary).context("no usable destination account on secondary")?;

    println!("\n== chosen legs ==");
    println!(
        "source  acct={} bankCode={} bankName={:?} kind={}",
        source.account_number, source.bank_code, source.bank_name, source.kind
    );
    println!(
        "dest    acct={} bankCode={} bankName={:?} kind={}",
        dest.account_number, dest.bank_code, dest.bank_name, dest.kind
    );

    if let Some(bal) = balance_of(&client, &source.account_number).await? {
        println!("source balance response: {bal}");
    }

    if !execute {
        println!("\nDry-run only. Re-run with PROOF_EXECUTE=1 to POST /bank/transfer/v2.");
        if base_url.contains("sandbox") {
            println!("Optional: PROOF_FUND=1 to mock-fund the primary VA first.");
        } else {
            println!("WARNING: base URL looks live — use a tiny PROOF_AMOUNT_KOBO if you execute.");
        }
        return Ok(());
    }

    if fund {
        if !base_url.contains("sandbox") && !base_url.contains("mock") {
            println!("PROOF_FUND ignored outside sandbox (mock fund is sandbox-only).");
        } else {
            let fund_amount = std::env::var("PROOF_FUND_AMOUNT_KOBO")
                .unwrap_or_else(|_| "1000000".into()); // ₦10,000 kobo
            println!("\n== mock fund primary ==");
            let funded = client
                .virtual_accounts()
                .fund_static(&FundVirtualAccountRequest {
                    amount: fund_amount.clone(),
                    account_number: source.account_number.clone(),
                })
                .await?;
            println!("fund amount={fund_amount} → {}", pretty_opt(&funded.data));
            if let Some(bal) = balance_of(&client, &source.account_number).await? {
                println!("source balance after fund: {bal}");
            }
        }
    }

    let txn_ref = format!(
        "split_proof_{}",
        chrono::Utc::now().format("%Y%m%d%H%M%S")
    );

    let req = FundsTransferRequest {
        destination_account_number: dest.account_number.clone(),
        destination_bank_code: dest.bank_code.clone(),
        source_account_number: source.account_number.clone(),
        source_bank_code: source.bank_code.clone(),
        amount: amount_kobo.clone(),
        txn_ref: txn_ref.clone(),
        zainbox_code: primary_code.clone(),
        call_back_url: None,
        narration: Some("koodify va-to-va proof".into()),
    };

    println!("\n== POST /bank/transfer/v2 ==");
    println!("request: {}", pretty(&serde_json::to_value(&req)?));

    let transfer = client.transfers().funds_transfer(&req).await?;
    println!(
        "response code={:?} status={:?} desc={:?}",
        transfer.code, transfer.status, transfer.description
    );
    println!("data: {}", pretty_opt(&transfer.data));

    println!("\n== GET verify/{txn_ref} ==");
    // Brief pause for queued transfers
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    match client.transfers().verify(&txn_ref).await {
        Ok(v) => {
            println!(
                "verify code={:?} status={:?} desc={:?}",
                v.code, v.status, v.description
            );
            println!("data: {}", pretty_opt(&v.data));
        }
        Err(e) => println!("verify error: {e:#}"),
    }

    if let Some(bal) = balance_of(&client, &source.account_number).await? {
        println!("\nsource balance after: {bal}");
    }
    if let Some(bal) = balance_of(&client, &dest.account_number).await? {
        println!("dest balance after: {bal}");
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct Acct {
    kind: &'static str,
    account_number: String,
    bank_code: String,
    bank_name: Option<String>,
}

struct BoxAccounts {
    vas: Vec<Acct>,
    isa: Option<Acct>,
}

async fn load_box(client: &ZainpayClient, code: &str) -> Result<BoxAccounts> {
    let vas_resp = client.zainbox().virtual_accounts(code).await?;
    let isa_resp = client.settlements().internal_account(code).await;

    let mut vas = Vec::new();
    if let Some(data) = vas_resp.data.as_ref() {
        collect_vas(data, &mut vas);
    }

    let isa = match isa_resp {
        Ok(r) => {
            println!("ISA raw for {code}: {}", pretty_opt(&r.data));
            r.data.as_ref().and_then(parse_isa)
        }
        Err(e) => {
            println!("ISA fetch failed for {code}: {e:#}");
            None
        }
    };

    Ok(BoxAccounts { vas, isa })
}

fn collect_vas(data: &Value, out: &mut Vec<Acct>) {
    let items = match data {
        Value::Array(a) => a.as_slice(),
        Value::Object(o) => {
            if let Some(Value::Array(a)) = o.get("accounts").or_else(|| o.get("data")) {
                a.as_slice()
            } else {
                // single object shaped as one VA
                std::slice::from_ref(data)
            }
        }
        _ => return,
    };

    for item in items {
        if let Some(acct) = parse_va(item) {
            out.push(acct);
        }
    }
}

fn parse_va(v: &Value) -> Option<Acct> {
    let account_number = v
        .get("bankAccount")
        .or_else(|| v.get("accountNumber"))
        .or_else(|| v.get("account_number"))
        .and_then(|x| x.as_str())?
        .to_string();
    if account_number.is_empty() {
        return None;
    }
    let bank_type = v
        .get("bankType")
        .and_then(|x| x.as_str())
        .map(str::to_string);
    // Zainpay often puts the NUBAN bank code in `bankName` (e.g. "000013", "090976").
    let bank_name_field = v.get("bankName").and_then(|x| x.as_str()).map(str::to_string);
    let bank_code = v
        .get("bankCode")
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .or_else(|| {
            bank_name_field.as_ref().and_then(|n| {
                if n.chars().all(|c| c.is_ascii_digit()) {
                    Some(n.clone())
                } else {
                    bank_name_to_code(n)
                }
            })
        })
        .or_else(|| bank_type.as_deref().and_then(bank_name_to_code))
        .unwrap_or_default();
    let kind = if bank_type
        .as_deref()
        .is_some_and(|t| t.eq_ignore_ascii_case("ZainBank"))
        || v.get("name")
            .and_then(|x| x.as_str())
            .is_some_and(|n| n.to_ascii_lowercase().contains("internal settlement"))
    {
        "isa"
    } else {
        "va"
    };
    Some(Acct {
        kind,
        account_number,
        bank_code,
        bank_name: bank_type.or(bank_name_field),
    })
}

fn parse_isa(data: &Value) -> Option<Acct> {
    // data may be object or wrapped
    let v = if data.get("bankAccount").is_some() || data.get("accountNumber").is_some() {
        data
    } else if let Some(inner) = data.get("data") {
        inner
    } else {
        data
    };

    let account_number = v
        .get("bankAccount")
        .or_else(|| v.get("accountNumber"))
        .and_then(|x| x.as_str())?
        .to_string();
    let bank_name = v
        .get("bankName")
        .or_else(|| v.get("bankType"))
        .and_then(|x| x.as_str())
        .map(str::to_string);
    // PHP SDK sample: bankName field sometimes holds the code ("035")
    let bank_code = v
        .get("bankCode")
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .or_else(|| {
            bank_name.as_ref().and_then(|n| {
                if n.chars().all(|c| c.is_ascii_digit()) {
                    Some(n.clone())
                } else {
                    bank_name_to_code(n)
                }
            })
        })
        .unwrap_or_default();

    Some(Acct {
        kind: "isa",
        account_number,
        bank_code,
        bank_name,
    })
}

fn bank_name_to_code(name: &str) -> Option<String> {
    // Best-effort map for discovery; live bank/list is authoritative.
    let code = match name.to_ascii_lowercase().as_str() {
        "gtbank" | "gtb" | "000013" => "000013",
        "wemabank" | "wema" | "000017" | "035" => "000017",
        "moniepoint" | "50515" => "090405",
        "sterlingbank" | "sterling" | "000001" => "000001",
        "providusbank" | "providus" | "000023" => "000023",
        "zainbank" | "zainmfb" | "090976" => "090976",
        _ => return None,
    };
    Some(code.into())
}

fn pick_source(b: &BoxAccounts) -> Option<Acct> {
    if let Ok(acct) = std::env::var("PROOF_SOURCE_ACCOUNT") {
        return find_acct(b, &acct);
    }
    // Prefer non-ISA VA (deposit wallets); fall back to ISA.
    b.vas
        .iter()
        .find(|a| a.kind == "va" && !a.bank_code.is_empty())
        .cloned()
        .or_else(|| {
            b.isa
                .clone()
                .filter(|a| !a.bank_code.is_empty() && !a.account_number.is_empty())
        })
        .or_else(|| b.vas.iter().find(|a| !a.bank_code.is_empty()).cloned())
}

fn pick_dest(b: &BoxAccounts) -> Option<Acct> {
    if let Ok(acct) = std::env::var("PROOF_DEST_ACCOUNT") {
        return find_acct(b, &acct);
    }
    // Prefer ISA for cross-zainbox internal settlement; else first VA.
    b.isa
        .clone()
        .filter(|a| !a.bank_code.is_empty())
        .or_else(|| {
            b.vas
                .iter()
                .find(|a| a.kind == "va" && !a.bank_code.is_empty())
                .cloned()
        })
}

fn find_acct(b: &BoxAccounts, acct: &str) -> Option<Acct> {
    b.vas
        .iter()
        .find(|a| a.account_number == acct)
        .cloned()
        .or_else(|| b.isa.clone().filter(|a| a.account_number == acct))
}

fn print_accounts(b: &BoxAccounts) {
    for a in &b.vas {
        println!(
            "  {:<3} {}  bankCode={} bankName={:?}",
            a.kind.to_ascii_uppercase(),
            a.account_number,
            a.bank_code,
            a.bank_name
        );
    }
    if let Some(a) = &b.isa {
        println!(
            "  ISA {}  bankCode={} bankName={:?}",
            a.account_number, a.bank_code, a.bank_name
        );
    }
    if b.vas.is_empty() && b.isa.is_none() {
        println!("  (none)");
    }
}

async fn balance_of(client: &ZainpayClient, account: &str) -> Result<Option<String>> {
    match client.virtual_accounts().balance(account).await {
        Ok(r) => Ok(Some(pretty_opt(
            &r.data
                .as_ref()
                .and_then(|b| serde_json::to_value(b).ok()),
        ))),
        Err(e) => {
            println!("balance fetch failed for {account}: {e:#}");
            Ok(None)
        }
    }
}

fn resolve_codes(data: Option<&Value>) -> Result<(String, String)> {
    if let (Ok(p), Ok(s)) = (
        std::env::var("PRIMARY_ZAINBOX"),
        std::env::var("SECONDARY_ZAINBOX"),
    ) {
        return Ok((p, s));
    }

    let Some(data) = data else {
        bail!("zainbox list returned empty data; set PRIMARY_ZAINBOX and SECONDARY_ZAINBOX");
    };

    let arr = match data {
        Value::Array(a) => a.clone(),
        Value::Object(o) => o
            .get("zainboxes")
            .or_else(|| o.get("data"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default(),
        _ => Vec::new(),
    };

    let mut codes = Vec::new();
    for item in arr {
        if let Some(c) = item
            .get("codeName")
            .or_else(|| item.get("code"))
            .or_else(|| item.get("zainboxCode"))
            .and_then(|v| v.as_str())
        {
            codes.push(c.to_string());
        }
    }
    codes.sort();
    codes.dedup();
    if codes.len() < 2 {
        bail!(
            "need two zainboxes; set PRIMARY_ZAINBOX and SECONDARY_ZAINBOX (found {})",
            codes.len()
        );
    }
    Ok((codes[0].clone(), codes[1].clone()))
}

fn env_flag(name: &str) -> bool {
    matches!(
        std::env::var(name).ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE") | Some("yes")
    )
}

fn pretty(v: &Value) -> String {
    serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
}

fn pretty_opt(v: &Option<Value>) -> String {
    match v {
        Some(val) => pretty(val),
        None => "null".into(),
    }
}
