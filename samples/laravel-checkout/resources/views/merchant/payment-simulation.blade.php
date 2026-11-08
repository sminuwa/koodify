@extends('layouts.merchant')

@section('title', 'Payment Simulation · Koodify')
@section('crumb', 'Payment Simulation')

@section('content')
@php
  $tabs = [
    'va' => ['label' => 'Static VA', 'icon' => 'bi-boxes', 'hint' => 'Create a reusable collection account'],
    'dva' => ['label' => 'Dynamic VA', 'icon' => 'bi-diagram-3', 'hint' => 'One-time account for a fixed amount'],
    'balance' => ['label' => 'Balance', 'icon' => 'bi-wallet2', 'hint' => 'Look up a virtual account balance'],
  ];
@endphp

<section class="ops-dashboard" id="paymentSimulation">
  <header class="dash-hero">
    <div>
      <h2>
        <span class="page-heading-mark" aria-hidden="true"><i class="bi bi-play-circle"></i></span>
        <span>Payment Simulation</span>
      </h2>
      <p>Try static VA, dynamic VA, and balance lookups against your sandbox or live secret key.</p>
    </div>
    <a class="menu-link" href="{{ route('virtual-accounts.index') }}">View virtual accounts <i class="bi bi-arrow-right" aria-hidden="true"></i></a>
  </header>

  @unless($configured)
    <div class="flash err" style="margin-bottom:16px">
      Issue a merchant <code>sk_…</code> from <a href="{{ route('credentials.show') }}">Credentials</a>, or set <code>KOODIFY_SECRET_KEY</code> in <code>.env</code>.
    </div>
  @endunless

  <div class="sim-tabs" role="tablist" aria-label="Simulation tools">
    @foreach ($tabs as $key => $tab)
      <a
        href="{{ route('payment-simulation', ['tool' => $key]) }}"
        @class(['sim-tab', 'is-on' => $tool === $key])
        role="tab"
        aria-selected="{{ $tool === $key ? 'true' : 'false' }}"
      >
        <span class="sim-tab-icon" aria-hidden="true"><i class="bi {{ $tab['icon'] }}"></i></span>
        <span>
          <strong>{{ $tab['label'] }}</strong>
          <small>{{ $tab['hint'] }}</small>
        </span>
      </a>
    @endforeach
  </div>

  <div class="merchant-panel" style="margin-top:18px">
    @if ($tool === 'va')
      <h1>Create a static virtual account</h1>
      <p class="lede">
        Server call to <code>POST /api/v1/va/create</code> with your <code>sk_…</code> key.
        Koodify resolves your zainbox and bank routing — you do not send those.
      </p>

      @if ($vaError)
        <div class="flash err">{{ $vaError }}</div>
      @endif

      <form method="post" action="{{ route('virtual-accounts.store') }}">
        @csrf
        <input type="hidden" name="redirect_tool" value="va">

        <div class="field">
          <label for="title">Title</label>
          <input class="form-control" id="title" name="title" value="{{ old('title', $vaDefaults['title']) }}" required>
        </div>

        <div class="grid">
          <div class="field">
            <label for="firstName">First name</label>
            <input class="form-control" id="firstName" name="firstName" value="{{ old('firstName', $vaDefaults['firstName']) }}" required>
          </div>
          <div class="field">
            <label for="surname">Surname</label>
            <input class="form-control" id="surname" name="surname" value="{{ old('surname', $vaDefaults['surname']) }}" required>
          </div>
        </div>

        <div class="grid">
          <div class="field">
            <label for="email">Email</label>
            <input class="form-control" id="email" name="email" type="email" value="{{ old('email', $vaDefaults['email']) }}" required>
          </div>
          <div class="field">
            <label for="mobileNumber">Mobile</label>
            <input class="form-control" id="mobileNumber" name="mobileNumber" value="{{ old('mobileNumber', $vaDefaults['mobileNumber']) }}" required>
          </div>
        </div>

        <div class="field">
          <label for="address">Address</label>
          <input class="form-control" id="address" name="address" value="{{ old('address', $vaDefaults['address']) }}" required>
        </div>

        <div class="field">
          <label for="state">State</label>
          <input class="form-control" id="state" name="state" value="{{ old('state', $vaDefaults['state']) }}" required>
        </div>

        <div class="actions">
          <button class="btn btn-primary" type="submit" @disabled(! $configured)>Create virtual account</button>
          <span class="hint">Uses <code>X-Api-Key: sk_…</code> on the server only</span>
        </div>
      </form>

      @if ($vaResult)
        @php $ok = ($vaResult['code'] ?? null) === '00' || ($vaResult['status'] ?? '') === '200 OK'; @endphp
        <div class="status {{ $ok ? 'ok' : 'err' }}" aria-live="polite">{{ json_encode($vaResult, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES) }}</div>
      @else
        <div class="status" aria-live="polite">API response will appear here…</div>
      @endif

      <p class="meta">API: <code>{{ rtrim(config('koodify.api_base'), '/') }}/api/v1/va/create</code></p>
    @elseif ($tool === 'dva')
      <h1>Create a dynamic virtual account</h1>
      <p class="lede">
        One-time account for a fixed amount via <code>POST /api/v1/dva/create</code>.
        Pay the exact amount before the account expires.
      </p>

      @if ($dvaError)
        <div class="flash err">{{ $dvaError }}</div>
      @endif

      <form method="post" action="{{ route('dynamic-virtual-accounts.store') }}">
        @csrf
        <input type="hidden" name="redirect_tool" value="dva">

        <div class="field">
          <label for="dvaEmail">Email</label>
          <input class="form-control" id="dvaEmail" name="email" type="email" value="{{ old('email', $dvaDefaults['email']) }}" required>
        </div>

        <div class="grid">
          <div class="field">
            <label for="amount">Amount (kobo)</label>
            <input class="form-control" id="amount" name="amount" value="{{ old('amount', $dvaDefaults['amount']) }}" required>
            <p class="hint" style="margin:.35rem 0 0">e.g. <code>50000</code> = ₦500.00</p>
          </div>
          <div class="field">
            <label for="duration">Duration (seconds)</label>
            <input class="form-control" id="duration" name="duration" type="number" min="300" max="259200" value="{{ old('duration', $dvaDefaults['duration']) }}" required>
            <p class="hint" style="margin:.35rem 0 0">300–259200 (5 min – 72 h)</p>
          </div>
        </div>

        <div class="field">
          <label for="txnRef">Transaction reference</label>
          <input class="form-control" id="txnRef" name="txnRef" value="{{ old('txnRef', $dvaDefaults['txnRef']) }}">
          <p class="hint" style="margin:.35rem 0 0">Leave blank to auto-generate</p>
        </div>

        <div class="field">
          <label for="callBackUrl">Callback URL</label>
          <input class="form-control" id="callBackUrl" name="callBackUrl" type="url" value="{{ old('callBackUrl', $dvaDefaults['callBackUrl']) }}">
        </div>

        <div class="actions">
          <button class="btn btn-primary" type="submit" @disabled(! $configured)>Create dynamic VA</button>
          <span class="hint">Uses <code>X-Api-Key: sk_…</code> on the server only</span>
        </div>
      </form>

      @if ($dvaResult)
        @php $ok = ($dvaResult['code'] ?? null) === '00' || ($dvaResult['status'] ?? '') === '200 OK'; @endphp
        <div class="status {{ $ok ? 'ok' : 'err' }}" aria-live="polite">{{ json_encode($dvaResult, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES) }}</div>
      @else
        <div class="status" aria-live="polite">API response will appear here…</div>
      @endif

      <p class="meta">API: <code>{{ rtrim(config('koodify.api_base'), '/') }}/api/v1/dva/create</code></p>
    @else
      <h1>Check virtual account balance</h1>
      <p class="lede">
        Server call to <code>GET /api/v1/va/balance/{accountNumber}</code> with your <code>sk_…</code> key.
      </p>

      @if ($balanceError)
        <div class="flash err">{{ $balanceError }}</div>
      @endif

      <form method="post" action="{{ route('account-balance.lookup') }}">
        @csrf
        <input type="hidden" name="redirect_tool" value="balance">

        <div class="field">
          <label for="accountNumber">Account number</label>
          <input
            class="form-control"
            id="accountNumber"
            name="accountNumber"
            inputmode="numeric"
            autocomplete="off"
            placeholder="e.g. 5006079921"
            value="{{ $accountNumber }}"
            required
          >
        </div>

        <div class="actions">
          <button class="btn btn-primary" type="submit" @disabled(! $configured)>Check balance</button>
          <span class="hint">Uses <code>X-Api-Key: sk_…</code> on the server only</span>
        </div>
      </form>

      @if ($balanceResult)
        @php $ok = ($balanceResult['code'] ?? null) === '00' || ($balanceResult['status'] ?? '') === '200 OK'; @endphp
        <div class="status {{ $ok ? 'ok' : 'err' }}" aria-live="polite">{{ json_encode($balanceResult, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES) }}</div>
      @else
        <div class="status" aria-live="polite">API response will appear here…</div>
      @endif

      <p class="meta">API: <code>{{ rtrim(config('koodify.api_base'), '/') }}/api/v1/va/balance/{accountNumber}</code></p>
    @endif
  </div>
