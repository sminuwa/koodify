@extends('layouts.merchant')

@section('title', 'Credentials · Koodify')
@section('crumb', 'Credentials')

@section('content')
@php
  $kycStatus = strtolower((string) data_get($profile, 'kyc.status', ''));
  $kycApproved = in_array($kycStatus, ['approved', 'active'], true);
  $hasPublic = filled($publicKey);
  $credList = is_array($credentials) ? $credentials : [];
  $live = collect($credList)->first(fn ($c) => ($c['environment'] ?? '') === 'live');
  $test = collect($credList)->first(fn ($c) => ($c['environment'] ?? '') === 'test');
  $liveSet = ! empty($live['secretKeySet']);
  $testSet = ! empty($test['secretKeySet']);
  $canIssue = $hasPublic || $kycApproved;
@endphp

<div class="surface merchant-panel card stack credentials-panel">
  <div>
    <h1>API credentials</h1>
    <p class="lede">
      After admin approves your KYC, Koodify provisions your publishable key.
      You generate and rotate secret keys here — copy each <code>sk_…</code> once.
    </p>
  </div>

  @if (! $canIssue)
    <div class="banner">
      Credentials unlock after admin KYC approval.
      Finish <a href="{{ route('compliance.show') }}">Compliance</a>, then wait for review.
    </div>
  @elseif (! $hasPublic)
    <div class="banner">
      KYC is approved. Your account is being provisioned — try issuing a secret below,
      or ask support to retry provision if this persists.
    </div>
  @else
    <div class="banner ok">
      Account provisioned. Generate a secret key for server APIs, then store it safely.
    </div>
  @endif

  <div class="keyrow">
    <div>
      <div class="hint">Publishable key (browser / checkout)</div>
      <code class="mono">{{ $publicKey ?: 'Not provisioned yet — waiting for admin KYC approval.' }}</code>
    </div>
  </div>

  @forelse ($credList as $cred)
    <div class="keyrow">
      <div>
        <div class="hint">{{ strtoupper($cred['environment'] ?? '') }} · {{ $cred['status'] ?? '' }}</div>
        <div><code class="mono">{{ $cred['publicKey'] ?? '—' }}</code></div>
        <div class="hint">
          Secret:
          @if (! empty($cred['secretKeySet']))
            <code>{{ $cred['secretKeyPrefix'] ?? 'sk_…' }}</code> · set
          @else
            not issued yet
          @endif
        </div>
      </div>
    </div>
  @empty
    @if ($canIssue)
      <div class="banner">No credential rows yet — issuing a secret will create them.</div>
    @endif
  @endforelse

  @if ($secretLive)
    <div class="banner ok">
      <strong>Live secret (copy now)</strong>
      <div><code class="mono">{{ $secretLive }}</code></div>
    </div>
  @endif
  @if ($secretTest)
    <div class="banner ok">
      <strong>Test secret (copy now)</strong>
      <div><code class="mono">{{ $secretTest }}</code></div>
    </div>
  @endif
  @if ($secretKey && ! $secretLive && ! $secretTest)
    <div class="banner ok">
      Session secret: <code class="mono">{{ $secretKey }}</code>
    </div>
  @endif

  @if ($canIssue)
    <div class="credentials-actions">
      <form method="post" action="{{ route('credentials.issue') }}" class="auth-submit-form credentials-issue-form">
        @csrf
        <input type="hidden" name="environment" value="live">
        <button
          class="btn btn-primary auth-submit-btn"
          type="submit"
          data-idle-label="{{ $liveSet ? 'Rotate live sk_…' : 'Generate live sk_…' }}"
          data-loading-label="{{ $liveSet ? 'Rotating…' : 'Generating…' }}"
        >
          <span class="auth-submit-spinner" aria-hidden="true"></span>
          <span class="auth-submit-label">{{ $liveSet ? 'Rotate live sk_…' : 'Generate live sk_…' }}</span>
        </button>
      </form>
      <form method="post" action="{{ route('credentials.issue') }}" class="auth-submit-form credentials-issue-form">
        @csrf
        <input type="hidden" name="environment" value="test">
        <button
          class="btn btn-outline-secondary auth-submit-btn"
          type="submit"
          data-idle-label="{{ $testSet ? 'Rotate test sk_…' : 'Generate test sk_…' }}"
          data-loading-label="{{ $testSet ? 'Rotating…' : 'Generating…' }}"
        >
          <span class="auth-submit-spinner" aria-hidden="true"></span>
          <span class="auth-submit-label">{{ $testSet ? 'Rotate test sk_…' : 'Generate test sk_…' }}</span>
        </button>
      </form>
    </div>
    <p class="hint">Rotating invalidates the previous secret. The new value is shown only once.</p>
  @endif
</div>
@endsection

@push('head')
<style>
  .credentials-panel .credentials-actions{display:flex;flex-wrap:wrap;gap:10px;margin-top:8px}
  .credentials-panel .auth-submit-btn{display:inline-flex;align-items:center;justify-content:center;gap:.55rem;width:auto;min-width:12rem}
  .credentials-panel .auth-submit-btn:disabled{cursor:wait;opacity:.85}
  .credentials-panel .auth-submit-spinner{width:1rem;height:1rem;border-radius:50%;border:2px solid rgba(255,255,255,.35);border-top-color:#fff;display:none;flex-shrink:0}
  .credentials-panel .auth-submit-btn.is-loading .auth-submit-spinner{display:inline-block;animation:cred-spin .7s linear infinite}
  .credentials-panel .auth-submit-btn.btn-outline-secondary .auth-submit-spinner{border-color:rgba(67,56,202,.25);border-top-color:#4338ca}
  .credentials-panel .mono{word-break:break-all}
  @keyframes cred-spin{to{transform:rotate(360deg)}}
</style>
@endpush

@push('scripts')
<script>
  document.addEventListener('DOMContentLoaded', function () {
    function setLoading(btn, loading) {
      if (!btn) return;
      var label = btn.querySelector('.auth-submit-label');
      var idle = btn.getAttribute('data-idle-label') || 'Continue';
      var busy = btn.getAttribute('data-loading-label') || 'Please wait…';
      btn.disabled = loading;
      btn.classList.toggle('is-loading', loading);
      btn.setAttribute('aria-busy', loading ? 'true' : 'false');
      if (label) label.textContent = loading ? busy : idle;
    }

    document.querySelectorAll('form.credentials-issue-form').forEach(function (form) {
      form.addEventListener('submit', function (event) {
        if (form.dataset.submitting === '1') {
          event.preventDefault();
          return;
        }
        form.dataset.submitting = '1';
        setLoading(form.querySelector('button.auth-submit-btn[type="submit"]'), true);
      });
    });

    window.addEventListener('pageshow', function (event) {
      if (!event.persisted) return;
      document.querySelectorAll('form.credentials-issue-form').forEach(function (form) {
        form.dataset.submitting = '0';
        setLoading(form.querySelector('button.auth-submit-btn[type="submit"]'), false);
      });
    });
  });
</script>
@endpush
