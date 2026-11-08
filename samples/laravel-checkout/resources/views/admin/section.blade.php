@extends('layouts.admin')

@section('title', $title.' · Admin')
@section('crumb', $title)

@section('content')
<div class="page-heading">
  <div>
    <h1>
      <span class="page-heading-mark" aria-hidden="true"><i class="bi bi-layers"></i></span>
      {{ $title }}
    </h1>
    <p>This section will be wired to live Koodify APIs next.</p>
  </div>
</div>

<section class="surface merchant-panel">
  <div class="empty-state" style="padding:40px;text-align:center">
    <h3>{{ $title }} preview</h3>
    <p class="lede">Use Merchants for live KYC approval today. Overview already reads the pending review queue.</p>
    <div class="admin-actions" style="justify-content:center">
      <a class="btn btn-primary" href="{{ route('admin.merchants.index') }}">Go to Merchants</a>
      <a class="btn btn-outline-secondary" href="{{ route('admin.dashboard') }}">Overview</a>
    </div>
  </div>
</section>
@endsection
