@extends('layouts.admin')

@section('title', $title.' · Koodify Admin')
@section('breadcrumb', $title)

@section('content')
<div class="page-heading">
  <div>
    <span class="card-kicker">COMING SOON</span>
    <h1>{{ $title }}</h1>
    <p class="muted">This section is part of the admin UI prototype and will be wired to live Koodify APIs next.</p>
  </div>
</div>
<section class="surface">
  <div class="empty-state">
    <h3>{{ $title }} preview</h3>
    <p>Use <b>Merchants</b> for live KYC approval today. Overview already reads the customer registry.</p>
    <p style="margin-top:16px"><a class="primary-button" href="{{ route('admin.merchants.index') }}" style="display:inline-flex;padding:12px 18px;text-decoration:none"><span>Go to Merchants</span><b>→</b></a></p>
  </div>
</section>
@endsection
