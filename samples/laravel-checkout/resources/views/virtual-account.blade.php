@extends('layouts.sample')

@section('title', 'Koodify · Create virtual account')

@section('content')
    <div class="card">
        <h1>Create a static virtual account</h1>
        <p class="lede">
            Server call to <code>POST /api/v1/va/create</code> with your
            <code>sk_…</code> key. Koodify resolves your zainbox from the secret,
            picks a bank via Adaptive Bank/DVA Routing, and fills BVN / DOB / gender
            defaults — you do not send those.
        </p>

        @unless($configured)
            <div class="banner">
                Set <code>KOODIFY_SECRET_KEY</code> in <code>.env</code> (<code>sk_live_…</code> or <code>sk_test_…</code>), then refresh.
            </div>
        @endunless

        @if ($error)
            <div class="banner err">{{ $error }}</div>
        @endif

        <form method="post" action="{{ route('virtual-accounts.store') }}">
            @csrf

            <div class="field">
                <label for="title">Title</label>
                <input id="title" name="title" value="{{ old('title', $defaults['title']) }}" required>
            </div>

            <div class="grid">
                <div class="field">
                    <label for="firstName">First name</label>
                    <input id="firstName" name="firstName" value="{{ old('firstName', $defaults['firstName']) }}" required>
                </div>
                <div class="field">
                    <label for="surname">Surname</label>
                    <input id="surname" name="surname" value="{{ old('surname', $defaults['surname']) }}" required>
                </div>
            </div>

            <div class="grid">
                <div class="field">
                    <label for="email">Email</label>
                    <input id="email" name="email" type="email" value="{{ old('email', $defaults['email']) }}" required>
                </div>
                <div class="field">
                    <label for="mobileNumber">Mobile</label>
                    <input id="mobileNumber" name="mobileNumber" value="{{ old('mobileNumber', $defaults['mobileNumber']) }}" required>
                </div>
            </div>

            <div class="field">
                <label for="address">Address</label>
                <input id="address" name="address" value="{{ old('address', $defaults['address']) }}" required>
            </div>

            <div class="field">
                <label for="state">State</label>
                <input id="state" name="state" value="{{ old('state', $defaults['state']) }}" required>
            </div>

            <div class="actions">
                <button type="submit" @disabled(! $configured)>Create virtual account</button>
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
            API: <code>{{ rtrim(config('koodify.api_base'), '/') }}/api/v1/va/create</code><br>
            BVN, DOB, and gender are applied by Koodify when omitted (<code>VA_DEFAULT_*</code>).
        </p>
    </div>
@endsection
