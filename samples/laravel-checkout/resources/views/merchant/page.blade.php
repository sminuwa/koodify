@extends('layouts.merchant')

@section('title', $title.' · Koodify')
@section('crumb', $title)

@section('content')
<div class="get-started">
  <section class="gs-hero" style="grid-template-columns: 1fr;">
    <div class="gs-intro">
      <h1><span class="page-heading-mark" aria-hidden="true"><i class="bi bi-layers"></i></span> {{ $title }}</h1>
      <p>{{ $subtitle }}</p>
    </div>
  </section>

  <section class="gs-panel" style="margin-top:18px">
    <div class="gs-panel-title">
      <span class="gs-panel-mark" aria-hidden="true"><i class="bi bi-info-circle"></i></span>
      <div>
        <h2>Coming soon</h2>
        <p>{{ $description }}</p>
      </div>
    </div>
    <div class="gs-help-links mt-3">
      <a href="{{ route('get-started') }}">
        <span class="gs-help-icon is-green" aria-hidden="true"><i class="bi bi-compass"></i></span>
        Back to Get Started
        <i class="bi bi-chevron-right" aria-hidden="true"></i>
      </a>
      <a href="{{ route('credentials.show') }}">
        <span class="gs-help-icon is-blue" aria-hidden="true"><i class="bi bi-key"></i></span>
        Manage credentials
        <i class="bi bi-chevron-right" aria-hidden="true"></i>
      </a>
      <a href="https://docs.koodify.ng/" target="_blank" rel="noopener noreferrer">
        <span class="gs-help-icon is-violet" aria-hidden="true"><i class="bi bi-journal-text"></i></span>
        Open API docs
        <i class="bi bi-chevron-right" aria-hidden="true"></i>
      </a>
    </div>
  </section>
</div>
@endsection
