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
      <p class="text-xs font-semibold uppercase tracking-[0.16em] text-ink-500">Sample response</p>
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
        "publicKey": "eyJ0eXAiOiJKV1QiLCJhbGciOi…",
        "privateKey": "charg_0atrmzL5qsMFKGYPrahj",
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
                "api-zainbox-activity",
                "GET",
                "/api/v1/zainbox/{zainboxCode}/activity",
                "Zainbox activity",
                "Transactions + deposit summary.",
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
                "/api/v1/virtual-account/{accountNumber}",
                "Virtual-account wallet",
                "Balance + transactions for one VA.",
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
                "/api/v1/bank/resolve",
                "Name resolve",
                "Bank list + name enquiry.",
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
                "api-merchant-activity",
                "GET",
                "/api/v1/merchant/activity",
                "Merchant activity",
                "Merchant-wide summary + transaction list.",
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
                "api-deposit",
                "GET",
                "/api/v1/deposit/{txnRef}",
                "Dynamic deposit check",
                "Dynamic deposit status + deposit verify.",
                composite(
                    "composite: dynamic deposit status + deposit verify",
                    depositStatus={"txnRef": "dep_001", "paid": True},
                    depositVerify={"txnRef": "dep_001", "verified": True},
                ),
                combines=[
                    "GET …/dynamic/deposit/status/{txnRef}",
                    "GET …/deposit/verify/v2/{txnRef}",
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
        "customers",
        "Customers",
        [
            endpoint(
                "customers-create",
                "POST",
                "/customers",
                "Create customer",
                "Assigns integer id and UUID uuid.",
                customer,
                body="displayName (required), email, mobileNumber, zainboxCode, metadata, status",
            ),
            endpoint(
                "customers-list",
                "GET",
                "/customers",
                "List customers",
                "Optionally filter by status.",
                [customer],
                query="status",
            ),
            endpoint(
                "customers-get",
                "GET",
                "/customers/{id}",
                "Get customer",
                "Fetch by integer id.",
                customer,
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
                "Linked by customerId → customers.id.",
                kyc,
                body="customerId, title, firstName, surname, email, mobileNumber, bvn, dob, gender, address, state, …",
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
                "Patch status only.",
                kyc,
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
                "One test + one live pair per customer. Seed sets privateKey = zainbox codeName.",
                cred,
                body="customerId, environment (test|live), publicKey, privateKey, metadata, status",
            ),
            endpoint(
                "credentials-list",
                "GET",
                "/credentials",
                "List credentials",
                "Filter by customer and environment.",
                [cred],
                query="customerId, environment",
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
