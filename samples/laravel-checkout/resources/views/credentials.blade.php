@extends('layouts.merchant')

@section('title', 'Credentials · Koodify')
@section('crumb', 'Credentials')

@section('content')
    <div class="surface merchant-panel card stack">
        <div>
            <h1>API credentials</h1>
            <p class="lede">Use the publishable key in checkout. Keep the secret key on your server only — it is shown once when issued or rotated.</p>
        </div>

        <div class="keyrow">
            <div>
                <div class="hint">Publishable key (browser)</div>
                <code>{{ $publicKey ?: 'Not provisioned yet — complete KYC first.' }}</code>
            </div>
        </div>

        @forelse ($credentials as $cred)
            <div class="keyrow">
                <div>
                    <div class="hint">{{ strtoupper($cred['environment'] ?? '') }} · {{ $cred['status'] ?? '' }}</div>
                    <div><code>{{ $cred['publicKey'] ?? '—' }}</code></div>
                    <div class="hint">Secret prefix: <code>{{ $cred['secretKeyPrefix'] ?? '—' }}</code>
                        @if (!empty($cred['secretKeySet']))
                            · set
                        @endif
                    </div>
                </div>
            </div>
        @empty
            <div class="banner">No credential rows yet. Approve KYC to provision.</div>
        @endforelse

        @if ($secretKey)
            <div class="banner ok">
                Session secret (use for VA samples): <code class="mono">{{ $secretKey }}</code>
            </div>
        @endif
        @if ($secretLive)
            <div class="keyrow"><div class="hint">Live secret (this session)</div><code>{{ $secretLive }}</code></div>
        @endif
        @if ($secretTest)
            <div class="keyrow"><div class="hint">Test secret (this session)</div><code>{{ $secretTest }}</code></div>
        @endif

        @if (!empty($publicKey) || count($credentials))
            <form method="post" action="{{ route('credentials.issue') }}" class="actions">
                @csrf
                <input type="hidden" name="environment" value="live">
                <button type="submit">Issue / rotate live sk_…</button>
            </form>
            <form method="post" action="{{ route('credentials.issue') }}" class="actions">
                @csrf
                <input type="hidden" name="environment" value="test">
                <button class="secondary" type="submit">Issue / rotate test sk_…</button>
            </form>
            <p class="hint">Rotating invalidates the previous secret. Copy the new value immediately.</p>
        @endif
    </div>
@endsection
