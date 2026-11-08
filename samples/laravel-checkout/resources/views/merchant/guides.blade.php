@extends('layouts.merchant')

@section('title', 'Guides & Tutorials · Koodify')
@section('crumb', 'Guides & Tutorials')

@section('content')
@php
  $kycStatus = is_array($kyc ?? null) ? ($kyc['status'] ?? 'none') : 'none';
  $approved = in_array($kycStatus, ['approved', 'active'], true);
  $keyReady = $hasSecret || ! empty($profile['publicKey'] ?? null);

  $steps = [
    ['title' => 'Create merchant account', 'state' => 'done'],
    ['title' => 'Submit compliance / KYC', 'state' => $approved ? 'done' : ($kycStatus === 'pending' ? 'progress' : 'pending')],
    ['title' => 'Issue API credentials', 'state' => $keyReady ? 'done' : ($approved ? 'progress' : 'pending')],
    ['title' => 'Try checkout', 'state' => $keyReady ? 'progress' : 'pending'],
    ['title' => 'Create a virtual account', 'state' => 'pending'],
    ['title' => 'Go live', 'state' => 'pending'],
  ];
  $done = collect($steps)->where('state', 'done')->count();
  $total = count($steps);
  $percent = (int) round(($done / max($total, 1)) * 100);

  $guides = [
    [
      'title' => 'API Integration',
      'detail' => 'Integrate using our REST API with simple HTTP requests.',
      'icon' => 'bi-code-slash',
      'tone' => 'blue',
      'caption' => 'API Integration',
      'duration' => '5:24',
      'href' => 'https://docs.koodify.ng/',
    ],
    [
      'title' => 'Inline Checkout',
      'detail' => 'Add a secure payment widget to your website with minimal code.',
      'icon' => 'bi-window',
      'tone' => 'sky',
      'caption' => 'Payment Widget',
      'duration' => '3:45',
      'href' => route('checkout'),
    ],
    [
      'title' => 'Virtual Accounts',
      'detail' => 'Create static and dynamic virtual accounts for collections.',
      'icon' => 'bi-boxes',
      'tone' => 'green',
      'caption' => 'Virtual Accounts',
      'duration' => '4:10',
      'href' => route('payment-simulation', ['tool' => 'va']),
    ],
    [
      'title' => 'Webhooks',
      'detail' => 'Receive real-time events for payments and settlements.',
      'icon' => 'bi-link-45deg',
      'tone' => 'violet',
      'caption' => 'Webhooks Setup',
      'duration' => '4:02',
      'href' => route('webhooks'),
    ],
    [
      'title' => 'Credentials',
      'detail' => 'Generate and secure publishable and secret API keys.',
      'icon' => 'bi-key',
      'tone' => 'amber',
      'caption' => 'API Keys',
      'duration' => '3:12',
      'href' => route('credentials.show'),
    ],
  ];
@endphp

