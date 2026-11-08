@extends('layouts.merchant')

@section('title', 'Dynamic VA · Koodify')
@section('crumb', 'Dynamic VA')

@section('content')
    <div class="surface merchant-panel card">
        <h1>Create a dynamic virtual account</h1>
        <p class="lede">
            One-time account for a fixed amount via <code>POST /api/v1/dva/create</code>.
            Koodify resolves your zainbox from <code>sk_…</code>, picks a bank, and sets
            <code>accountName</code> to Zainpay’s fixed <code>Zainpay Checkout</code>.
        </p>

        @unless($configured)
            <div class="banner">
                Set <code>KOODIFY_SECRET_KEY</code> in <code>.env</code> (<code>sk_live_…</code> or <code>sk_test_…</code>), then refresh.
            </div>
        @endunless

        @if ($error)
            <div class="banner err">{{ $error }}</div>
        @endif

        <form method="post" action="{{ route('dynamic-virtual-accounts.store') }}">
            @csrf

            <div class="field">
                <label for="email">Email</label>
                <input id="email" name="email" type="email" value="{{ old('email', $defaults['email']) }}" required>
            </div>

            <div class="grid">
                <div class="field">
                    <label for="amount">Amount (kobo)</label>
                    <input id="amount" name="amount" value="{{ old('amount', $defaults['amount']) }}" required>
                    <p class="hint" style="margin:.35rem 0 0">e.g. <code>50000</code> = ₦500.00</p>
                </div>
                <div class="field">
                    <label for="duration">Duration (seconds)</label>
                    <input id="duration" name="duration" type="number" min="300" max="259200" value="{{ old('duration', $defaults['duration']) }}" required>
                    <p class="hint" style="margin:.35rem 0 0">300–259200 (5 min – 72 h)</p>
                </div>
            </div>

            <div class="field">
                <label for="txnRef">Transaction reference</label>
                <input id="txnRef" name="txnRef" value="{{ old('txnRef', $defaults['txnRef']) }}">
                <p class="hint" style="margin:.35rem 0 0">Leave blank to auto-generate</p>
            </div>

            <div class="field">
                <label for="callBackUrl">Callback URL</label>
                <input id="callBackUrl" name="callBackUrl" type="url" value="{{ old('callBackUrl', $defaults['callBackUrl']) }}">
                <p class="hint" style="margin:.35rem 0 0">Defaults to your sample webhook. Use a public HTTPS URL for live delivery.</p>
            </div>

            <div class="actions">
                <button type="submit" @disabled(! $configured)>Create dynamic VA</button>
                <span class="hint">Uses <code>X-Api-Key: sk_…</code> on the server only</span>
            </div>
        </form>

        @if ($result)
            @php
                $ok = ($result['code'] ?? null) === '00' || ($result['status'] ?? '') === '200 OK';
            @endphp
            <div class="status {{ $ok ? 'ok' : 'err' }}" aria-live="polite">{{ json_encode($result, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES) }}</div>
        @else
            <div class="status" aria-live="polite">API response will appear here…</div>
        @endif

        <p class="meta">
            API: <code>{{ rtrim(config('koodify.api_base'), '/') }}/api/v1/dva/create</code><br>
            Temporary account — pay the exact amount before it expires (<code>timeToLive</code>).
        </p>
    </div>
@endsection