</section>
@endsection

@push('head')
<style>
  .sim-tabs{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:12px;margin-top:8px}
  .sim-tab{display:flex;align-items:flex-start;gap:12px;padding:14px 16px;border:1px solid var(--line,#e4eaf2);border-radius:14px;background:rgba(255,255,255,.72);text-decoration:none;color:inherit;transition:box-shadow .16s ease,border-color .16s ease,transform .16s ease}
  .sim-tab:hover{border-color:#c7c3ff;box-shadow:0 8px 20px rgba(91,77,255,.08)}
  .sim-tab.is-on{background:#fff;border-color:transparent;box-shadow:0 1px 2px rgba(11,27,51,.04),0 8px 20px rgba(91,77,255,.12)}
  .sim-tab strong{display:block;color:var(--navy,#0b1b33);font-size:.95rem}
  .sim-tab small{display:block;color:var(--muted,#546578);font-size:.78rem;margin-top:2px;line-height:1.35}
  .sim-tab-icon{width:36px;height:36px;border-radius:10px;display:grid;place-items:center;background:var(--blue-soft,#eef0ff);color:var(--blue,#5b4dff);flex-shrink:0}
  .sim-tab.is-on .sim-tab-icon{background:var(--brand-grad,linear-gradient(135deg,#2563eb,#5b4dff,#7c3aed));color:#fff}
  .merchant-panel .status{margin-top:18px;padding:14px;border-radius:12px;background:#0b1b33;color:#e8eefc;font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:12px;white-space:pre-wrap;overflow:auto}
  .merchant-panel .status.ok{box-shadow:inset 0 0 0 1px rgba(16,185,129,.45)}
  .merchant-panel .status.err{box-shadow:inset 0 0 0 1px rgba(239,68,68,.45)}
  .merchant-panel .meta{margin:14px 0 0;color:var(--muted,#546578);font-size:.82rem}
  .merchant-panel .hint{color:var(--muted,#546578);font-size:.8rem}
  @media(max-width:900px){.sim-tabs{grid-template-columns:1fr}}
</style>
@endpush
