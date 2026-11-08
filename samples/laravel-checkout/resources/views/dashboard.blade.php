@extends('layouts.merchant')

@section('title', 'Get Started')
@section('crumb', 'Get Started')

@section('content')
@php
  $kycStatus = is_array($kyc ?? null) ? ($kyc['status'] ?? 'none') : 'none';
  $approved = in_array($kycStatus, ['approved', 'active'], true);
  $kycPending = $kycStatus === 'pending';
  $hasPublic = ! empty($profile['publicKey']);
  $keyReady = $hasSecret || $hasPublic;
  $accountReady = true; // registered + logged in
  $apiBase = (string) config('koodify.api_base');
  $live = ! str_contains($apiBase, '127.0.0.1') && ! str_contains($apiBase, 'localhost') && ! str_contains($apiBase, 'sandbox');

  $steps = [
    [
      'title' => 'Create your merchant account',
      'detail' => 'You are signed in and ready to continue setup.',
      'href' => route('get-started'),
      'state' => $accountReady ? 'done' : 'progress',
      'status' => $accountReady ? 'Completed' : 'In progress',
    ],
    [
      'title' => 'Submit KYC for review',
      'detail' => 'Verify your business so Koodify can provision a zainbox and keys.',
      'href' => route('compliance.show'),
      'state' => $approved ? 'done' : ($kycPending ? 'progress' : 'pending'),
      'status' => $approved ? 'Completed' : ($kycPending ? 'In review' : 'Next'),
    ],
    [
      'title' => 'Issue API credentials',
      'detail' => 'Generate publishable and secret keys for checkout and server APIs.',
      'href' => route('credentials.show'),
      'state' => $keyReady ? 'done' : ($approved ? 'progress' : 'pending'),
      'status' => $keyReady ? 'Completed' : ($approved ? 'Ready' : 'Pending'),
    ],
    [
      'title' => 'Try hosted checkout',
      'detail' => 'Run the inline checkout sample with your merchant keys.',
      'href' => route('checkout'),
      'state' => $keyReady ? 'progress' : 'pending',
      'status' => $keyReady ? 'Next' : 'Pending',
    ],
    [
      'title' => 'Create a virtual account',
      'detail' => 'Open a static VA for collections once credentials are available.',
      'href' => route('payment-simulation', ['tool' => 'va']),
      'state' => 'pending',
      'status' => 'Pending',
    ],
    [
      'title' => 'Check account balance',
      'detail' => 'Look up balances for accounts linked to your merchant.',
      'href' => route('payment-simulation', ['tool' => 'balance']),
      'state' => 'pending',
      'status' => 'Pending',
    ],
  ];

  $done = collect($steps)->where('state', 'done')->count();
  $total = count($steps);
  $percent = (int) round(($done / max($total, 1)) * 100);

  $cards = [
    [
      'label' => 'Private Key',
      'value' => $hasSecret ? 'Ready' : ($hasPublic ? 'Public only' : 'Missing'),
      'detail' => $hasSecret ? 'Your environment is configured.' : ($hasPublic ? 'Issue sk_ from Credentials.' : 'Complete KYC, then issue keys.'),
      'href' => route('credentials.show'),
      'icon' => 'bi-key',
      'tone' => $hasSecret ? 'green' : 'amber',
    ],
    [
      'label' => 'KYC',
      'value' => ucfirst($kycStatus === 'none' ? 'Not started' : $kycStatus),
      'detail' => $approved ? 'Approved and ready for provisioning.' : ($kycPending ? 'Waiting for admin review.' : 'Submit verification to unlock keys.'),
      'href' => route('compliance.show'),
      'icon' => 'bi-shield-check',
      'tone' => $approved ? 'green' : ($kycPending ? 'blue' : 'amber'),
    ],
    [
      'label' => 'Tools',
      'value' => '4',
      'detail' => 'Checkout, VA, Dynamic VA, and balance samples.',
      'href' => route('checkout'),
      'icon' => 'bi-boxes',
      'tone' => 'blue',
    ],
  ];
@endphp

@if(session('status'))
  <div class="flash ok" style="margin-bottom:16px">{{ session('status') }}</div>
@endif

