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
  $accountLabel = $hasPublic ? 'Active' : ($kycApproved ? 'Provisioning' : ($kycStatus === 'pending' ? 'Pending' : 'Inactive'));
  $accountPill = $hasPublic ? 'done' : ($kycApproved || $kycStatus === 'pending' ? 'progress' : 'pending');

  $livePublic = (string) ($live['publicKey'] ?? $publicKey ?? '');
  $testPublic = (string) ($test['publicKey'] ?? '');
  $liveSecretFull = (string) ($secretLive ?: '');
  $testSecretFull = (string) ($secretTest ?: '');
  $liveSecretPrefix = (string) ($live['secretKeyPrefix'] ?? '');
  $testSecretPrefix = (string) ($test['secretKeyPrefix'] ?? '');

  $maskSecret = function (?string $full, ?string $prefix, bool $set): array {
      if (filled($full)) {
          $shown = strlen($full) > 18
              ? substr($full, 0, 14).str_repeat('•', 12)
              : $full;

          return ['display' => $shown, 'full' => $full, 'set' => true, 'revealable' => true];
      }
      if ($set && filled($prefix)) {
          return [
              'display' => rtrim($prefix, '•.').str_repeat('•', 12),
              'full' => '',
              'set' => true,
              'revealable' => false,
          ];
      }

      return ['display' => 'Not generated yet', 'full' => '', 'set' => false, 'revealable' => false];
  };

  $liveSecret = $maskSecret($liveSecretFull, $liveSecretPrefix, $liveSet);
  $testSecret = $maskSecret($testSecretFull, $testSecretPrefix, $testSet);

  if (! $canIssue) {
      $lede = 'Credentials unlock after admin KYC approval. Finish Compliance, then wait for review.';
  } elseif (! $hasPublic) {
      $lede = 'KYC is approved. Your account is being provisioned — try generating a secret below if this persists.';
  } else {
      $lede = 'Account provisioned. Use the keys below to authenticate your server API requests.';
  }
@endphp

