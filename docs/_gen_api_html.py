#!/usr/bin/env python3
"""Generate docs/api.html with sidebar + sample responses."""

from __future__ import annotations

import html as html_lib
import json
from pathlib import Path


def j(obj) -> str:
    if isinstance(obj, str):
        return html_lib.escape(obj)
    return html_lib.escape(json.dumps(obj, indent=2))


def zp(data, description: str = "successful"):
    return {
        "code": "00",
        "status": "200 OK",
        "description": description,
        "data": data,
    }


def composite(description: str, **parts):
    return {
        "code": "00",
        "status": "200 OK",
        "description": description,
        "data": {k: zp(v) for k, v in parts.items()},
    }


METHOD = {"GET": "emerald", "POST": "sky", "PUT": "violet", "PATCH": "amber"}


def method_badge(m: str) -> str:
    c = METHOD.get(m, "slate")
    return (
        f'<span class="rounded-md bg-{c}-100 px-2 py-1 font-mono '
        f'text-xs font-semibold text-{c}-800">{m}</span>'
    )


def endpoint(
    eid: str,
    method: str,
    path: str,
    title: str,
    summary: str,
    sample,
    *,
    query: str | None = None,
    body: str | None = None,
    combines: list[str] | None = None,
    note: str | None = None,
    sample_label: str = "Sample response",
):
    extra = ""
    if note:
        extra += f'<p class="mt-3 text-sm leading-6 text-ink-700">{note}</p>'
    if query:
        extra += (
            '<p class="mt-5 text-xs font-semibold uppercase tracking-[0.16em] text-ink-500">'
            "Query</p>"
            f'<p class="mt-2 font-mono text-sm text-ink-700">{query}</p>'
        )
    if body:
        extra += (
            '<p class="mt-5 text-xs font-semibold uppercase tracking-[0.16em] text-ink-500">'
            "Body</p>"
            f'<p class="mt-2 text-sm leading-6 text-ink-700">{body}</p>'
        )
    if combines:
        items = "".join(f"<li>{c}</li>" for c in combines)
        extra += (
            '<p class="mt-5 text-xs font-semibold uppercase tracking-[0.16em] text-ink-500">'
            "Combines</p>"
            f'<ul class="mt-2 space-y-1 font-mono text-xs text-ink-700">{items}</ul>'
        )
    return {
        "id": eid,
        "method": method,
        "path": path,
        "title": title,
        "html": f"""
<article id="{eid}" class="scroll-mt-24 border-b border-slate-200 py-10">
  <div class="grid gap-8 xl:grid-cols-2">
    <div>
      <div class="flex flex-wrap items-center gap-3">{method_badge(method)}</div>
      <h3 class="mt-3 text-2xl font-semibold tracking-tight text-ink-950">{title}</h3>
      <p class="mt-2 break-all font-mono text-sm text-accent">{path}</p>
      <p class="mt-4 text-base leading-7 text-ink-700">{summary}</p>
      {extra}
    </div>
    <div>
      <p class="text-xs font-semibold uppercase tracking-[0.16em] text-ink-500">{sample_label}</p>
      <pre class="mt-3 max-h-[28rem] overflow-auto rounded-lg bg-ink-950 p-4 font-mono text-xs leading-relaxed text-slate-100"><code>{j(sample)}</code></pre>
    </div>
  </div>
</article>""",
    }