<div class="get-started">
  <section class="gs-hero">
    <div class="gs-intro">
      <h1><span class="page-heading-mark" aria-hidden="true"><i class="bi bi-compass"></i></span> Get Started with Koodify</h1>
      <p>Follow these steps to set up your account, complete KYC, and start collecting payments.</p>
      <div class="gs-progress" role="progressbar" aria-valuenow="{{ $percent }}" aria-valuemin="0" aria-valuemax="100" aria-label="{{ $done }} of {{ $total }} completed">
        <div class="gs-progress-track">
          <span style="width: {{ $percent }}%"></span>
        </div>
        <div class="gs-progress-meta">
          <b>{{ $done }} of {{ $total }} completed</b>
          <b>{{ $percent }}%</b>
        </div>
      </div>
    </div>

    <div class="gs-art" aria-hidden="true">
      <div class="gs-art-glow"></div>
      <div class="gs-art-card gs-art-card-back">
        <span class="gs-mark"></span>
        <i class="bi bi-check-circle-fill"></i>
      </div>
      <div class="gs-art-sheet">
        <span></span>
        <span></span>
        <span class="is-check"></span>
        <span></span>
        <span class="is-check"></span>
      </div>
      <div class="gs-art-card gs-art-card-front">
        <span class="gs-mark"></span>
        <i class="bi bi-check-circle-fill"></i>
      </div>
    </div>

    <aside class="gs-env {{ $live ? 'is-live' : 'is-sandbox' }}">
      <div class="gs-env-label"><i class="bi bi-hdd-network" aria-hidden="true"></i> Environment</div>
      <div class="gs-env-state"><span aria-hidden="true"></span> {{ $live ? 'Live' : 'Sandbox' }}</div>
      <p>{{ $live ? 'This account uses the production API. Real payments move here.' : 'This account uses the local/sandbox API. Test collections and payouts here.' }}</p>
      <a href="{{ route('credentials.show') }}">View environment details <i class="bi bi-arrow-right" aria-hidden="true"></i></a>
    </aside>
  </section>

  <section class="gs-stats" aria-label="Setup status">
    @foreach ($cards as $card)
      <a class="gs-stat" href="{{ $card['href'] }}">
        <span class="gs-stat-icon is-{{ $card['tone'] }}" aria-hidden="true"><i class="bi {{ $card['icon'] }}"></i></span>
        <span class="gs-stat-copy">
          <small>{{ $card['label'] }}</small>
          <strong>{{ $card['value'] }}</strong>
          <span>{{ $card['detail'] }}</span>
        </span>
        <i class="bi bi-chevron-right" aria-hidden="true"></i>
      </a>
    @endforeach
  </section>

  <div class="gs-body">
    <section class="gs-panel" aria-labelledby="setup-checklist">
      <div class="gs-panel-head">
        <div class="gs-panel-title">
          <span class="gs-panel-mark" aria-hidden="true"><i class="bi bi-list-check"></i></span>
          <div>
            <h2 id="setup-checklist">Setup checklist</h2>
            <p>Complete these steps to start receiving live payments.</p>
          </div>
        </div>
        <a class="gs-outline" href="https://docs.koodify.ng/" target="_blank" rel="noopener noreferrer">View documentation <i class="bi bi-arrow-right" aria-hidden="true"></i></a>
      </div>
      <div class="gs-steps mt-3">
        @foreach ($steps as $i => $step)
          <a class="gs-step is-{{ $step['state'] }}" href="{{ $step['href'] }}">
            <span class="gs-step-mark" aria-hidden="true">
              @if ($step['state'] === 'done')
                <i class="bi bi-check-lg"></i>
              @else
                {{ $i + 1 }}
              @endif
            </span>
            <span class="gs-step-copy">
              <b>{{ $step['title'] }}</b>
              <small>{{ $step['detail'] }}</small>
            </span>
            <span class="gs-pill is-{{ $step['state'] }}">
              @if ($step['state'] === 'done')
                <i class="bi bi-check2" aria-hidden="true"></i>
              @elseif ($step['state'] === 'progress')
                <i class="gs-pill-dot" aria-hidden="true"></i>
              @else
                <i class="bi bi-clock" aria-hidden="true"></i>
              @endif
              {{ $step['status'] }}
            </span>
            <i class="bi bi-chevron-right" aria-hidden="true"></i>
          </a>
        @endforeach
      </div>
    </section>

    <div class="gs-side">
      <section class="gs-panel gs-video" aria-labelledby="video-walkthrough">
        <div class="gs-panel-title">
          <span class="gs-panel-mark is-play" aria-hidden="true"><i class="bi bi-play-fill"></i></span>
          <div>
            <h2 id="video-walkthrough">Video walkthrough</h2>
            <p>Get up and running in minutes.</p>
          </div>
        </div>
        <a class="gs-video-thumb" href="https://docs.koodify.ng/">
          <span class="gs-video-brand">
            <span class="gs-mark" aria-hidden="true"></span>
            <span>
              <b>koodify</b>
              <small>Quick Setup Guide</small>
            </span>
          </span>
          <span class="gs-play" aria-hidden="true"><i class="bi bi-play-fill"></i></span>
          <span class="gs-video-time">4:32</span>
        </a>
        <a class="gs-outline gs-outline-block" href="https://docs.koodify.ng/">Watch now <i class="bi bi-arrow-right" aria-hidden="true"></i></a>
      </section>

      <section class="gs-panel" aria-labelledby="need-help">
        <div class="gs-panel-title">
          <span class="gs-panel-mark is-help" aria-hidden="true"><i class="bi bi-question-lg"></i></span>
          <div>
            <h2 id="need-help">Need help?</h2>
            <p>Check our guides or contact support.</p>
          </div>
        </div>
        <div class="gs-help-links mt-3">
          <a href="https://docs.koodify.ng/">
            <span class="gs-help-icon is-green" aria-hidden="true"><i class="bi bi-journal-text"></i></span>
            Read the developer docs
            <i class="bi bi-chevron-right" aria-hidden="true"></i>
          </a>
          <a href="{{ route('credentials.show') }}">
            <span class="gs-help-icon is-blue" aria-hidden="true"><i class="bi bi-key"></i></span>
            Manage API credentials
            <i class="bi bi-chevron-right" aria-hidden="true"></i>
          </a>
          <a href="mailto:support@koodify.ng">
            <span class="gs-help-icon is-violet" aria-hidden="true"><i class="bi bi-headset"></i></span>
            Contact support
            <i class="bi bi-chevron-right" aria-hidden="true"></i>
          </a>
        </div>
      </section>
    </div>
  </div>
</div>
@endsection
