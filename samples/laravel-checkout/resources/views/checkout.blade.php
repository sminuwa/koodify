@extends('layouts.sample')

@section('title', 'Koodify · Inline checkout demo')

@section('content')
    <div class="card">
        <h1>Pay with bank transfer</h1>
        <p class="lede">
            Loads <code>koodify-inline.js</code>, opens a temporary virtual account, and posts
            <code>payment.success</code> to your webhook when the transfer clears.
        </p>

        @unless($configured)
            <div class="banner">
                Set <code>KOODIFY_PUBLIC_KEY</code> in <code>.env</code> (a <code>pk_live_…</code> or <code>pk_test_…</code> key), then refresh.
            </div>
        @endunless

        <form id="checkout-form" autocomplete="on">
            <div class="grid">
                <div class="field">
                    <label for="name">Customer name</label>
                    <input id="name" name="name" value="Ada Lovelace" required>
                </div>
                <div class="field">
                    <label for="amount">Amount (kobo / minor units as string)</label>
                    <input id="amount" name="amount" value="{{ $demoAmount }}" required>
                </div>
            </div>
            <div class="grid">
                <div class="field">
                    <label for="email">Email</label>
                    <input id="email" name="email" type="email" value="parent@example.com" required>
                </div>
                <div class="field">
                    <label for="phone">Mobile</label>
                    <input id="phone" name="phone" value="08012345678" required>
                </div>
            </div>
            <div class="field">
                <label for="matric">Custom param · matric</label>
                <input id="matric" name="matric" value="ND/12/2211">
            </div>

            <div class="actions">
                <button type="submit" id="pay-btn" @disabled(! $configured)>Pay with Koodify</button>
                <span class="hint">txnRef is generated per attempt</span>
            </div>
        </form>

        <div id="status" class="status" aria-live="polite">Waiting for payment…</div>

        <p class="meta">
            Webhook: <code>{{ $callbackUrl }}</code><br>
            Must be publicly reachable for Koodify to deliver events (use a tunnel locally).
        </p>
    </div>
@endsection

@push('scripts')
    <script src="{{ $inlineJs }}"></script>
    <script>
        const publicKey = @json($publicKey);
        const callBackUrl = @json($callbackUrl);
        const form = document.getElementById('checkout-form');
        const statusEl = document.getElementById('status');
        const payBtn = document.getElementById('pay-btn');

        function setStatus(message, kind) {
            statusEl.textContent = typeof message === 'string' ? message : JSON.stringify(message, null, 2);
            statusEl.className = 'status' + (kind ? ' ' + kind : '');
        }

        form.addEventListener('submit', function (event) {
            event.preventDefault();
            if (!publicKey) {
                setStatus('Missing KOODIFY_PUBLIC_KEY', 'err');
                return;
            }
            if (typeof koodifyInitPayment !== 'function') {
                setStatus('koodify-inline.js did not load', 'err');
                return;
            }

            const txnRef = 'LARAVEL_' + Date.now();
            const payload = {
                publicKey: publicKey,
                amount: String(document.getElementById('amount').value).trim(),
                txnRef: txnRef,
                mobileNumber: document.getElementById('phone').value.trim(),
                emailAddress: document.getElementById('email').value.trim(),
                callBackUrl: callBackUrl,
                name: document.getElementById('name').value.trim(),
                customParam: {
                    matric: document.getElementById('matric').value.trim(),
                    session: '2026',
                    source: 'laravel-checkout-sample'
                }
            };

            payBtn.disabled = true;
            setStatus('Opening checkout for ' + txnRef + '…');

            koodifyInitPayment(payload, function (response) {
                payBtn.disabled = false;
                const ok = response && (response.status === 'success' || response.status === 'paid');
                setStatus(response || { error: 'empty callback' }, ok ? 'ok' : 'err');
            });
        });
    </script>
@endpush