<section class="ops-dashboard credentials-page">
  <header class="dash-hero">
    <div>
      <h2>
        <span class="page-heading-mark" aria-hidden="true"><i class="bi bi-key"></i></span>
        <span>Credentials</span>
      </h2>
      <p>{{ $lede }}</p>
    </div>
    <div class="cred-status">
      <small>Account Status</small>
      <span class="gs-pill is-{{ $accountPill }}">
        @if ($accountPill === 'done')
          <i class="bi bi-check2" aria-hidden="true"></i>
        @elseif ($accountPill === 'progress')
          <i class="gs-pill-dot" aria-hidden="true"></i>
        @else
          <i class="bi bi-clock" aria-hidden="true"></i>
        @endif
        {{ $accountLabel }}
      </span>
    </div>
  </header>

  @if (session('status'))
    <div class="flash ok" role="status">
      <strong>{{ str_contains(session('status'), 'rotated') ? 'Secret key rotated successfully!' : 'Secret key generated successfully!' }}</strong>
      Copy it now — it will not be shown again.
    </div>
  @endif
  @if (session('error'))
    <div class="flash err" role="alert">{{ session('error') }}</div>
  @endif

  <div class="gs-body">
    <section class="gs-panel" aria-labelledby="api-keys-heading">
      <div class="gs-panel-head">
        <div class="gs-panel-title">
          <span class="gs-panel-mark" aria-hidden="true"><i class="bi bi-layers"></i></span>
          <div>
            <h2 id="api-keys-heading">Your API Keys</h2>
            <p>Publishable and secret keys for live and sandbox requests.</p>
          </div>
        </div>
      </div>

      <div class="cred-env-block">
        <div class="cred-env-head">
          <strong><span class="cred-dot is-live" aria-hidden="true"></span> Production (Live)</strong>
          <span class="gs-pill is-done">LIVE</span>
        </div>

        <label class="field" for="livePublicKey">Publishable Key <span class="field-note">(browser / checkout)</span>
          <span class="cred-input-row">
            <input
              id="livePublicKey"
              type="text"
              readonly
              value="{{ $livePublic ?: 'Not provisioned yet' }}"
              data-copy-value="{{ $livePublic }}"
            >
            @if ($livePublic)
              <button type="button" class="btn btn-outline-secondary" data-copy aria-label="Copy publishable key"><i class="bi bi-clipboard"></i></button>
            @endif
          </span>
          <span class="field-note">Use in your frontend (e.g. web or mobile)</span>
        </label>

        <label class="field" for="liveSecretKey">Secret Key
          <span class="cred-input-row">
            <input
              id="liveSecretKey"
              type="text"
              readonly
              value="{{ $liveSecret['display'] }}"
              data-copy-value="{{ $liveSecret['full'] ?: ($liveSecret['set'] ? $liveSecret['display'] : '') }}"
              data-masked="{{ $liveSecret['display'] }}"
              data-full="{{ $liveSecret['full'] }}"
              data-secret-display
            >
            @if ($liveSecret['revealable'])
              <button type="button" class="btn btn-outline-secondary" data-toggle-secret aria-label="Show secret key"><i class="bi bi-eye"></i></button>
            @endif
            @if ($liveSecret['set'] || $liveSecret['full'])
              <button type="button" class="btn btn-outline-secondary" data-copy aria-label="Copy secret key"><i class="bi bi-clipboard"></i></button>
            @endif
          </span>
          <span class="field-note">Use on your server only (keep it private)</span>
        </label>
      </div>

      <div class="cred-env-block">
        <div class="cred-env-head">
          <strong><span class="cred-dot is-test" aria-hidden="true"></span> Test (Sandbox)</strong>
          <span class="gs-pill is-progress">TEST</span>
        </div>

        <label class="field" for="testPublicKey">Test Publishable Key
          <span class="cred-input-row">
            <input
              id="testPublicKey"
              type="text"
              readonly
              value="{{ $testPublic ?: 'Not provisioned yet' }}"
              data-copy-value="{{ $testPublic }}"
            >
            @if ($testPublic)
              <button type="button" class="btn btn-outline-secondary" data-copy aria-label="Copy test publishable key"><i class="bi bi-clipboard"></i></button>
            @endif
          </span>
          <span class="field-note">Use in your frontend during development</span>
        </label>

        <label class="field" for="testSecretKey">Test Secret Key
          <span class="cred-input-row">
            <input
              id="testSecretKey"
              type="text"
              readonly
              value="{{ $testSecret['display'] }}"
              data-copy-value="{{ $testSecret['full'] ?: ($testSecret['set'] ? $testSecret['display'] : '') }}"
              data-masked="{{ $testSecret['display'] }}"
              data-full="{{ $testSecret['full'] }}"
              data-secret-display
            >
            @if ($testSecret['revealable'])
              <button type="button" class="btn btn-outline-secondary" data-toggle-secret aria-label="Show test secret key"><i class="bi bi-eye"></i></button>
            @endif
            @if ($testSecret['set'] || $testSecret['full'])
              <button type="button" class="btn btn-outline-secondary" data-copy aria-label="Copy test secret key"><i class="bi bi-clipboard"></i></button>
            @endif
          </span>
          <span class="field-note">Use on your server only (keep it private)</span>
        </label>
      </div>

      @if ($canIssue)
        <div class="cred-actions">
          <form method="post" action="{{ route('credentials.issue') }}" class="auth-submit-form credentials-issue-form">
            @csrf
            <input type="hidden" name="environment" value="live">
            <button
              class="btn btn-primary auth-submit-btn"
              type="submit"
              data-idle-label="{{ $liveSet ? 'Rotate live secret key' : 'Generate live secret key' }}"
              data-loading-label="{{ $liveSet ? 'Rotating…' : 'Generating…' }}"
            >
              <span class="auth-submit-spinner" aria-hidden="true"></span>
              <i class="bi bi-arrow-repeat" aria-hidden="true"></i>
              <span class="auth-submit-label">{{ $liveSet ? 'Rotate live secret key' : 'Generate live secret key' }}</span>
            </button>
          </form>
          <form method="post" action="{{ route('credentials.issue') }}" class="auth-submit-form credentials-issue-form">
            @csrf
            <input type="hidden" name="environment" value="test">
            <button
              class="btn btn-outline-primary auth-submit-btn"
              type="submit"
              data-idle-label="{{ $testSet ? 'Rotate test secret key' : 'Generate test secret key' }}"
              data-loading-label="{{ $testSet ? 'Rotating…' : 'Generating…' }}"
            >
              <span class="auth-submit-spinner" aria-hidden="true"></span>
              <i class="bi bi-arrow-repeat" aria-hidden="true"></i>
              <span class="auth-submit-label">{{ $testSet ? 'Rotate test secret key' : 'Generate test secret key' }}</span>
            </button>
          </form>
        </div>
        <p class="field-note cred-footnote"><i class="bi bi-info-circle" aria-hidden="true"></i> Rotating invalidates the previous secret. The new value is shown only once.</p>
      @else
        <p class="field-note cred-footnote">
          Finish <a href="{{ route('compliance.show') }}">Compliance</a> first, then return here after admin approval.
        </p>
      @endif
    </section>

    <div class="gs-side">
      <section class="gs-panel" aria-labelledby="keys-safe-heading">
        <div class="gs-panel-title">
          <span class="gs-panel-mark is-help" aria-hidden="true"><i class="bi bi-shield-lock"></i></span>
          <div>
            <h2 id="keys-safe-heading">Keep your keys safe</h2>
            <p>Best practices for publishable and secret keys.</p>
          </div>
        </div>
        <div class="gs-help-links mt-3">
          <div class="cred-tip">
            <span class="gs-help-icon is-violet" aria-hidden="true"><i class="bi bi-lock"></i></span>
            <span>
              <b>Keep your secret key private</b>
              <small>Never expose it in frontend code.</small>
            </span>
          </div>
          <div class="cred-tip">
            <span class="gs-help-icon is-blue" aria-hidden="true"><i class="bi bi-arrow-repeat"></i></span>
            <span>
              <b>Rotate keys regularly</b>
              <small>For better security and compliance.</small>
            </span>
          </div>
          <div class="cred-tip">
            <span class="gs-help-icon is-green" aria-hidden="true"><i class="bi bi-shield-check"></i></span>
            <span>
              <b>Use the right environment</b>
              <small>Use TEST for development and LIVE for production.</small>
            </span>
          </div>
        </div>
        <a class="gs-outline gs-outline-block" href="https://docs.koodify.ng/" target="_blank" rel="noopener noreferrer">
          <i class="bi bi-journal-text" aria-hidden="true"></i>
          View API Documentation
          <i class="bi bi-arrow-right" aria-hidden="true"></i>
        </a>
      </section>

      <aside class="gs-env {{ $hasPublic ? 'is-live' : 'is-sandbox' }}">
        <div class="gs-env-label"><i class="bi bi-hdd-network" aria-hidden="true"></i> Environment</div>
        <div class="gs-env-state"><span aria-hidden="true"></span> {{ $hasPublic ? 'Live ready' : 'Sandbox' }}</div>
        <p>
          @if ($hasPublic)
            Publishable key is provisioned. Generate secrets for server APIs from this page.
          @elseif ($kycApproved)
            KYC approved — provisioning in progress. Secrets unlock as soon as the public key appears.
          @else
            Complete compliance and wait for admin approval before live keys are issued.
          @endif
        </p>
        <a href="{{ route('compliance.show') }}">Open compliance <i class="bi bi-arrow-right" aria-hidden="true"></i></a>
      </aside>
    </div>
  </div>
