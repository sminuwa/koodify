@extends('layouts.admin')

@section('title', ($profile['displayName'] ?? 'Merchant').' · Admin')
@section('breadcrumb', 'Merchants')

@section('content')
<div class="page-heading">
  <div>
    <span class="card-kicker">MERCHANT #{{ $profile['id'] ?? '' }}</span>
    <h1>{{ $profile['displayName'] ?? 'Merchant' }}</h1>
    <p class="muted">{{ $profile['email'] ?? '' }} · status <span class="badge {{ $profile['status'] ?? '' }}">{{ $profile['status'] ?? '—' }}</span></p>
  </div>
  <div class="heading-actions">
    <a class="outline-button" href="{{ route('admin.merchants.index') }}">← Back</a>
    <form method="post" action="{{ route('admin.merchants.provision', $profile['id']) }}">
      @csrf
      <button class="primary-button" type="submit" style="padding:10px 16px"><span>Retry provision</span></button>
    </form>
  </div>
</div>

<section class="detail-grid">
  <div class="detail-item"><small>Public key</small><b>{{ $profile['publicKey'] ?? 'Not provisioned' }}</b></div>
  <div class="detail-item"><small>Has password</small><b>{{ !empty($profile['hasPassword']) ? 'Yes' : 'No' }}</b></div>
  <div class="detail-item"><small>Zainbox</small><b>{{ data_get($profile, 'zainbox.zainboxCode') ?? data_get($profile, 'zainboxCode') ?? '—' }}</b></div>
  <div class="detail-item"><small>Mobile</small><b>{{ $profile['mobileNumber'] ?? '—' }}</b></div>
</section>

<section class="surface" style="padding:20px;margin-bottom:18px">
  <div class="surface-heading">
    <div>
      <h3>KYC</h3>
      <p class="muted">Approve to provision the merchant zainbox and publishable key. The merchant then generates secret keys from Credentials.</p>
    </div>
    @if($kyc)
      @php $kst = $kyc['status'] ?? ''; @endphp
      <span class="status {{ $kst === 'approved' || $kst === 'active' ? 'success' : ($kst === 'pending' ? 'pending' : 'failed') }}">● {{ ucfirst($kst ?: '—') }}</span>
    @endif
  </div>

  @if($fullKyc)
    <div class="detail-grid" style="margin:16px 0">
      <div class="detail-item"><small>Name</small><b>{{ ($fullKyc['title'] ?? '').' '.($fullKyc['firstName'] ?? '').' '.($fullKyc['surname'] ?? '') }}</b></div>
      <div class="detail-item"><small>BVN</small><b>{{ $fullKyc['bvn'] ?? '—' }}</b></div>
      <div class="detail-item"><small>DOB</small><b>{{ $fullKyc['dob'] ?? '—' }}</b></div>
      <div class="detail-item"><small>Address</small><b>{{ ($fullKyc['address'] ?? '').', '.($fullKyc['state'] ?? '') }}</b></div>
    </div>
    <div class="actions-inline">
      <form method="post" action="{{ route('admin.merchants.kyc', $profile['id']) }}">
        @csrf
        <input type="hidden" name="kycId" value="{{ $fullKyc['id'] }}">
        <input type="hidden" name="status" value="approved">
        <button class="primary-button" type="submit" style="padding:10px 16px"><span>Approve KYC</span></button>
      </form>
      <form method="post" action="{{ route('admin.merchants.kyc', $profile['id']) }}">
        @csrf
        <input type="hidden" name="kycId" value="{{ $fullKyc['id'] }}">
        <input type="hidden" name="status" value="rejected">
        <button class="outline-button" type="submit">Reject</button>
      </form>
      <form method="post" action="{{ route('admin.merchants.kyc', $profile['id']) }}">
        @csrf
        <input type="hidden" name="kycId" value="{{ $fullKyc['id'] }}">
        <input type="hidden" name="status" value="pending">
        <button class="outline-button" type="submit">Mark pending</button>
      </form>
    </div>
  @elseif($kyc)
    <p class="muted">Summary only — full KYC row not loaded. Status: <b>{{ $kyc['status'] ?? '—' }}</b></p>
  @else
    <div class="empty-state" style="padding:28px">
      <h3>No KYC submitted</h3>
      <p>Waiting for the merchant to complete verification.</p>
    </div>
  @endif
</section>

@if(!empty($profile['credentials']) && is_array($profile['credentials']))
<section class="surface transactions-card">
  <div class="surface-heading"><div><h3>Credentials</h3><p class="muted">Test / live key pairs for this merchant.</p></div></div>
  <div class="table-scroll">
    <table class="admin-table">
      <thead><tr><th>ENV</th><th>PUBLIC KEY</th><th>SECRET</th><th>STATUS</th></tr></thead>
      <tbody>
        @foreach($profile['credentials'] as $cred)
          <tr>
            <td><b>{{ strtoupper($cred['environment'] ?? '') }}</b></td>
            <td><code>{{ $cred['publicKey'] ?? '—' }}</code></td>
            <td>{{ !empty($cred['secretKeySet']) ? ($cred['secretKeyPrefix'] ?? 'set') : 'not set' }}</td>
            <td><span class="badge {{ $cred['status'] ?? '' }}">{{ $cred['status'] ?? '—' }}</span></td>
          </tr>
        @endforeach
      </tbody>
    </table>
  </div>
</section>
@endif
@endsection