<div class="qs">
  <div class="qs-top">
    <section class="qs-hero">
      <div class="qs-copy">
        <p class="qs-chip"><i class="bi bi-rocket-takeoff" aria-hidden="true"></i> Quick Start</p>
        <h1>Integrate Koodify<br>in minutes</h1>
        <p>Watch step-by-step video guides to learn how to integrate Koodify, set up your account, and start collecting payments.</p>
        <div class="qs-actions">
          <a class="btn btn-primary" href="#qs-guides"><i class="bi bi-play-fill" aria-hidden="true"></i> Watch introduction</a>
          <a class="btn btn-outline-primary" href="https://docs.koodify.ng/" target="_blank" rel="noopener noreferrer"><i class="bi bi-journal-text" aria-hidden="true"></i> View documentation</a>
        </div>
      </div>

      <div class="qs-stage">
        <img src="{{ asset('images/preview1.png') }}" alt="Koodify merchant dashboard preview.">
        <a class="qs-stage-play" href="#qs-guides" aria-label="Play introduction">
          <svg viewBox="0 0 12 14" aria-hidden="true"><path d="M1 1.1 11 7 1 12.9V1.1z"/></svg>
        </a>
      </div>
    </section>

    <aside class="gs-panel qs-rail">
      <p class="qs-complete"><b>{{ $percent }}%</b> complete</p>
      <div class="gs-progress">
        <div class="gs-progress-track" role="progressbar" aria-valuenow="{{ $percent }}" aria-valuemin="0" aria-valuemax="100" aria-label="{{ $percent }} percent complete">
          <span style="width: {{ $percent }}%"></span>
        </div>
      </div>
      <div class="qs-rail-art" aria-hidden="true">
        <img src="{{ asset('images/compliance-kyc.png') }}" alt="">
      </div>
      <div class="qs-rail-links">
        <a class="gs-outline" href="{{ route('compliance.show') }}">Compliance <i class="bi bi-arrow-right" aria-hidden="true"></i></a>
        <a class="gs-outline" href="{{ route('get-started') }}">Get Started <i class="bi bi-arrow-right" aria-hidden="true"></i></a>
      </div>
    </aside>
  </div>

  <section class="qs-guides" id="qs-guides">
    <div class="qs-section-head">
      <div>
        <h2>Integration video guides</h2>
        <p>Choose your preferred integration method and follow the step-by-step tutorial.</p>
      </div>
      <a href="#qs-guides">View all videos <i class="bi bi-arrow-right" aria-hidden="true"></i></a>
    </div>
    <div class="qs-guide-grid">
      @foreach ($guides as $guide)
        <article class="gs-panel qs-guide">
          <div class="gs-panel-title">
            <span class="gs-stat-icon is-{{ $guide['tone'] }}" aria-hidden="true"><i class="bi {{ $guide['icon'] }}"></i></span>
            <div>
              <h3>{{ $guide['title'] }}</h3>
              <p>{{ $guide['detail'] }}</p>
            </div>
          </div>
          <div class="qs-thumb is-{{ $guide['tone'] }}" aria-hidden="true">
            <div class="qs-scene">
              <span>koodify</span>
              <b>{{ $guide['caption'] }}</b>
              <span class="qs-lines"><i></i><i></i><i></i><i class="is-short"></i></span>
            </div>
            <span class="qs-thumb-play"><svg viewBox="0 0 12 14" aria-hidden="true"><path d="M1 1.1 11 7 1 12.9V1.1z"/></svg></span>
            <span class="qs-thumb-time">{{ $guide['duration'] }}</span>
          </div>
          <a class="btn btn-outline-primary" href="{{ $guide['href'] }}">Watch guide <i class="bi bi-arrow-right" aria-hidden="true"></i></a>
        </article>
      @endforeach
    </div>
  </section>

  <section class="qs-resources">
    <article class="gs-panel qs-resource">
      <div class="gs-panel-title">
        <span class="gs-stat-icon is-green" aria-hidden="true"><i class="bi bi-journal-text"></i></span>
        <div>
          <h3>Read the integration guide</h3>
          <p>Detailed documentation with code samples, API references, and best practices.</p>
        </div>
      </div>
      <a class="btn btn-outline-primary" href="https://docs.koodify.ng/" target="_blank" rel="noopener noreferrer">View docs <i class="bi bi-arrow-right" aria-hidden="true"></i></a>
    </article>
    <article class="gs-panel qs-resource">
      <div class="gs-panel-title">
        <span class="gs-stat-icon is-sky" aria-hidden="true"><i class="bi bi-flask"></i></span>
        <div>
          <h3>Try the sandbox</h3>
          <p>Test your integration with sample data before going live.</p>
        </div>
      </div>
      <a class="btn btn-outline-primary" href="{{ route('payment-simulation') }}">Go to sandbox <i class="bi bi-arrow-right" aria-hidden="true"></i></a>
    </article>
    <article class="gs-panel qs-resource">
      <div class="gs-panel-title">
        <span class="gs-stat-icon is-violet" aria-hidden="true"><i class="bi bi-chat-dots"></i></span>
        <div>
          <h3>Need help from support?</h3>
          <p>Reach the Koodify team if you get stuck during integration.</p>
        </div>
      </div>
      <a class="btn btn-outline-primary" href="mailto:support@koodify.ng">Contact support <i class="bi bi-arrow-right" aria-hidden="true"></i></a>
    </article>
  </section>
</div>
@endsection