</section>
@endsection

@push('head')
<style>
  .credentials-page .dash-hero{margin-bottom:14px}
  .credentials-page .cred-status{display:flex;flex-direction:column;align-items:flex-end;gap:6px}
  .credentials-page .cred-status small{color:#94a3b8;font-size:11px;font-weight:600;text-transform:uppercase;letter-spacing:.04em}
  .credentials-page .cred-env-block{margin-top:14px;padding-top:14px;border-top:1px solid #eef3f9}
  .credentials-page .cred-env-block:first-of-type{margin-top:10px}
  .credentials-page .cred-env-head{display:flex;align-items:center;justify-content:space-between;gap:10px;margin-bottom:12px}
  .credentials-page .cred-env-head strong{display:inline-flex;align-items:center;gap:8px;font-size:14px;color:#0b1b33}
  .credentials-page .cred-dot{width:8px;height:8px;border-radius:50%;display:inline-block}
  .credentials-page .cred-dot.is-live{background:#22c55e;box-shadow:0 0 0 4px rgba(34,197,94,.16)}
  .credentials-page .cred-dot.is-test{background:#7c3aed;box-shadow:0 0 0 4px rgba(124,58,237,.16)}
  .credentials-page .cred-input-row{display:flex;align-items:stretch;gap:8px}
  .credentials-page .cred-input-row input{flex:1;min-width:0;font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;font-size:13px}
  .credentials-page .cred-input-row .btn{flex:0 0 auto;min-height:37px;padding:6px 12px}
  .credentials-page .cred-actions{display:flex;flex-wrap:wrap;gap:10px;margin:6px 0 4px}
  .credentials-page .cred-actions form{margin:0}
  .credentials-page .auth-submit-btn{display:inline-flex;align-items:center;justify-content:center;gap:.55rem}
  .credentials-page .auth-submit-btn:disabled{cursor:wait;opacity:.85}
  .credentials-page .auth-submit-spinner{width:1rem;height:1rem;border-radius:50%;border:2px solid rgba(255,255,255,.35);border-top-color:#fff;display:none;flex-shrink:0}
  .credentials-page .auth-submit-btn.is-loading .auth-submit-spinner{display:inline-block;animation:cred-spin .7s linear infinite}
  .credentials-page .btn-outline-primary .auth-submit-spinner{border-color:rgba(91,77,255,.25);border-top-color:#5b4dff}
  .credentials-page .cred-footnote{display:flex;align-items:flex-start;gap:6px;margin:8px 0 12px}
  .credentials-page .cred-tip{display:flex;align-items:flex-start;gap:12px;padding:12px 4px;border-bottom:1px solid #eef3f9}
  .credentials-page .cred-tip:last-child{border-bottom:0}
  .credentials-page .cred-tip b{display:block;font-size:14px;color:#0b1b33}
  .credentials-page .cred-tip small{display:block;margin-top:2px;color:#64748b;font-size:13px;font-weight:500}
  @keyframes cred-spin{to{transform:rotate(360deg)}}
</style>
@endpush

@push('scripts')
<script>
  document.addEventListener('DOMContentLoaded', function () {
    document.querySelectorAll('[data-copy]').forEach(function (btn) {
      btn.addEventListener('click', async function () {
        var row = btn.closest('.cred-input-row');
        var input = row ? row.querySelector('[data-copy-value]') : null;
        var value = input ? (input.getAttribute('data-copy-value') || input.value || '') : '';
        if (!value || value === 'Not generated yet' || value === 'Not provisioned yet') return;
        try {
          await navigator.clipboard.writeText(value);
          btn.innerHTML = '<i class="bi bi-check2"></i>';
          setTimeout(function () { btn.innerHTML = '<i class="bi bi-clipboard"></i>'; }, 1400);
        } catch (e) {}
      });
    });

    document.querySelectorAll('[data-toggle-secret]').forEach(function (btn) {
      btn.addEventListener('click', function () {
        var row = btn.closest('.cred-input-row');
        var input = row ? row.querySelector('[data-secret-display]') : null;
        if (!input) return;
        var full = input.getAttribute('data-full') || '';
        var masked = input.getAttribute('data-masked') || '';
        var showing = input.getAttribute('data-showing') === '1';
        if (!full) return;
        if (showing) {
          input.value = masked;
          input.setAttribute('data-showing', '0');
          btn.innerHTML = '<i class="bi bi-eye"></i>';
          btn.setAttribute('aria-label', 'Show secret key');
        } else {
          input.value = full;
          input.setAttribute('data-showing', '1');
          btn.innerHTML = '<i class="bi bi-eye-slash"></i>';
          btn.setAttribute('aria-label', 'Hide secret key');
        }
      });
    });

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
