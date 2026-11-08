@extends('layouts.admin')

@section('title', 'Koodify Admin Portal')
@section('breadcrumb', 'Overview')

@section('content')
@php
  $first = explode(' ', trim($adminName))[0] ?? 'Admin';
  $merchantLabel = $merchantCount === null ? '6,842' : number_format($merchantCount);
  $merchantDelta = $merchantCount === null ? '↗ 8.1%' : '↗ live';
  $merchantSub = $merchantCount === null ? 'vs. previous month' : 'from Koodify registry';
@endphp
<div class="page-heading">
  <div>
    <span class="card-kicker">{{ strtoupper(now()->format('l, F j, Y')) }}</span>
    <h1 id="pageTitle">Overview</h1>
    <p class="muted">Here's what's happening across your platform today.</p>
  </div>
  <div class="heading-actions">
    <button class="outline-button" type="button" id="exportBtn">↓ Export report</button>
    <button class="date-button" type="button">▦ <span>Last 30 days</span>⌄</button>
  </div>
</div>

@if($apiError)
  <div class="flash err">Could not reach Koodify admin APIs: {{ $apiError }}</div>
@endif

<div class="welcome-strip">
  <div>
    <b id="welcomeName">Welcome back, {{ $first }}</b>
    <p>Your platform is looking healthy. Here's your latest performance snapshot.</p>
  </div>
  <div class="welcome-art">✦</div>
</div>

<section class="stat-grid">
  <article class="stat-card">
    <div class="stat-top"><span>Total transactions</span><span class="stat-icon blue">⇄</span></div>
    <strong>1,248,320</strong>
    <div class="stat-foot"><span class="positive">↗ 12.6%</span><span>vs. previous month</span></div>
    <div class="sparkline spark-blue"><i></i><i></i><i></i><i></i><i></i><i></i><i></i><i></i></div>
  </article>
  <article class="stat-card">
    <div class="stat-top"><span>Transaction value</span><span class="stat-icon green">₦</span></div>
    <strong>₦2.45B</strong>
    <div class="stat-foot"><span class="positive">↗ 18.4%</span><span>vs. previous month</span></div>
    <div class="sparkline spark-green"><i></i><i></i><i></i><i></i><i></i><i></i><i></i><i></i></div>
  </article>
  <article class="stat-card">
    <div class="stat-top"><span>Active merchants</span><span class="stat-icon violet">♙</span></div>
    <strong>{{ $merchantLabel }}</strong>
    <div class="stat-foot"><span class="positive">{{ $merchantDelta }}</span><span>{{ $merchantSub }}</span></div>
    <div class="sparkline spark-violet"><i></i><i></i><i></i><i></i><i></i><i></i><i></i><i></i></div>
  </article>
  <article class="stat-card">
    <div class="stat-top"><span>Active customers</span><span class="stat-icon orange">♧</span></div>
    <strong>428,725</strong>
    <div class="stat-foot"><span class="positive">↗ 11.9%</span><span>{{ $pendingKyc > 0 ? number_format($pendingKyc).' merchants pending KYC' : 'vs. previous month' }}</span></div>
    <div class="sparkline spark-orange"><i></i><i></i><i></i><i></i><i></i><i></i><i></i><i></i></div>
  </article>
</section>

