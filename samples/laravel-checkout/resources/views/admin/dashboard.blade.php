@extends('layouts.admin')

@section('title', 'Admin overview')
@section('crumb', 'Overview')

@php
  $first = explode(' ', trim($adminName))[0] ?? 'Admin';
  $hour = (int) now()->format('G');
  $greeting = $hour < 12 ? 'Good morning' : ($hour < 17 ? 'Good afternoon' : 'Good evening');
@endphp

@section('content')
<section class="ops-dashboard">
  <header class="dash-hero">
    <div>
      <h2>
        <span class="page-heading-mark" aria-hidden="true"><i class="bi bi-grid"></i></span>
        <span>{{ $greeting }}, {{ $first }}</span>
      </h2>
      <p>Review pending KYC, approve compliance sections, and provision merchant accounts.</p>
    </div>
    <a class="btn btn-outline-secondary" href="{{ route('admin.merchants.index') }}">All merchants</a>
  </header>

  @if($apiError)
    <div class="flash err">Could not reach Koodify admin APIs: {{ $apiError }}</div>
  @endif

  <div class="kpi-row">
    <div class="dashboard-figures">
      <article class="gs-stat">
        <span class="gs-stat-icon is-violet" aria-hidden="true"><i class="bi bi-shop"></i></span>
        <span class="gs-stat-copy">
          <small>Merchants</small>
          <strong>{{ number_format($merchantCount) }}</strong>
          <span><span class="kpi-note">Registry accounts</span></span>
        </span>
      </article>
      <article class="gs-stat">
        <span class="gs-stat-icon is-blue" aria-hidden="true"><i class="bi bi-hourglass-split"></i></span>
        <span class="gs-stat-copy">
          <small>Pending review</small>
          <strong>{{ number_format($pendingKyc) }}</strong>
          <span><span class="kpi-note">KYC or sections pending</span></span>
        </span>
      </article>
      <article class="gs-stat">
        <span class="gs-stat-icon is-green" aria-hidden="true"><i class="bi bi-shield-check"></i></span>
        <span class="gs-stat-copy">
          <small>Approved KYC</small>
          <strong>{{ number_format($approvedKyc) }}</strong>
          <span><span class="kpi-note">Ready or provisioned</span></span>
        </span>
      </article>
      <article class="gs-stat">
        <span class="gs-stat-icon is-blue" aria-hidden="true"><i class="bi bi-arrow-repeat"></i></span>
        <span class="gs-stat-copy">
          <small>Awaiting provision</small>
          <strong>{{ number_format($awaitingProvision) }}</strong>
          <span><span class="kpi-note">{{ number_format($pendingSections) }} open sections</span></span>
        </span>
      </article>
    </div>
  </div>

  <section class="gs-panel">
    <div class="gs-panel-head">
      <div class="gs-panel-title">
        <span class="gs-panel-mark" aria-hidden="true"><i class="bi bi-list-check"></i></span>
        <div>
          <h2>Pending review queue</h2>
          <p>Merchants with pending KYC or compliance sections</p>
        </div>
      </div>
      <a class="btn btn-outline-secondary" href="{{ route('admin.merchants.index') }}">View all</a>
    </div>

    <div class="table-wrap">
      <table class="table dash-table">
        <thead>
          <tr>
            <th>ID</th>
            <th>Merchant</th>
            <th>Email</th>
            <th>KYC</th>
            <th>Sections</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          @forelse($pendingQueue as $c)
            @php
              $name = $c['displayName'] ?? 'Merchant';
              $kst = $c['kycStatus'] ?? 'none';
              $pill = in_array($kst, ['approved', 'active'], true) ? 'is-ok' : ($kst === 'pending' ? 'is-wait' : 'is-bad');
            @endphp
            <tr>
              <td><b>#{{ $c['id'] ?? '—' }}</b></td>
              <td>{{ $name }}</td>
              <td>{{ $c['email'] ?? '—' }}</td>
              <td><span class="status-pill {{ $pill }}">{{ ucfirst($kst) }}</span></td>
              <td>{{ (int) ($c['pendingSections'] ?? 0) }} pending</td>
              <td><a href="{{ route('admin.merchants.show', $c['id']) }}">Review <i class="bi bi-arrow-right"></i></a></td>
            </tr>
          @empty
            <tr>
              <td colspan="6">
                <div class="empty-state" style="padding:28px;text-align:center">
                  <h3>Queue clear</h3>
                  <p class="lede">No merchants waiting on KYC or compliance review.</p>
                </div>
              </td>
            </tr>
          @endforelse
        </tbody>
      </table>
    </div>
  </section>
</section>
@endsection
