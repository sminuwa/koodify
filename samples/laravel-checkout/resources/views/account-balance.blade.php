@extends('layouts.sample')

@section('title', 'Koodify · Account balance')

@section('content')
    <div class="card">
        <h1>Check virtual account balance</h1>
        <p class="lede">
            Server call to <code>GET /api/v1/va/balance/{accountNumber}</code> with your
            <code>sk_…</code> key. Use an account number from static VA create (or any VA
            belonging to that merchant zainbox).
        </p>

        @unless($configured)
            <div class="banner">
                Set <code>KOODIFY_SECRET_KEY</code> in <code>.env</code> (<code>sk_live_…</code> or <code>sk_test_…</code>), then refresh.
            </div>
        @endunless

        @if ($error)
            <div class="banner err">{{ $error }}</div>
        @endif

        <form method="post" action="{{ route('account-balance.lookup') }}">
            @csrf

            <div class="field">
                <label for="accountNumber">Account number</label>
                <input
                    id="accountNumber"
                    name="accountNumber"
                    inputmode="numeric"
                    autocomplete="off"
                    placeholder="e.g. 5006079921"
                    value="{{ old('accountNumber', $accountNumber) }}"
                    required
                >
            </div>

            <div class="actions">
                <button type="submit" @disabled(! $configured)>Check balance</button>
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
            API: <code>{{ rtrim(config('koodify.api_base'), '/') }}/api/v1/va/balance/{accountNumber}</code><br>
            Balance amounts are typically returned in kobo / minor units (see <code>balance</code> or <code>balanceAmount</code>).
        </p>
    </div>
@endsection