<section class="analytics-grid">
  <article class="surface chart-card">
    <div class="surface-heading">
      <div>
        <h3>Transaction volume</h3>
        <p class="muted">Daily transaction performance</p>
      </div>
      <div class="segmented">
        <button class="selected" type="button" data-chart="volume">Volume</button>
        <button type="button" data-chart="count">Count</button>
      </div>
    </div>
    <div class="chart-summary"><strong id="chartTotal">₦2,457,930,000</strong><span class="positive">↗ 18.4%</span></div>
    <div class="chart-wrap">
      <div class="y-labels"><span>₦250M</span><span>₦200M</span><span>₦150M</span><span>₦100M</span><span>₦50M</span><span>₦0</span></div>
      <canvas id="volumeChart" aria-label="Transaction volume chart"></canvas>
    </div>
    <div class="x-labels"><span>Oct 09</span><span>Oct 14</span><span>Oct 19</span><span>Oct 24</span><span>Oct 29</span><span>Nov 07</span></div>
  </article>
  <article class="surface service-card">
    <div class="surface-heading">
      <div>
        <h3>Transactions by service</h3>
        <p class="muted">Share of total transactions</p>
      </div>
      <button class="dots-button" type="button">•••</button>
    </div>
    <div class="donut-layout">
      <div class="donut"><div><b>1.25M</b><span>Total</span></div></div>
      <div class="legend">
        <div><i class="legend-purple"></i><span>Payments</span><b>56.2%</b></div>
        <div><i class="legend-blue"></i><span>Transfers</span><b>18.7%</b></div>
        <div><i class="legend-green"></i><span>Virtual accounts</span><b>12.4%</b></div>
        <div><i class="legend-orange"></i><span>Airtime & data</span><b>7.8%</b></div>
        <div><i class="legend-pink"></i><span>Other services</span><b>4.9%</b></div>
      </div>
    </div>
  </article>
</section>

<section class="surface transactions-card">
  <div class="surface-heading">
    <div>
      <h3>Recent transactions</h3>
      <p class="muted">A snapshot of the latest activity on your platform.</p>
    </div>
    <a class="text-button" href="{{ route('admin.section', 'transactions') }}">View all transactions →</a>
  </div>
  <div class="table-scroll">
    <table>
      <thead>
        <tr><th>REFERENCE</th><th>MERCHANT</th><th>SERVICE</th><th>DATE</th><th>AMOUNT</th><th>STATUS</th></tr>
      </thead>
      <tbody>
        <tr><td><b>TXN-839201</b></td><td><span class="merchant-avatar ma-purple">SB</span> Swift Basket</td><td>Payment</td><td>{{ now()->format('M d, Y') }}</td><td><b>₦245,000.00</b></td><td><span class="status success">● Successful</span></td></tr>
        <tr><td><b>TXN-839200</b></td><td><span class="merchant-avatar ma-blue">NG</span> Nova Goods</td><td>Transfer</td><td>{{ now()->format('M d, Y') }}</td><td><b>₦82,500.00</b></td><td><span class="status pending">● Pending</span></td></tr>
        <tr><td><b>TXN-839199</b></td><td><span class="merchant-avatar ma-orange">AM</span> Amina Mart</td><td>Virtual account</td><td>{{ now()->subDay()->format('M d, Y') }}</td><td><b>₦1,250,000.00</b></td><td><span class="status success">● Successful</span></td></tr>
        <tr><td><b>TXN-839198</b></td><td><span class="merchant-avatar ma-green">FC</span> FreshCart</td><td>Payment</td><td>{{ now()->subDay()->format('M d, Y') }}</td><td><b>₦36,700.00</b></td><td><span class="status failed">● Failed</span></td></tr>
        @foreach(array_slice($recentCustomers, 0, 2) as $c)
          @php
            $name = $c['displayName'] ?? $c['display_name'] ?? 'Merchant';
            $parts = preg_split('/\s+/', trim($name)) ?: [];
            $av = collect($parts)->take(2)->map(fn ($p) => mb_strtoupper(mb_substr($p, 0, 1)))->implode('') ?: 'M';
            $colors = ['ma-purple', 'ma-blue', 'ma-orange', 'ma-green'];
            $color = $colors[($c['id'] ?? 0) % 4];
            $st = $c['status'] ?? 'pending';
          @endphp
          <tr>
            <td><b>MCH-{{ $c['id'] ?? '—' }}</b></td>
            <td><span class="merchant-avatar {{ $color }}">{{ $av }}</span> {{ $name }}</td>
            <td>Onboarding</td>
            <td>{{ now()->format('M d, Y') }}</td>
            <td><b>—</b></td>
            <td><span class="status {{ $st === 'active' || $st === 'approved' ? 'success' : ($st === 'pending' ? 'pending' : 'failed') }}">● {{ ucfirst($st) }}</span></td>
          </tr>
        @endforeach
      </tbody>
    </table>
  </div>
</section>
@endsection

@push('scripts')
<script src="{{ asset('admin/js/dashboard-chart.js') }}"></script>
@endpush
