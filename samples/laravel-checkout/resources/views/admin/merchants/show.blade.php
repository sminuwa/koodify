@extends('layouts.admin')

@section('title', ($profile['displayName'] ?? 'Merchant').' · Admin')
@section('crumb', 'Merchants')

@php
  $sectionLabels = [
    'business' => 'Business details',
    'representative' => 'Company representative',
    'documents' => 'Documents',
    'declaration' => 'Declaration',
  ];
  $kycStatus = is_array($fullKyc) ? ($fullKyc['status'] ?? '') : (is_array($kyc) ? ($kyc['status'] ?? '') : '');
  $kycApproved = in_array($kycStatus, ['approved', 'active'], true);
@endphp

@section('content')
<div class="page-heading">
  <div>
    <h1>
      <span class="page-heading-mark" aria-hidden="true"><i class="bi bi-shop"></i></span>
      {{ $profile['displayName'] ?? 'Merchant' }}
    </h1>
    <p>
      #{{ $profile['id'] ?? '' }} · {{ $profile['email'] ?? '' }}
      · <span class="status-pill {{ in_array($profile['status'] ?? '', ['active', 'approved'], true) ? 'is-ok' : (($profile['status'] ?? '') === 'pending' ? 'is-wait' : 'is-bad') }}">{{ ucfirst($profile['status'] ?? '—') }}</span>
    </p>
  </div>
  <div class="page-heading-actions">
    <a class="btn btn-outline-secondary" href="{{ route('admin.merchants.index') }}">← Back</a>
    <form method="post" action="{{ route('admin.merchants.provision', $profile['id']) }}">
      @csrf
      <button class="btn btn-primary" type="submit">Retry provision</button>
    </form>
  </div>
</div>

<section class="admin-detail-grid">
  <div class="admin-detail-item"><small>Public key</small><b>{{ $profile['publicKey'] ?? 'Not provisioned' }}</b></div>
  <div class="admin-detail-item"><small>Has password</small><b>{{ !empty($profile['hasPassword']) ? 'Yes' : 'No' }}</b></div>
  <div class="admin-detail-item"><small>Zainbox</small><b>{{ data_get($profile, 'zainbox.zainboxCode') ?? data_get($profile, 'zainboxCode') ?? '—' }}</b></div>
  <div class="admin-detail-item"><small>Mobile</small><b>{{ $profile['mobileNumber'] ?? '—' }}</b></div>
</section>

<section class="surface merchant-panel" style="margin-bottom:18px">
  <div>
    <h2>Compliance sections</h2>
    <p class="lede">Approve each submitted section, or approve full KYC to provision the merchant.</p>
  </div>

  <div style="margin-top:18px">
    @foreach($sectionLabels as $key => $label)
      @php
        $row = $compliance[$key] ?? ['status' => 'none', 'data' => [], 'submittedAt' => null];
        $st = $row['status'] ?? 'none';
        $pill = $st === 'approved' ? 'is-ok' : ($st === 'pending' ? 'is-wait' : ($st === 'rejected' ? 'is-bad' : ''));
      @endphp
      <article class="admin-section-card">
        <div class="section-top">
          <div>
            <small style="display:block;color:var(--muted);font-weight:600;margin-bottom:4px">{{ strtoupper($key) }}</small>
            <strong>{{ $label }}</strong>
            <div class="lede" style="margin-top:4px;font-size:.88rem">
              @if(!empty($row['submittedAt']))
                Submitted {{ $row['submittedAt'] }}
              @else
                Not submitted
              @endif
            </div>
          </div>
          @if($st !== 'none')
            <span class="status-pill {{ $pill }}">{{ ucfirst($st) }}</span>
          @else
            <span class="status-pill">None</span>
          @endif
        </div>

        @if($st !== 'none')
          <div class="admin-actions">
            <form method="post" action="{{ route('admin.merchants.compliance', $profile['id']) }}">
              @csrf
              <input type="hidden" name="section" value="{{ $key }}">
              <input type="hidden" name="status" value="approved">
              <button class="btn btn-primary" type="submit" @disabled($st === 'approved')>Approve</button>
            </form>
            <form method="post" action="{{ route('admin.merchants.compliance', $profile['id']) }}">
              @csrf
              <input type="hidden" name="section" value="{{ $key }}">
              <input type="hidden" name="status" value="rejected">
              <button class="btn btn-outline-secondary" type="submit" @disabled($st === 'rejected')>Reject</button>
            </form>
            <form method="post" action="{{ route('admin.merchants.compliance', $profile['id']) }}">
              @csrf
              <input type="hidden" name="section" value="{{ $key }}">
              <input type="hidden" name="status" value="pending">
              <button class="btn btn-outline-secondary" type="submit" @disabled($st === 'pending')>Mark pending</button>
            </form>
          </div>
        @endif

        @if($key === 'representative' && !empty($row['data']) && is_array($row['data']))
          <div class="admin-detail-grid" style="margin-top:14px;margin-bottom:0">
            <div class="admin-detail-item"><small>Name</small><b>{{ trim(($row['data']['title'] ?? '').' '.($row['data']['firstName'] ?? '').' '.($row['data']['surname'] ?? '')) ?: '—' }}</b></div>
            <div class="admin-detail-item"><small>BVN</small><b>{{ $row['data']['bvn'] ?? '—' }}</b></div>
            <div class="admin-detail-item"><small>DOB</small><b>{{ $row['data']['dob'] ?? '—' }}</b></div>
            <div class="admin-detail-item"><small>Address</small><b>{{ trim(($row['data']['address'] ?? '').', '.($row['data']['state'] ?? ''), ' ,') ?: '—' }}</b></div>
          </div>
        @elseif($key === 'business' && !empty($row['data']) && is_array($row['data']))
          <div class="admin-detail-grid" style="margin-top:14px;margin-bottom:0">
            <div class="admin-detail-item"><small>Legal name</small><b>{{ $row['data']['legalName'] ?? $row['data']['businessName'] ?? '—' }}</b></div>
            <div class="admin-detail-item"><small>RC / BN</small><b>{{ $row['data']['registrationNumber'] ?? $row['data']['rcNumber'] ?? '—' }}</b></div>
            <div class="admin-detail-item"><small>Category</small><b>{{ $row['data']['category'] ?? $row['data']['businessCategory'] ?? '—' }}</b></div>
            <div class="admin-detail-item"><small>Website</small><b>{{ $row['data']['website'] ?? '—' }}</b></div>
          </div>
        @elseif($key === 'documents' && !empty($row['data']['files']) && is_array($row['data']['files']))
          <ul class="lede" style="margin:14px 0 0;padding-left:18px">
            @foreach($row['data']['files'] as $docType => $file)
              <li><b>{{ $docType }}</b> — {{ $file['name'] ?? 'file' }}@if(!empty($file['size'])) ({{ number_format((int) $file['size'] / 1024, 1) }} KB)@endif</li>
            @endforeach
          </ul>
        @endif
      </article>
    @endforeach
  </div>
