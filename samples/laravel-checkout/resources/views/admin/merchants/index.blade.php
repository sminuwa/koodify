@extends('layouts.admin')

@section('title', 'Merchants')
@section('crumb', 'Merchants')

@section('content')
<div class="page-heading">
  <div>
    <h1>
      <span class="page-heading-mark" aria-hidden="true"><i class="bi bi-shop"></i></span>
      Merchants
    </h1>
    <p>Review registry accounts, compliance sections, and KYC decisions.</p>
  </div>
  <div class="page-heading-actions">
    <a class="btn btn-outline-secondary" href="{{ route('admin.dashboard') }}">← Overview</a>
  </div>
</div>

@if($error)
  <div class="flash err">{{ $error }}</div>
@endif

<section class="surface merchant-panel">
  <div class="gs-panel-head" style="margin-bottom:16px">
    <div class="gs-panel-title">
      <span class="gs-panel-mark" aria-hidden="true"><i class="bi bi-people"></i></span>
      <div>
        <h2>All merchants</h2>
        <p class="lede" style="margin:4px 0 0">{{ count($customers) }} account{{ count($customers) === 1 ? '' : 's' }} in the registry.</p>
      </div>
    </div>
  </div>

  @if(count($customers))
    <div class="table-wrap">
      <table class="table dash-table">
        <thead>
          <tr>
            <th>ID</th>
            <th>Merchant</th>
            <th>Email</th>
            <th>KYC</th>
            <th>Sections</th>
            <th>Status</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          @foreach($customers as $c)
            @php
              $name = $c['displayName'] ?? $c['display_name'] ?? 'Merchant';
              $st = $c['status'] ?? '—';
              $kst = $c['kycStatus'] ?? 'none';
              $pending = (int) ($c['pendingSections'] ?? 0);
              $kycPill = in_array($kst, ['approved', 'active'], true) ? 'is-ok' : ($kst === 'pending' ? 'is-wait' : 'is-bad');
              $stPill = in_array($st, ['active', 'approved'], true) ? 'is-ok' : ($st === 'pending' ? 'is-wait' : 'is-bad');
            @endphp
            <tr>
              <td><b>#{{ $c['id'] ?? '—' }}</b></td>
              <td>{{ $name }}</td>
              <td>{{ $c['email'] ?? '—' }}</td>
              <td><span class="status-pill {{ $kycPill }}">{{ ucfirst($kst) }}</span></td>
              <td>{{ $pending }} pending</td>
              <td><span class="status-pill {{ $stPill }}">{{ ucfirst($st) }}</span></td>
              <td><a href="{{ route('admin.merchants.show', $c['id']) }}">Review <i class="bi bi-arrow-right"></i></a></td>
            </tr>
          @endforeach
        </tbody>
      </table>
    </div>
  @else
    <div class="empty-state" style="padding:36px;text-align:center">
      <h3>No merchants found</h3>
      <p class="lede">Have a business register at <a href="{{ route('register') }}">/register</a>.</p>
    </div>
  @endif
</section>
@endsection
