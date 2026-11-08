@extends('layouts.merchant')

@section('title', 'Profile · Koodify')
@section('crumb', 'Profile')

@section('content')
@php
  $display = $profile['displayName'] ?? session('merchant.display_name', 'Merchant');
  $email = $profile['email'] ?? session('merchant.email', '—');
  $kycStatus = is_array($profile['kyc'] ?? null) ? (($profile['kyc']['status'] ?? 'none')) : 'none';
@endphp
<div class="gs-panel">
  <div class="gs-panel-title">
    <span class="gs-panel-mark" aria-hidden="true"><i class="bi bi-person"></i></span>
    <div>
      <h2>Profile</h2>
      <p>Signed-in merchant account for this portal session.</p>
    </div>
  </div>
  <div class="gs-stats mt-3" style="grid-template-columns:1fr">
    <div class="gs-stat" style="cursor:default">
      <span class="gs-stat-icon is-blue" aria-hidden="true"><i class="bi bi-building"></i></span>
      <span class="gs-stat-copy">
        <small>Business</small>
        <strong>{{ $display }}</strong>
        <span>Merchant #{{ $profile['id'] ?? '—' }} · status {{ $profile['status'] ?? '—' }}</span>
      </span>
    </div>
    <div class="gs-stat" style="cursor:default">
      <span class="gs-stat-icon is-green" aria-hidden="true"><i class="bi bi-envelope"></i></span>
      <span class="gs-stat-copy">
        <small>Email</small>
        <strong>{{ $email }}</strong>
        <span>Mobile {{ $profile['mobileNumber'] ?? '—' }}</span>
      </span>
    </div>
    <div class="gs-stat" style="cursor:default">
      <span class="gs-stat-icon is-amber" aria-hidden="true"><i class="bi bi-shield-check"></i></span>
      <span class="gs-stat-copy">
        <small>KYC</small>
        <strong>{{ ucfirst($kycStatus) }}</strong>
        <span>Public key {{ $profile['publicKey'] ?? 'not provisioned' }}</span>
      </span>
    </div>
  </div>
  <div class="gs-help-links mt-3">
    <a href="{{ route('credentials.show') }}">
      <span class="gs-help-icon is-blue" aria-hidden="true"><i class="bi bi-key"></i></span>
      Manage credentials
      <i class="bi bi-chevron-right" aria-hidden="true"></i>
    </a>
    <a href="{{ route('compliance.show') }}">
      <span class="gs-help-icon is-green" aria-hidden="true"><i class="bi bi-shield-check"></i></span>
      Open compliance / KYC
      <i class="bi bi-chevron-right" aria-hidden="true"></i>
    </a>
  </div>
</div>
@endsection