</section>

<section class="surface merchant-panel" style="margin-bottom:18px">
  <div class="gs-panel-head">
    <div class="gs-panel-title">
      <span class="gs-panel-mark" aria-hidden="true"><i class="bi bi-shield-check"></i></span>
      <div>
        <h2>Full KYC decision</h2>
        <p class="lede" style="margin:4px 0 0">Approve provisions the zainbox + publishable key and emails the merchant. They mint <code>sk_…</code> from Credentials.</p>
      </div>
    </div>
    @if($kycStatus !== '')
      <span class="status-pill {{ $kycApproved ? 'is-ok' : ($kycStatus === 'pending' ? 'is-wait' : 'is-bad') }}">{{ ucfirst($kycStatus) }}</span>
    @endif
  </div>

  @if($fullKyc || $kyc)
    <div class="admin-actions">
      <form method="post" action="{{ route('admin.merchants.kyc', $profile['id']) }}">
        @csrf
        <input type="hidden" name="action" value="approved">
        <button class="btn btn-primary" type="submit" @disabled($kycApproved)>Approve KYC &amp; provision</button>
      </form>
      <form method="post" action="{{ route('admin.merchants.kyc', $profile['id']) }}">
        @csrf
        <input type="hidden" name="action" value="rejected">
        <button class="btn btn-outline-secondary" type="submit" @disabled($kycStatus === 'rejected')>Reject KYC</button>
      </form>
    </div>
  @else
    <div class="empty-state" style="padding:28px;text-align:center">
      <h3>No KYC submitted</h3>
      <p class="lede">Waiting for the merchant to complete Company Representative verification.</p>
    </div>
  @endif
</section>

@if(!empty($profile['credentials']) && is_array($profile['credentials']))
<section class="surface merchant-panel">
  <div>
    <h2>Credentials</h2>
    <p class="lede">Test / live key pairs for this merchant.</p>
  </div>
  <div class="table-wrap" style="margin-top:16px">
    <table class="table dash-table">
      <thead>
        <tr><th>ENV</th><th>PUBLIC KEY</th><th>SECRET</th><th>STATUS</th></tr>
      </thead>
      <tbody>
        @foreach($profile['credentials'] as $cred)
          <tr>
            <td><b>{{ strtoupper($cred['environment'] ?? '') }}</b></td>
            <td><code>{{ $cred['publicKey'] ?? '—' }}</code></td>
            <td>{{ !empty($cred['secretKeySet']) ? ($cred['secretKeyPrefix'] ?? 'set') : 'not set' }}</td>
            <td><span class="status-pill {{ ($cred['status'] ?? '') === 'active' ? 'is-ok' : 'is-wait' }}">{{ $cred['status'] ?? '—' }}</span></td>
          </tr>
        @endforeach
      </tbody>
    </table>
  </div>
</section>
@endif
@endsection