def main() -> None:
    customer = {
        "uuid": "814e819e-34ce-4853-b21a-1ed38ebad3e1",
        "id": 1,
        "displayName": "kamfany",
        "email": "itkamfany@gmail.com",
        "mobileNumber": None,
        "zainboxCode": "charg_0atrmzL5qsMFKGYPrahj",
        "metadata": {"source": "zainbox-sync"},
        "status": "active",
        "createdAt": "2026-10-30T11:00:00Z",
        "updatedAt": "2026-10-30T11:00:00Z",
    }
    kyc = {
        "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "customerId": 1,
        "title": "Mr",
        "firstName": "Muhammad",
        "surname": "Abdullahi",
        "email": "muhammad@example.com",
        "mobileNumber": "08012345678",
        "bvn": "12345678901",
        "zainboxCode": "charg_0atrmzL5qsMFKGYPrahj",
        "status": "active",
    }
    cred = {
        "id": "2b018720-a714-4e9e-861f-488e7e795d9f",
        "customerId": 1,
        "environment": "live",
        "publicKey": "pk_live_…",
        "zainboxCode": "charg_0atrmzL5qsMFKGYPrahj",
        "zainboxCodeSet": True,
        "status": "active",
    }

    sections: list[dict] = []

    def sec(sid: str, title: str, endpoints: list):
        sections.append({"id": sid, "title": title, "endpoints": endpoints})

    sec(
        "health",
        "Health",
        [
            endpoint(
                "health-check",
                "GET",
                "/health",
                "Health check",
                "Returns plain text. No auth required.",
                "ok",
            )
        ],
    )

    sec(
        "api-v1",
        "API v1 convenience",
        [
            endpoint(
                "api-zainbox-snapshot",
                "GET",
                "/api/v1/zainbox/{zainboxCode}",
                "Zainbox snapshot",
                "Profile + balances for one zainbox.",
                composite(
                    "composite: zainbox profile + balances",
                    profile={
                        "name": "kamfany",
                        "codeName": "charg_0atrmzL5qsMFKGYPrahj",
                        "isActive": True,
                    },
                    balances=[{"accountNumber": "5006079921", "balance": "125000.00"}],
                ),
                combines=[
                    "GET /zainbox/profile/{zainboxCode}",
                    "GET /zainbox/accounts/balance/{zainboxCode}",
                ],
            ),
            endpoint(
                "api-zainbox-wallet",
                "GET",
                "/api/v1/zainbox/{zainboxCode}/wallet",
                "Zainbox wallet",
                "Virtual accounts + balances.",
                composite(
                    "composite: zainbox virtual accounts + balances",
                    virtualAccounts=[
                        {"accountNumber": "5006079921", "accountName": "KAMFANY LTD"}
                    ],
                    balances=[{"accountNumber": "5006079921", "balance": "125000.00"}],
                ),
                combines=[
                    "GET /zainbox/virtual-accounts/{zainboxCode}",
                    "GET /zainbox/accounts/balance/{zainboxCode}",
                ],
            ),
            endpoint(
                "api-zainbox-transactions",
                "GET",
                "/api/v1/zainbox/{zainboxCode}/transactions",
                "Zainbox transactions",
                "Transactions + deposit summary. Alias: /api/v1/zainbox/{zainboxCode}/activity (deprecated).",
                composite(
                    "composite: zainbox transactions + deposit summary",
                    transactions=[
                        {"txnRef": "txn_1", "amount": "5000", "txnType": "credit"}
                    ],
                    depositSummary={"totalDeposits": "150000", "count": 12},
                ),
                query="count, txnType, accountNumber, paymentChannel, dateFrom, dateTo",
                combines=[
                    "GET /zainbox/transactions/{zainboxCode}",
                    "GET /zainbox/transfer/deposit/summary/{zainboxCode}",
                ],
            ),
            endpoint(
                "api-zainbox-settlement",
                "GET",
                "/api/v1/zainbox/{zainboxCode}/settlement",
                "Settlement context",
                "Settlement config + internal settlement account.",
                composite(
                    "composite: settlement config + internal account",
                    settlement={
                        "zainboxCode": "charg_0atrmzL5qsMFKGYPrahj",
                        "scheduleType": "auto",
                    },
                    internalAccount={
                        "accountNumber": "5006079921",
                        "bankName": "Zainpay MFB",
                    },
                ),
                note="Live settlement GET may return HTTP 405.",
                combines=[
                    "GET /zainbox/settlement?zainboxCode=",
                    "GET /zainbox/internal-settlement-account/{zainboxCode}",
                ],
            ),
            endpoint(
                "api-zainbox-billing",
                "GET",
                "/api/v1/zainbox/{zainboxCode}/billing",
                "Profile + billing",
                "Profile alongside billing config.",
                composite(
                    "composite: zainbox profile + billing config",
                    profile={
                        "name": "kamfany",
                        "codeName": "charg_0atrmzL5qsMFKGYPrahj",
                    },
                    billingConfig={"feeBearer": "merchant"},
                ),
                combines=[
                    "GET /zainbox/profile/{zainboxCode}",
                    "GET /zainbox/billing/config/{zainboxCode}",
                ],
            ),
            endpoint(
                "api-zainbox-subscriptions",
                "GET",
                "/api/v1/zainbox/{zainboxCode}/subscriptions",
                "Subscriptions bundle",
                "All + active subscriptions.",
                composite(
                    "composite: subscriptions all + active",
                    all=[{"serviceName": "virtualAccount", "status": "active"}],
                    active=[{"serviceName": "virtualAccount", "status": "active"}],
                ),
                combines=[
                    "…/subscriptions/all/{zainboxCode}",
                    "…/subscriptions/active/{zainboxCode}",
                ],
            ),
            endpoint(
                "api-va-wallet",
                "GET",
                "/api/v1/va/{accountNumber}",
                "VA wallet (composite)",
                "Balance + transactions for one VA. Alias: /api/v1/virtual-account/{accountNumber} (deprecated).",
                composite(
                    "composite: virtual-account balance + transactions",
                    balance={
                        "accountNumber": "5006079921",
                        "balanceAmount": "125000.00",
                    },
                    transactions=[{"txnRef": "txn_1", "amount": "5000"}],
                ),
                query="count, txnType, paymentChannel, dateFrom, dateTo",
                combines=[
                    "GET /virtual-account/wallet/balance/v2/{accountNumber}",
                    "GET /virtual-account/wallet/transactions/{accountNumber}",
                ],
            ),
            endpoint(
                "api-bank-resolve",
                "GET",
                "/api/v1/banks/resolve",
                "Name resolve",
                "Bank list + name enquiry. Alias: /api/v1/bank/resolve.",
                composite(
                    "composite: bank list + name enquiry",
                    banks=[{"code": "000013", "name": "GTBank"}],
                    nameEnquiry={
                        "accountName": "ZNPAY SUNUSI INUWA",
                        "accountNumber": "5006079921",
                    },
                ),
                query="bankCode, accountNumber",
                combines=["GET /bank/list", "GET /bank/name-enquiry"],
            ),
            endpoint(
                "api-merchant-transactions",
                "GET",
                "/api/v1/merchant/transactions",
                "Merchant transactions",
                "Merchant-wide summary + transaction list. Alias: /api/v1/merchant/activity (deprecated).",
                composite(
                    "composite: merchant transactions summary + list",
                    summary={"totalCredit": "1000000", "totalDebit": "250000"},
                    transactions=[{"txnRef": "txn_9", "amount": "10000"}],
                ),
                query="dateFrom, dateTo, count, txnType, accountNumber, paymentChannel",
                combines=[
                    "GET /zainbox/transactions/summary",
                    "GET /zainbox/transactions",
                ],
            ),
            endpoint(
                "api-payout-prep",
                "GET",
                "/api/v1/payout/prep",
                "Payout prep",
                "Network status + name enquiry before transfer.",
                composite(
                    "composite: transfer network status + name enquiry",
                    networkStatus={"status": "UP"},
                    nameEnquiry={"accountName": "ZNPAY SUNUSI INUWA"},
                ),
                query="bankCode, accountNumber",
                combines=[
                    "GET /bank/transfer/network/status",
                    "GET /bank/name-enquiry",
                ],
            ),
            endpoint(
                "api-transfer",
                "POST",
                "/api/v1/transfer",
                "Transfer and verify",
                "Funds transfer then verify by request txnRef.",
                composite(
                    "composite: funds transfer + verify",
                    transfer={"txnRef": "trf_001", "status": "pending"},
                    verify={"txnRef": "trf_001", "status": "success"},
                ),
                body=(
                    'Same as <code class="font-mono text-sm">POST /bank/transfer/v2</code>; '
                    'optional <code class="font-mono text-sm">metadata</code>.'
                ),
                combines=[
                    "POST /bank/transfer/v2",
                    "GET …/transaction/verify/{txnRef}",
                ],
            ),
            endpoint(
                "api-dva-verify",
                "GET",
                "/api/v1/dva/verify/{txnRef}",
                "DVA deposit verify (composite)",
                "Dynamic deposit status + deposit verify. Alias: /api/v1/deposit/{txnRef} (deprecated).",
                composite(
                    "composite: dynamic deposit status + deposit verify",
                    depositStatus={"txnRef": "dep_001", "paid": True},
                    depositVerify={"txnRef": "dep_001", "verified": True},
                ),
                combines=[
                    "GET /virtual-account/dynamic/deposit/status/{txnRef}",
                    "GET /virtual-account/wallet/deposit/verify/v2/{txnRef}",
                ],
            ),
            endpoint(
                "api-settlement",
                "POST",
                "/api/v1/settlement",
                "Settlement create with history",
                "Create settlement then fetch history.",
                composite(
                    "composite: settlement create + history",
                    create={
                        "zainboxCode": "charg_0atrmzL5qsMFKGYPrahj",
                        "status": "active",
                    },
                    history=[{"id": "set_1", "status": "completed"}],
                ),
                query="count, dateFrom, dateTo, status",
                body=(
                    'Same as <code class="font-mono text-sm">POST /zainbox/settlement</code>; '
                    'optional <code class="font-mono text-sm">metadata</code>.'
                ),
                combines=[
                    "POST /zainbox/settlement",
                    "GET /zainbox/settlement/history/{zainboxCode}",
                ],
            ),
        ],
    )

    sec(
        "api-v1-va",
        "API v1 · Static VA",
        [
            endpoint(
                "api-va-create",
                "POST",
                "/api/v1/va/create",
                "Create static VA",
                "Stable wrapper. Talks to Zainpay create-request.",
                zp({"accountNumber": "5006079921", "accountName": "KAMFANY LTD"}),
                note="Talks to: POST /virtual-account/create/request",
                body="Zainpay virtual-account create fields (+ optional metadata)",
            ),
            endpoint(
                "api-va-balance",
                "GET",
                "/api/v1/va/balance/{accountNumber}",
                "VA balance",
                "Stable wrapper for wallet balance.",
                zp({"accountNumber": "5006079921", "balanceAmount": "125000.00"}),
                note="Talks to: GET /virtual-account/wallet/balance/v2/{accountNumber}",
            ),
            endpoint(
                "api-va-txns",
                "GET",
                "/api/v1/va/transactions/{accountNumber}",
                "VA transactions",
                "Stable wrapper for wallet ledger.",
                zp([{"txnRef": "txn_1", "amount": "5000"}]),
                note="Talks to: GET /virtual-account/wallet/transactions/{accountNumber}",
                query="count, txnType, paymentChannel, dateFrom, dateTo",
            ),
            endpoint(
                "api-va-status",
                "PATCH",
                "/api/v1/va/status",
                "Change VA status",
                "Stable wrapper.",
                zp({"accountNumber": "5006079921", "status": "active"}),
                note="Talks to: PATCH /virtual-account/change/account/status",
            ),
            endpoint(
                "api-va-bvn",
                "POST",
                "/api/v1/va/bvn",
                "Update VA BVN",
                "Stable wrapper.",
                zp({"updated": True}),
                note="Talks to: POST /virtual-account/update/bvn",
            ),
            endpoint(
                "api-va-fund",
                "POST",
                "/api/v1/va/fund",
                "Fund static VA (sandbox)",
                "Mock-host fund helper.",
                zp({"funded": True}),
                note="Talks to: POST /virtual-account/static/fund (mock host)",
            ),
        ],
    )

    sec(
        "api-v1-dva",
        "API v1 · Dynamic VA",
        [
            endpoint(
                "api-dva-create",
                "POST",
                "/api/v1/dva/create",
                "Create dynamic VA",
                "Stable wrapper for one-time / dynamic accounts.",
                zp({"accountNumber": "9…", "txnRef": "LIVE_…"}),
                note="Talks to: POST /virtual-account/dynamic/create/request",
            ),
            endpoint(
                "api-dva-deposit",
                "GET",
                "/api/v1/dva/deposit/{txnRef}",
                "DVA deposit status",
                "Stable wrapper for dynamic deposit status.",
                zp({"txnRef": "dep_001", "paid": True}),
                note="Talks to: GET /virtual-account/dynamic/deposit/status/{txnRef}",
            ),
            endpoint(
                "api-dva-fund",
                "POST",
                "/api/v1/dva/fund",
                "Fund dynamic VA (sandbox)",
                "Mock-host fund helper.",
                zp({"funded": True}),
                note="Talks to: POST /virtual-account/dynamic/fund (mock host)",
            ),
        ],
    )

    sec(
        "api-v1-zainbox",
        "API v1 · Zainbox",
        [
            endpoint(
                "api-zainboxes-list",
                "GET",
                "/api/v1/zainboxes",
                "List zainboxes",
                "Stable wrapper.",
                zp([{"codeName": "charg_…", "name": "Store"}]),
                note="Talks to: GET /zainbox/list",
            ),
            endpoint(
                "api-zainbox-create",
                "POST",
                "/api/v1/zainbox",
                "Create zainbox",
                "Stable wrapper.",
                zp({"codeName": "charg_…", "name": "Store"}),
                note="Talks to: POST /zainbox/create/request",
            ),
            endpoint(
                "api-zainbox-update",
                "PATCH",
                "/api/v1/zainbox",
                "Update zainbox",
                "Stable wrapper.",
                zp({"codeName": "charg_…", "callbackUrl": "https://…"}),
                note="Talks to: PATCH /zainbox/update",
            ),
            endpoint(
                "api-zainbox-vas",
                "GET",
                "/api/v1/zainbox/{zainboxCode}/vas",
                "List VAs on a zainbox",
                "Stable wrapper.",
                zp([{"accountNumber": "5006079921"}]),
                note="Talks to: GET /zainbox/virtual-accounts/{zainboxCode}",
            ),
            endpoint(
                "api-zainbox-balances",
                "GET",
                "/api/v1/zainbox/{zainboxCode}/balances",
                "Account balances",
                "Stable wrapper.",
                zp([{"accountNumber": "5006079921", "balance": "125000.00"}]),
                note="Talks to: GET /zainbox/accounts/balance/{zainboxCode}",
            ),
            endpoint(
                "api-zainbox-profile",
                "GET",
                "/api/v1/zainbox/{zainboxCode}/profile",
                "Zainbox profile",
                "Uses profile v2 under the hood.",
                zp({"name": "kamfany", "codeName": "charg_…"}),
                note="Talks to: GET /zainbox/profile/v2/{zainboxCode}",
            ),
            endpoint(
                "api-zainbox-ledger",
                "GET",
                "/api/v1/zainbox/{zainboxCode}/ledger",
                "Zainbox ledger",
                "Single-box transaction list (not the composite).",
                zp([{"txnRef": "txn_1", "amount": "5000"}]),
                note="Talks to: GET /zainbox/transactions/{zainboxCode}",
            ),
            endpoint(
                "api-zainbox-subscribe",
                "POST",
                "/api/v1/zainbox/subscribe",
                "Subscribe service",
                "Stable wrapper.",
                zp({"status": "active"}),
                note="Talks to: POST /zainbox/service-subscription/subscribe",
            ),
        ],
    )

    sec(
        "api-v1-checkout",
        "API v1 · Checkout",
        [
            endpoint(
                "api-checkout-init",
                "POST",
                "/api/v1/checkout/initialize",
                "Initialize payment",
                "Stable wrapper for card/TVA session start.",
                zp({"txnRef": "ORDER_1", "sessionId": "…"}),
                note="Talks to: POST /zainbox/card/initialize/payment",
            ),
            endpoint(
                "api-checkout-inline",
                "POST",
                "/api/v1/checkout/initialize/inline",
                "Initialize inline checkout",
                "Open browser checkout (publicKey). Prefer this for koodify-inline.",
                zp({"txnRef": "LIVE_…", "accountNumber": "…"}),
                note="Talks to: POST /zainbox/card/initialize/payment/inline",
            ),
            endpoint(
                "api-checkout-account",
                "GET",
                "/api/v1/checkout/account",
                "Generate temporary VA",
                "Stable wrapper.",
                zp({"accountNumber": "…", "bankName": "GTBank"}),
                note="Talks to: GET /zainbox/card/generate-temporary-virtual-account",
                query="bankType, e",
            ),
            endpoint(
                "api-checkout-status",
                "GET",
                "/api/v1/checkout/status",
                "Temporary VA payment status",
                "Stable wrapper used by inline polling.",
                zp({"paymentStatus": "success", "txnRef": "LIVE_…"}),
                note="Talks to: GET /zainbox/card/temporary-virtual-account-payment-status",
                query="txnRef",
            ),
            endpoint(
                "api-checkout-recurring",
                "POST",
                "/api/v1/checkout/recurring",
                "Recurring purchase",
                "Stable wrapper.",
                zp({"txnRef": "card_rec_1", "status": "success"}),
                note="Talks to: POST /zainbox/card/recurring/purchase",
            ),
        ],
    )

    customer_detail = {
        **customer,
        "hasPassword": False,
        "zainbox": {
            "zainboxCode": "charg_…",
            "name": "Acme School",
            "publicKey": "pk_live_…",
            "isActive": True,
        },
        "publicKey": "pk_live_…",
        "kyc": {"id": "…", "status": "approved", "firstName": "Ada", "surname": "Lovelace"},
        "credentials": [
            {"environment": "live", "publicKey": "pk_live_…", "zainboxCodeSet": True, "status": "active"}
        ],
    }

    sec(
        "customers",
        "Customers",
        [
            endpoint(
                "customers-create",
                "POST",
                "/customers",
                "Create customer",
                "Assigns integer id and UUID uuid. Also at /api/v1/customers. Use status pending for self-register.",
                customer,
                body="displayName (required), email, mobileNumber, zainboxCode, metadata, status",
            ),
            endpoint(
                "customers-list",
                "GET",
                "/customers",
                "List customers",
                "Filter by status, or pass email for a single dashboard detail (login prep).",
                [customer],
                query="status | email",
            ),
            endpoint(
                "customers-get",
                "GET",
                "/customers/{id}",
                "Get customer dashboard",
                "Profile + zainbox + publicKey + KYC + credentials. Alias: /api/v1/customers/{id}.",
                customer_detail,
                query="revealZainboxCode",
            ),
            endpoint(
                "customers-by-uuid",
                "GET",
                "/customers/by-uuid/{uuid}",
                "Get customer by UUID",
                "Same dashboard payload as GET by id.",
                customer_detail,
                query="revealZainboxCode",
            ),
            endpoint(
                "customers-zainbox",
                "GET",
                "/customers/{id}/zainbox",
                "Customer zainbox",
                "Assigned zainbox and publishable publicKey.",
                {"customerId": 1, "zainbox": customer_detail["zainbox"], "publicKey": "pk_live_…"},
            ),
            endpoint(
                "customers-provision",
                "POST",
                "/customers/{id}/provision",
                "Provision / sync zainbox",
                "Creates Zainpay zainbox if missing, assigns publicKey, mirrors credentials. Retry after KYC approve failures.",
                {
                    "code": "00",
                    "description": "zainbox created and credentials synced",
                    "data": {
                        "customerId": 1,
                        "zainboxCode": "charg_…",
                        "publicKey": "pk_live_…",
                        "createdNewZainbox": True,
                    },
                },
            ),
            endpoint(
                "customers-update",
                "PUT",
                "/customers/{id}",
                "Update customer",
                "Replace customer fields.",
                customer,
                body="displayName, email, mobileNumber, zainboxCode, metadata, status",
            ),
            endpoint(
                "customers-patch",
                "PATCH",
                "/customers/{id}",
                "Patch customer",
                "Partial profile update for merchant UI.",
                customer,
                body="any of: displayName, email, mobileNumber, zainboxCode, metadata, status",
            ),
            endpoint(
                "customers-status",
                "PATCH",
                "/customers/{id}/status",
                "Update customer status",
                "Patch status only.",
                customer,
                body='{ "status": "active" }',
            ),
        ],
    )

    sec(
        "kyc",
        "KYC",
        [
            endpoint(
                "kyc-upsert",
                "PUT",
                "/kyc",
                "Upsert KYC",
                "Linked by customerId → customers.id. status=approved provisions zainbox + publicKey.",
                kyc,
                body="customerId, title, firstName, surname, email, mobileNumber, bvn, dob, gender, address, state, status (pending|approved|…)",
            ),
            endpoint(
                "kyc-list",
                "GET",
                "/kyc",
                "List KYC profiles",
                "Optionally filter by status.",
                [kyc],
                query="status",
            ),
            endpoint(
                "kyc-by-customer",
                "GET",
                "/kyc/by-customer/{customerId}",
                "KYC by customer",
                "Fetch KYC for a customer id.",
                kyc,
            ),
            endpoint(
                "kyc-get",
                "GET",
                "/kyc/{id}",
                "KYC by profile UUID",
                "Fetch by KYC row UUID.",
                kyc,
            ),
            endpoint(
                "kyc-status",
                "PATCH",
                "/kyc/{id}/status",
                "Update KYC status",
                "Patch status only. status=approved provisions merchant zainbox + publicKey (see provision field).",
                {**kyc, "status": "approved", "provision": {"zainboxCode": "charg_…", "publicKey": "pk_live_…", "createdNewZainbox": True}},
                body='{ "status": "active" }',
            ),
        ],
    )

    sec(
        "credentials",
        "Credentials",
        [
            endpoint(
                "credentials-upsert",
                "PUT",
                "/credentials",
                "Upsert credentials",
                "One test + one live pair per customer. Seed sets zainboxCode = Zainpay codeName (Bearer id). Redacted unless ?revealZainboxCode=true (alias: revealPrivateKey).",
                cred,
                body="customerId, environment (test|live), publicKey, zainboxCode (alias privateKey), metadata, status",
            ),
            endpoint(
                "credentials-list",
                "GET",
                "/credentials",
                "List credentials",
                "Filter by customer and environment.",
                [cred],
                query="customerId, environment, revealZainboxCode",
            ),
            endpoint(
                "credentials-by-customer",
                "GET",
                "/credentials/by-customer/{customerId}",
                "Credentials for customer",
                "Returns test + live rows.",
                [cred, {**cred, "environment": "test"}],
            ),
            endpoint(
                "credentials-by-env",
                "GET",
                "/credentials/by-customer/{customerId}/{environment}",
                "Credential by env",
                "Single environment row.",
                cred,
            ),
            endpoint(
                "credentials-get",
                "GET",
                "/credentials/{id}",
                "Credential by UUID",
                "Fetch by credential UUID.",
                cred,
            ),
            endpoint(
                "credentials-status",
                "PATCH",
                "/credentials/{id}/status",
                "Update credential status",
                "Patch status only.",
                cred,
                body='{ "status": "active" }',
            ),
        ],
    )

    sec(
        "local",
        "Local lookups",
        [
            endpoint(
                "local-va",
                "GET",
                "/virtual-accounts/{accountNumber}",
                "DB virtual account",
                "Gateway-persisted VA with metadata.",
                {
                    "id": "9f3c2a1b-0000-4000-8000-000000000001",
                    "accountNumber": "5006079921",
                    "accountName": "KAMFANY LTD",
                    "bankName": "Zainpay MFB",
                    "zainboxCode": "charg_0atrmzL5qsMFKGYPrahj",
                    "metadata": {"orderId": "ord_123"},
                },
            ),
            endpoint(
                "local-va-balance",
                "GET",
                "/virtual-accounts/{accountNumber}/balance",
                "VA balance via gateway",
                "Proxies live balance for a known account.",
                zp({"accountNumber": "5006079921", "balanceAmount": "125000.00"}),
            ),
            endpoint(
                "local-ops",
                "GET",
                "/operations/{resourceKey}",
                "Operation history",
                "Lookup by txnRef / accountNumber / zainboxCode.",
                [
                    {
                        "id": "op-1",
                        "operation": "virtual_account.create",
                        "resourceKey": "5006079921",
                        "zainpayCode": "00",
                        "metadata": {"orderId": "ord_123"},
                    }
                ],
            ),
        ],
    )

    sec(
        "zainbox",
        "Zainbox (proxy)",
        [
            endpoint(
                "zb-create",
                "POST",
                "/zainbox/create/request",
                "Create zainbox",
                "Proxies Zainpay create; optional metadata.",
                zp({"codeName": "store_abc", "name": "My Store"}),
                body="name, callbackUrl, …; optional metadata wrapper",
            ),
            endpoint(
                "zb-list",
                "GET",
                "/zainbox/list",
                "List zainboxes",
                "All merchant zainboxes.",
                zp(
                    [
                        {
                            "name": "kamfany",
                            "codeName": "charg_0atrmzL5qsMFKGYPrahj",
                            "isActive": True,
                        },
                        {
                            "name": "main",
                            "codeName": "73971_zAIbbOG148EjIaSWCIFR",
                            "isActive": True,
                        },
                    ]
                ),
                query="status",
            ),
            endpoint(
                "zb-update",
                "PATCH",
                "/zainbox/update",
                "Update zainbox",
                "Update name/callback and related fields.",
                zp({"codeName": "charg_0atrmzL5qsMFKGYPrahj", "name": "kamfany"}),
                body="codeName, name, …",
            ),
            endpoint(
                "zb-vas",
                "GET",
                "/zainbox/virtual-accounts/{zainboxCode}",
                "List VAs for zainbox",
                "Virtual accounts under a zainbox.",
                zp(
                    [
                        {
                            "accountNumber": "5006079921",
                            "accountName": "KAMFANY LTD",
                        }
                    ]
                ),
            ),
            endpoint(
                "zb-balances",
                "GET",
                "/zainbox/accounts/balance/{zainboxCode}",
                "Account balances",
                "Balances for accounts in a zainbox.",
                zp([{"accountNumber": "5006079921", "balance": "125000.00"}]),
            ),
            endpoint(
                "zb-merchant-txns",
                "GET",
                "/zainbox/transactions",
                "Merchant transactions",
                "Merchant-wide transaction list.",
                zp([{"txnRef": "txn_1", "amount": "5000"}]),
                query="count, txnType, accountNumber, paymentChannel, dateFrom, dateTo",
            ),
            endpoint(
                "zb-txns",
                "GET",
                "/zainbox/transactions/{zainboxCode}",
                "Zainbox transactions",
                "Transactions for one zainbox.",
                zp([{"txnRef": "txn_1", "amount": "5000"}]),
                query="count, txnType, accountNumber, paymentChannel, dateFrom, dateTo",
            ),
            endpoint(
                "zb-deposit-summary",
                "GET",
                "/zainbox/transfer/deposit/summary/{zainboxCode}",
                "Deposit summary",
                "Deposit aggregates for a date window.",
                zp({"totalDeposits": "150000", "count": 12}),
                query="dateFrom, dateTo",
            ),
            endpoint(
                "zb-profile",
                "GET",
                "/zainbox/profile/{zainboxCode}",
                "Profile",
                "Zainbox profile.",
                zp(
                    {
                        "name": "kamfany",
                        "codeName": "charg_0atrmzL5qsMFKGYPrahj",
                    }
                ),
            ),
            endpoint(
                "zb-profile-v2",
                "GET",
                "/zainbox/profile/v2/{zainboxCode}",
                "Profile v2",
                "Expanded profile payload.",
                zp({"name": "kamfany", "billing": {"feeBearer": "merchant"}}),
            ),
            endpoint(
                "zb-billing",
                "GET",
                "/zainbox/billing/config/{zainboxCode}",
                "Billing config",
                "Billing configuration for a zainbox.",
                zp({"feeBearer": "merchant"}),
            ),
            endpoint(
                "zb-txn-summary",
                "GET",
                "/zainbox/transactions/summary",
                "Transactions summary",
                "Merchant-wide totals.",
                zp({"totalCredit": "1000000", "totalDebit": "250000"}),
                query="dateFrom, dateTo",
            ),
            endpoint(
                "zb-card-txns",
                "GET",
                "/zainbox/card/transactions/{zainboxCode}",
                "Card transactions",
                "Card payments for a zainbox.",
                zp([{"txnRef": "card_1", "amount": "2000"}]),
                query="count, dateFrom, dateTo, status, email, txnRef",
            ),
            endpoint(
                "zb-subscribe",
                "POST",
                "/zainbox/service-subscription/subscribe",
                "Subscribe",
                "Subscribe a zainbox to a service.",
                zp({"subscribed": True}),
                body="zainboxCode, serviceName",
            ),
            endpoint(
                "zb-unsubscribe",
                "POST",
                "/zainbox/service-subscription/unsubscribe",
                "Unsubscribe",
                "Remove a service subscription.",
                zp({"subscribed": False}),
                body="zainboxCode, serviceName",
            ),
            endpoint(
                "zb-subs-all",
                "GET",
                "/zainbox/service-subscription/subscriptions/all/{zainboxCode}",
                "All subscriptions",
                "All subscriptions for a zainbox.",
                zp([{"serviceName": "virtualAccount", "status": "active"}]),
                query="serviceName",
            ),
            endpoint(
                "zb-subs-active",
                "GET",
                "/zainbox/service-subscription/subscriptions/active/{zainboxCode}",
                "Active subscriptions",
                "Active-only subscriptions.",
                zp([{"serviceName": "virtualAccount", "status": "active"}]),
            ),
        ],
    )

    sec(
        "virtual-account",
        "Virtual accounts (proxy)",
        [
            endpoint(
                "va-create",
                "POST",
                "/virtual-account/create/request",
                "Create static VA",
                "Create a static virtual account.",
                zp(
                    {
                        "accountNumber": "5006079921",
                        "accountName": "MUHAMMAD ABDULLAHI",
                        "bankName": "Zainpay MFB",
                    },
                    "Virtual account created",
                ),
                body="KYC fields + zainboxCode; optional metadata",
            ),
            endpoint(
                "va-dynamic",
                "POST",
                "/virtual-account/dynamic/create/request",
                "Create dynamic VA",
                "Create a dynamic virtual account.",
                zp({"accountNumber": "9001112223", "txnRef": "dyn_001"}),
                body="Zainpay dynamic VA fields",
            ),
            endpoint(
                "va-balance",
                "GET",
                "/virtual-account/wallet/balance/v2/{accountNumber}",
                "VA balance",
                "Live balance path.",
                zp({"accountNumber": "5006079921", "balanceAmount": "125000.00"}),
            ),
            endpoint(
                "va-balance-alias",
                "GET",
                "/zainbox/virtual-account/wallet/balance/v2/{accountNumber}",
                "VA balance (legacy alias)",
                "Same handler as the canonical balance path.",
                zp({"accountNumber": "5006079921", "balanceAmount": "125000.00"}),
            ),
            endpoint(
                "va-status",
                "PATCH",
                "/virtual-account/change/account/status",
                "Change VA status",
                "Enable/disable a virtual account.",
                zp({"accountNumber": "5006079921", "status": True}),
                body="accountNumber, zainboxCode, status",
            ),
            endpoint(
                "va-bvn",
                "POST",
                "/virtual-account/update/bvn",
                "Update VA BVN",
                "Update BVN on an account.",
                zp({"accountNumber": "5006079921", "updated": True}),
                body="accountNumber, zainboxCode, bvn",
            ),
            endpoint(
                "va-txns",
                "GET",
                "/virtual-account/wallet/transactions/{accountNumber}",
                "VA transactions",
                "Transactions for one account.",
                zp([{"txnRef": "txn_1", "amount": "5000"}]),
                query="count, txnType, paymentChannel, dateFrom, dateTo",
            ),
            endpoint(
                "va-dyn-status",
                "GET",
                "/virtual-account/dynamic/deposit/status/{txnRef}",
                "Dynamic deposit status",
                "Poll dynamic deposit payment status.",
                zp({"txnRef": "dep_001", "paid": True}),
            ),
            endpoint(
                "va-fund-static",
                "POST",
                "/virtual-account/static/fund",
                "Sandbox fund static VA",
                "Mock host funding helper.",
                zp({"funded": True}),
                note="Sandbox / mock host only.",
            ),
            endpoint(
                "va-fund-dynamic",
                "POST",
                "/virtual-account/dynamic/fund",
                "Sandbox fund dynamic VA",
                "Mock host funding helper.",
                zp({"funded": True}),
                note="Sandbox / mock host only.",
            ),
        ],
    )

    bank_list = zp(
        [
            {"code": "000013", "name": "GTBank"},
            {"code": "000014", "name": "Access Bank"},
        ]
    )
    name_enq = zp(
        {
            "accountName": "ZNPAY SUNUSI INUWA",
            "accountNumber": "5006079921",
            "bankCode": "000013",
        }
    )
    transfer_plain = zp({"txnRef": "trf_001", "status": "pending"})

    sec(
        "bank",
        "Bank",
        [
            endpoint(
                "bank-list",
                "GET",
                "/bank/list",
                "Bank list",
                "Canonical live path.",
                bank_list,
                note="Prefer this over the legacy alias.",
            ),
            endpoint(
                "bank-name",
                "GET",
                "/bank/name-enquiry",
                "Name enquiry",
                "Resolve account name.",
                name_enq,
                query="bankCode, accountNumber",
            ),
            endpoint(
                "bank-list-alias",
                "GET",
                "/zainbox/bank/list",
                "Bank list (legacy alias)",
                "Same handler as /bank/list.",
                bank_list,
                note="Live /zainbox/bank/list returns 405.",
            ),
            endpoint(
                "bank-name-alias",
                "GET",
                "/zainbox/bank/name-enquiry",
                "Name enquiry (legacy alias)",
                "Same handler as /bank/name-enquiry.",
                name_enq,
            ),
        ],
    )

    sec(
        "transfers",
        "Transfers",
        [
            endpoint(
                "trf-create",
                "POST",
                "/bank/transfer/v2",
                "Funds transfer",
                "Initiate a bank transfer.",
                transfer_plain,
                body="destinationAccountNumber, destinationBankCode, sourceAccountNumber, sourceBankCode, amount, txnRef, zainboxCode",
            ),
            endpoint(
                "trf-alias",
                "POST",
                "/zainbox/bank/transfer/v2",
                "Funds transfer (legacy alias)",
                "Same handler as /bank/transfer/v2.",
                transfer_plain,
            ),
            endpoint(
                "trf-verify",
                "GET",
                "/virtual-account/wallet/transaction/verify/{txnRef}",
                "Verify transaction",
                "Verify transfer/wallet transaction.",
                zp({"txnRef": "trf_001", "status": "success", "amount": "5000"}),
            ),
            endpoint(
                "trf-network",
                "GET",
                "/bank/transfer/network/status",
                "Network status",
                "Transfer rail availability.",
                zp({"status": "UP"}),
            ),
        ],
    )

    settlement = zp(
        {
            "zainboxCode": "charg_0atrmzL5qsMFKGYPrahj",
            "scheduleType": "auto",
            "status": "active",
        }
    )
    sec(
        "settlements",
        "Settlements",
        [
            endpoint(
                "set-create",
                "POST",
                "/zainbox/settlement",
                "Create settlement",
                "Create a settlement schedule.",
                settlement,
                body="name, scheduleType, schedulePeriod, zainboxCode, status, settlementAccountList",
            ),
            endpoint(
                "set-get",
                "GET",
                "/zainbox/settlement",
                "Get settlement config",
                "Fetch settlement configuration.",
                settlement,
                query="zainboxCode",
                note="Live may return HTTP 405.",
            ),
            endpoint(
                "set-history",
                "GET",
                "/zainbox/settlement/history/{zainboxCode}",
                "Settlement history",
                "Historical settlement runs.",
                zp([{"id": "set_1", "amount": "100000", "status": "completed"}]),
                query="count, dateFrom, dateTo, status",
            ),
            endpoint(
                "set-internal",
                "GET",
                "/zainbox/internal-settlement-account/{zainboxCode}",
                "Internal settlement account",
                "Internal payout wallet for a zainbox.",
                zp(
                    {
                        "accountNumber": "5006079921",
                        "accountName": "INTERNAL SETTLEMENT",
                    }
                ),
            ),
        ],
    )

    sec(
        "reconciliation",
        "Reconciliation",
        [
            endpoint(
                "rec-deposit",
                "GET",
                "/virtual-account/wallet/deposit/verify/v2/{txnRef}",
                "Verify deposit",
                "Verify a deposit by txnRef.",
                zp({"txnRef": "dep_001", "verified": True}),
            ),
            endpoint(
                "rec-bank",
                "GET",
                "/virtual-account/wallet/transaction/reconcile/bank-deposit",
                "Reconcile bank deposit",
                "Reconcile a bank deposit.",
                zp({"reconciled": True}),
                query="bankType, verificationType, accountNumber, sessionId",
            ),
            endpoint(
                "rec-card",
                "GET",
                "/virtual-account/wallet/transaction/reconcile/card-payment",
                "Reconcile card payment",
                "Reconcile a card payment.",
                zp({"reconciled": True}),
                query="txnRef and related Zainpay params",
            ),
        ],
    )

    sec(
        "cards",
        "Cards",
        [
            endpoint(
                "card-init",
                "POST",
                "/zainbox/card/initialize/payment",
                "Initialize card payment",
                "Start a card checkout.",
                zp("https://checkout.example/pay/abc123"),
                body="Zainpay card init fields; optional metadata",
            ),
            endpoint(
                "card-recurring",
                "POST",
                "/zainbox/card/recurring/purchase",
                "Recurring card purchase",
                "Charge a saved card.",
                zp({"txnRef": "card_rec_1", "status": "success"}),
                body="Zainpay recurring fields",
            ),
        ],
    )

    repush = zp({"txnRef": "dep_001", "repushed": True})
    merchant_webhook_body = {
        "event": "payment.success",
        "status": "success",
        "reference": "ORDER_1793402668162",
        "description": "Payment was successful",
        "amount": "500",
        "currencyCode": "NGN",
        "zainboxCode": "main_box",
        "payer": {
            "email": "customer@example.com",
            "phone": "08012345678",
            "name": "Ada Lovelace",
        },
        "customParam": {
            "matric": "ND/12/2211",
            "session": "2026",
            "paymentType": "School Fees",
            "studentId": "STU001",
        },
        "paymentSplit": {
            "txnRef": "ORDER_1793402668162",
            "primaryZainboxCode": "main_box",
            "checkoutAmount": "500",
            "totalKobo": 50000,
            "status": "success",
            "sourceAccount": "0123456789",
            "sourceBankCode": "058",
            "error": None,
            "legs": [],
        },
    }
    sec(
        "webhooks",
        "Webhooks",
        [
            endpoint(
                "wh-inbound",
                "POST",
                "/webhooks/zainpay",
                "Inbound Zainpay webhook",
                "Receive Zainpay webhook events.",
                {"received": True},
                note="HMAC Zainpay-Signature via ZAINPAY_WEBHOOK_SECRET. No X-Api-Key.",
            ),
            endpoint(
                "wh-merchant-outbound",
                "POST",
                "{callBackUrl}",
                "Outbound merchant webhook",
                "On payment success Koodify POSTs a signed payment.success JSON body to checkout callBackUrl (or zainbox callback_url). Delivered once per txnRef — independent of the browser JS callback.",
                merchant_webhook_body,
                note=(
                    "Headers: Content-Type application/json, User-Agent Koodify-Webhook/1.0, "
                    "X-Koodify-Event payment.success, X-Koodify-Signature hex HMAC-SHA256 "
                    "(MERCHANT_WEBHOOK_SECRET or GATEWAY_API_KEY). Directory URLs get a trailing slash. "
                    "See docs/inline-checkout.html#merchant-webhook."
                ),
                sample_label="Sample POST body",
            ),
            endpoint(
                "wh-repush-dep",
                "GET",
                "/zainbox/repush/deposit/{txnRef}",
                "Repush deposit",
                "Ask Zainpay to repush a deposit webhook.",
                repush,
            ),
            endpoint(
                "wh-repush-card",
                "GET",
                "/zainbox/repush/card-payment/{txnRef}",
                "Repush card payment",
                "Ask Zainpay to repush a card-payment webhook.",
                repush,
            ),
        ],
    )

    sidebar_parts = [
        '<a data-nav class="block rounded-md px-3 py-2 text-sm text-ink-700 hover:bg-white hover:text-accent" href="#auth">Authentication</a>',
        '<a data-nav class="block rounded-md px-3 py-2 text-sm text-ink-700 hover:bg-white hover:text-accent" href="#metadata">Metadata</a>',
    ]
    for s in sections:
        sidebar_parts.append(
            f'<a data-nav class="mt-3 block rounded-md px-3 py-2 text-sm font-semibold text-ink-950 hover:bg-white hover:text-accent" href="#{s["id"]}">{s["title"]}</a>'
        )
        for ep in s["endpoints"]:
            sidebar_parts.append(
                f'<a data-nav class="ml-2 block truncate rounded-md px-3 py-1.5 font-mono text-[11px] text-ink-500 hover:bg-white hover:text-accent" href="#{ep["id"]}" title="{ep["method"]} {ep["path"]}">{ep["method"]} {ep["path"]}</a>'
            )

    main_parts = []
    for s in sections:
        body = "".join(ep["html"] for ep in s["endpoints"])
        main_parts.append(
            f'<section id="{s["id"]}" class="scroll-mt-24 border-b border-slate-200 py-12">'
            f'<h2 class="text-4xl font-semibold tracking-tight text-ink-950">{s["title"]}</h2>'
            f'<div class="mt-2">{body}</div></section>'
        )

    html = f"""<!DOCTYPE html>
<html lang="en" class="scroll-smooth">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Koodify API Reference</title>
  <script src="https://cdn.tailwindcss.com"></script>
  <script>
    tailwind.config = {{
      theme: {{
        extend: {{
          fontFamily: {{
            sans: ['"IBM Plex Sans"', 'ui-sans-serif', 'system-ui', 'sans-serif'],
            mono: ['"IBM Plex Mono"', 'ui-monospace', 'monospace'],
          }},
          colors: {{
            ink: {{ 950: '#0c1222', 900: '#121a2f', 700: '#2a3655', 500: '#5b6b8c' }},
            accent: {{ DEFAULT: '#0f766e', soft: '#ccfbf1' }},
          }},
        }},
      }},
    }};
  </script>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
  <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500&family=IBM+Plex+Sans:wght@400;500;600;700&display=swap" rel="stylesheet" />
  <style>
    [data-nav].is-active {{ background: #fff; color: #0f766e; font-weight: 600; }}
    #sidebar-nav {{ scrollbar-width: thin; }}
  </style>
</head>
<body class="bg-[radial-gradient(ellipse_at_top,_#e8f5f3_0%,_#f8fafc_45%,_#f1f5f9_100%)] text-ink-900 font-sans antialiased">
  <div class="min-h-screen lg:grid lg:grid-cols-[280px_minmax(0,1fr)]">
    <aside class="border-b border-slate-200 bg-white/90 lg:sticky lg:top-0 lg:h-screen lg:overflow-hidden lg:border-b-0 lg:border-r">
      <div class="border-b border-slate-200 px-5 py-5">
        <p class="text-xs font-semibold uppercase tracking-[0.18em] text-accent">Koodify</p>
        <p class="mt-1 text-lg font-semibold text-ink-950">API reference</p>
        <p class="mt-1 text-xs text-ink-500">baseURL + path below</p>
      </div>
      <nav id="sidebar-nav" class="max-h-[50vh] space-y-0.5 overflow-y-auto px-3 py-4 lg:max-h-[calc(100vh-7.5rem)]">
        {''.join(sidebar_parts)}
      </nav>
    </aside>
    <div>
      <header class="border-b border-slate-200/80 bg-white/70 backdrop-blur">
        <div class="mx-auto max-w-5xl px-6 py-12">
          <h1 class="text-5xl font-bold tracking-tight text-ink-950 sm:text-6xl">Gateway endpoints</h1>
          <p class="mt-5 max-w-3xl text-lg leading-8 text-ink-700">
            Plain <code class="rounded bg-white px-1.5 py-0.5 font-mono text-[0.9em] shadow-sm">/api/v1</code> convenience routes,
            local customer APIs, and Zainpay proxies. Each endpoint includes a sample response.
          </p>
          <p class="mt-3 text-sm text-ink-500">Composite deep-dive: <a class="text-accent underline" href="composite-endpoints.html">composite-endpoints.html</a>. Inline checkout: <a class="text-accent underline" href="inline-checkout.html">inline-checkout.html</a>.</p>
        </div>
      </header>
      <main class="mx-auto max-w-5xl px-6 py-12">
        <section id="auth" class="scroll-mt-24 border-b border-slate-200 pb-12">
          <h2 class="text-4xl font-semibold tracking-tight text-ink-950">Authentication</h2>
          <div class="mt-6 grid gap-8 xl:grid-cols-2">
            <ul class="space-y-3 text-base leading-7 text-ink-700">
              <li>Most routes: <code class="font-mono text-sm">X-Api-Key: $GATEWAY_API_KEY</code></li>
              <li>Upstream Zainpay: server <code class="font-mono text-sm">ZAINPAY_API_KEY</code> (Bearer)</li>
              <li><code class="font-mono text-sm">POST /webhooks/zainpay</code>: HMAC <code class="font-mono text-sm">Zainpay-Signature</code></li>
              <li>Outbound merchant <code class="font-mono text-sm">callBackUrl</code>: HMAC <code class="font-mono text-sm">X-Koodify-Signature</code></li>
              <li><code class="font-mono text-sm">GET /health</code> is open</li>
            </ul>
            <div>
              <p class="text-xs font-semibold uppercase tracking-[0.16em] text-ink-500">Sample request</p>
              <pre class="mt-3 overflow-x-auto rounded-lg bg-ink-950 p-4 font-mono text-xs text-slate-100"><code>curl -H "X-Api-Key: $GATEWAY_API_KEY" \\
  "$HOST/api/v1/zainbox/charg_0atrmzL5qsMFKGYPrahj"</code></pre>
            </div>
          </div>
        </section>
        <section id="metadata" class="scroll-mt-24 border-b border-slate-200 py-12">
          <h2 class="text-4xl font-semibold tracking-tight text-ink-950">Metadata on mutations</h2>
          <div class="mt-6 grid gap-8 xl:grid-cols-2">
            <p class="text-base leading-7 text-ink-700">
              Mutating Zainpay proxies accept custom metadata that is persisted locally and never forwarded upstream.
              Successful responses include <code class="font-mono text-sm">data.metadata</code>.
            </p>
            <div>
              <p class="text-xs font-semibold uppercase tracking-[0.16em] text-ink-500">Sample body</p>
              <pre class="mt-3 overflow-x-auto rounded-lg bg-ink-950 p-4 font-mono text-xs text-slate-100"><code>{{
  "zainpay": {{ "...": "official Zainpay fields" }},
  "metadata": {{ "orderId": "ord_123" }}
}}</code></pre>
            </div>
          </div>
        </section>
        {''.join(main_parts)}
        <section class="py-12">
          <h2 class="text-4xl font-semibold tracking-tight text-ink-950">Base URL &amp; env</h2>
          <ul class="mt-6 space-y-3 text-base leading-7 text-ink-700">
            <li>Bind: <code class="font-mono text-sm">GATEWAY_BIND</code> (default <code class="font-mono text-sm">0.0.0.0:8080</code>)</li>
            <li>Gateway key: <code class="font-mono text-sm">GATEWAY_API_KEY</code></li>
            <li>Database: <code class="font-mono text-sm">DATABASE_URL</code></li>
            <li>Zainpay: <code class="font-mono text-sm">ZAINPAY_BASE_URL</code>, <code class="font-mono text-sm">ZAINPAY_API_KEY</code>, <code class="font-mono text-sm">ZAINPAY_WEBHOOK_SECRET</code></li>
          </ul>
        </section>
      </main>
    </div>
  </div>
  <script>
    (function () {{
      const links = [...document.querySelectorAll("[data-nav]")];
      const map = links.map((a) => {{
        const id = a.getAttribute("href").slice(1);
        return {{ a, el: document.getElementById(id) }};
      }}).filter((x) => x.el);
      const io = new IntersectionObserver((entries) => {{
        const visible = entries
          .filter((e) => e.isIntersecting)
          .sort((a, b) => b.intersectionRatio - a.intersectionRatio)[0];
        if (!visible) return;
        const id = visible.target.id;
        links.forEach((a) => a.classList.toggle("is-active", a.getAttribute("href") === "#" + id));
        const active = links.find((a) => a.getAttribute("href") === "#" + id);
        if (active) active.scrollIntoView({{ block: "nearest" }});
      }}, {{ rootMargin: "-20% 0px -65% 0px", threshold: [0, 0.2, 0.6] }});
      map.forEach(({{ el }}) => io.observe(el));
    }})();
  </script>
</body>
</html>
"""

    out = Path(__file__).with_name("api.html")
    out.write_text(html)
    print(f"wrote {out} ({out.stat().st_size} bytes)")
    print("endpoints", sum(len(s["endpoints"]) for s in sections))
    print("sample panels", html.count("Sample response"))


if __name__ == "__main__":
    main()
