@extends('layouts.merchant')

@section('title', 'Dashboard · Koodify')
@section('crumb', 'Dashboard')

@section('content')
@php
  $display = $profile['displayName'] ?? session('merchant.display_name', 'Merchant');
  $first = explode(' ', trim($display))[0] ?: 'there';
  $hour = (int) now()->format('G');
  $greeting = $hour < 12 ? 'Good morning' : ($hour < 17 ? 'Good afternoon' : 'Good evening');

  $range = request('range', 'daily');
  if (! in_array($range, ['daily', 'weekly', 'monthly'], true)) {
      $range = 'daily';
  }
  $rangeLabels = [
      'daily' => 'Last 14 days',
      'weekly' => 'Last 8 weeks',
      'monthly' => 'Last 6 months',
  ];
  $periodLabel = $rangeLabels[$range];

  $naira = function (float $amount): string {
      return '₦'.number_format($amount, 2);
  };

  // Sample presentational figures (transaction APIs not wired yet).
  $kpis = [
      ['label' => 'Total Collected', 'value' => $naira(2450000), 'change' => 12.6, 'icon' => 'bi-receipt', 'tone' => 'is-blue', 'spark' => 'in', 'color' => '#5b4dff'],
      ['label' => 'Total Payouts', 'value' => $naira(820000), 'change' => 4.2, 'icon' => 'bi-arrow-left-right', 'tone' => 'is-blue', 'spark' => 'out', 'color' => '#b7c3d6'],
      ['label' => 'Transactions', 'value' => number_format(1284), 'change' => 8.1, 'icon' => 'bi-file-earmark-text', 'tone' => 'is-violet', 'spark' => 'both', 'color' => '#7c3aed'],
      ['label' => 'Available to pay out', 'value' => $naira(1630000), 'change' => null, 'icon' => 'bi-wallet2', 'tone' => 'is-green', 'spark' => null, 'color' => null, 'note' => 'Waiting payout'],
  ];

  $points = match ($range) {
      'weekly' => [
          ['label' => 'W1', 'in' => 420000, 'out' => 180000],
          ['label' => 'W2', 'in' => 510000, 'out' => 210000],
          ['label' => 'W3', 'in' => 390000, 'out' => 150000],
          ['label' => 'W4', 'in' => 640000, 'out' => 260000],
          ['label' => 'W5', 'in' => 580000, 'out' => 240000],
          ['label' => 'W6', 'in' => 710000, 'out' => 300000],
          ['label' => 'W7', 'in' => 680000, 'out' => 280000],
          ['label' => 'W8', 'in' => 760000, 'out' => 310000],
      ],
      'monthly' => [
          ['label' => 'Jun', 'in' => 1800000, 'out' => 720000],
          ['label' => 'Jul', 'in' => 2100000, 'out' => 800000],
          ['label' => 'Aug', 'in' => 1950000, 'out' => 750000],
          ['label' => 'Sep', 'in' => 2400000, 'out' => 910000],
          ['label' => 'Oct', 'in' => 2650000, 'out' => 980000],
          ['label' => 'Nov', 'in' => 2450000, 'out' => 820000],
      ],
      default => [
          ['label' => 'Oct 25', 'in' => 120000, 'out' => 45000],
          ['label' => 'Oct 26', 'in' => 98000, 'out' => 32000],
          ['label' => 'Oct 27', 'in' => 150000, 'out' => 61000],
          ['label' => 'Oct 28', 'in' => 132000, 'out' => 40000],
          ['label' => 'Oct 29', 'in' => 176000, 'out' => 72000],
          ['label' => 'Oct 30', 'in' => 160000, 'out' => 55000],
          ['label' => 'Oct 31', 'in' => 210000, 'out' => 88000],
          ['label' => 'Nov 01', 'in' => 188000, 'out' => 64000],
          ['label' => 'Nov 02', 'in' => 205000, 'out' => 70000],
          ['label' => 'Nov 03', 'in' => 240000, 'out' => 91000],
          ['label' => 'Nov 04', 'in' => 198000, 'out' => 60000],
          ['label' => 'Nov 05', 'in' => 225000, 'out' => 78000],
          ['label' => 'Nov 06', 'in' => 260000, 'out' => 95000],
          ['label' => 'Nov 07', 'in' => 275000, 'out' => 102000],
      ],
  };
  $moneyIn = array_sum(array_column($points, 'in'));
  $moneyOut = array_sum(array_column($points, 'out'));

  $boxes = [
      ['name' => 'Main collections', 'codeName' => 'main', 'collected' => 1250000],
      ['name' => 'Checkout', 'codeName' => 'checkout', 'collected' => 680000],
      ['name' => 'Virtual accounts', 'codeName' => 'va', 'collected' => 420000],
      ['name' => 'Transfers', 'codeName' => 'transfers', 'collected' => 100000],
  ];
  $peak = max(1, max(array_column($boxes, 'collected')));

  $transactions = [
      ['reference' => 'TXN-839201', 'zainbox' => 'Checkout', 'sender' => 'Amaka Adeyemi', 'type' => 'Payment', 'amount' => 245000, 'out' => false, 'date' => now(), 'status' => 'Successful', 'tone' => 'is-success'],
      ['reference' => 'TXN-839200', 'zainbox' => 'Main collections', 'sender' => 'GTBank transfer', 'type' => 'Bank transfer', 'amount' => 82500, 'out' => false, 'date' => now(), 'status' => 'Pending', 'tone' => 'is-pending'],
      ['reference' => 'TXN-839199', 'zainbox' => 'Virtual accounts', 'sender' => 'Amina Mart', 'type' => 'Virtual account', 'amount' => 1250000, 'out' => false, 'date' => now()->subDay(), 'status' => 'Successful', 'tone' => 'is-success'],
      ['reference' => 'TXN-839198', 'zainbox' => 'Transfers', 'sender' => 'Payroll', 'type' => 'Transfer', 'amount' => 36700, 'out' => true, 'date' => now()->subDay(), 'status' => 'Failed', 'tone' => 'is-failed'],
      ['reference' => 'TXN-839197', 'zainbox' => 'Checkout', 'sender' => 'FreshCart', 'type' => 'Payment', 'amount' => 18400, 'out' => false, 'date' => now()->subDays(2), 'status' => 'Successful', 'tone' => 'is-success'],
  ];
@endphp

<section class="ops-dashboard" id="opsDashboard">
  <header class="dash-hero">
    <div>
      <h2>
        <span class="page-heading-mark" aria-hidden="true"><i class="bi bi-grid"></i></span>
        <span>{{ $greeting }}, {{ $first }} <span class="dash-wave" aria-hidden="true">👋</span></span>
      </h2>
      <p>Here's what's happening with your payments today.</p>
    </div>
    <div class="dropdown">
      <button class="range-pill" type="button" data-bs-toggle="dropdown" aria-expanded="false">
        <i class="bi bi-calendar3" aria-hidden="true"></i>
        <span>{{ $periodLabel }}</span>
        <i class="bi bi-chevron-down" aria-hidden="true"></i>
      </button>
      <ul class="dropdown-menu dropdown-menu-end range-menu">
        <li><a class="dropdown-item" href="{{ route('dashboard', ['range' => 'daily']) }}">Last 14 days</a></li>
        <li><a class="dropdown-item" href="{{ route('dashboard', ['range' => 'weekly']) }}">Last 8 weeks</a></li>
        <li><a class="dropdown-item" href="{{ route('dashboard', ['range' => 'monthly']) }}">Last 6 months</a></li>
      </ul>
    </div>
  </header>

  <div class="kpi-row">
    <div class="dashboard-figures">
      @foreach ($kpis as $kpi)
        <article class="gs-stat">
          <span class="gs-stat-icon {{ $kpi['tone'] }}" aria-hidden="true"><i class="bi {{ $kpi['icon'] }}"></i></span>
          <span class="gs-stat-copy">
            <small>{{ $kpi['label'] }}</small>
            <strong>{{ $kpi['value'] }}</strong>
            <span>
              @if ($kpi['change'] === null)
                <span class="kpi-note">{{ $kpi['note'] ?? 'vs previous period' }}</span>
              @elseif ($kpi['change'] > 0)
                <span class="kpi-delta is-up">↑ {{ number_format($kpi['change'], 1) }}%</span>
              @elseif ($kpi['change'] < 0)
                <span class="kpi-delta is-down">↓ {{ number_format(abs($kpi['change']), 1) }}%</span>
              @else
                <span class="kpi-delta is-flat">0.0%</span>
              @endif
            </span>
          </span>
          @if ($kpi['spark'])
            <span class="kpi-spark" data-spark="{{ $kpi['spark'] }}" data-spark-color="{{ $kpi['color'] }}" aria-hidden="true"></span>
          @endif
        </article>
      @endforeach
    </div>
  </div>

  <div class="dash-split">
    <section class="gs-panel">
      <div class="gs-panel-head">
        <div class="gs-panel-title">
          <span class="gs-panel-mark" aria-hidden="true"><i class="bi bi-bar-chart-line"></i></span>
          <div>
            <h2>Money Movement</h2>
            <p>Daily collection and payout trend</p>
          </div>
        </div>
        <div class="dropdown">
          <button class="range-pill" type="button" data-bs-toggle="dropdown" aria-expanded="false">
            <span>{{ $periodLabel }}</span>
            <i class="bi bi-chevron-down" aria-hidden="true"></i>
          </button>
          <ul class="dropdown-menu dropdown-menu-end range-menu">
            <li><a class="dropdown-item" href="{{ route('dashboard', ['range' => 'daily']) }}">Last 14 days</a></li>
            <li><a class="dropdown-item" href="{{ route('dashboard', ['range' => 'weekly']) }}">Last 8 weeks</a></li>
            <li><a class="dropdown-item" href="{{ route('dashboard', ['range' => 'monthly']) }}">Last 6 months</a></li>
          </ul>
        </div>
      </div>
      <div class="flow-summary">
        <div>
          <span>Money In</span>
          <strong>{{ $naira($moneyIn) }}</strong>
        </div>
        <div>
          <span>Money Out</span>
          <strong>{{ $naira($moneyOut) }}</strong>
        </div>
        <div class="flow-legend">
          <span><i class="is-in"></i> Money In</span>
          <span><i class="is-out"></i> Money Out</span>
        </div>
      </div>
      <div class="chart-frame">
        <canvas id="dashboardFlow" aria-label="Money movement for {{ $periodLabel }}" data-series='@json($points)'></canvas>
      </div>
    </section>

    <section class="gs-panel">
      <div class="gs-panel-head">
        <div class="gs-panel-title">
          <span class="gs-panel-mark" aria-hidden="true"><i class="bi bi-box"></i></span>
          <div>
            <h2>Collections by service</h2>
            <p>Top performing channels</p>
          </div>
        </div>
        <a class="menu-link" href="{{ route('payment-simulation') }}">View tools</a>
      </div>
      <ul class="box-bars">
        @foreach ($boxes as $row)
          <li>
            <span class="box-bars-name">{{ $row['name'] }}</span>
            <span class="box-bars-track" aria-hidden="true"><span style="width: {{ max(4, ($row['collected'] / $peak) * 100) }}%"></span></span>
            <strong>{{ $naira($row['collected']) }}</strong>
          </li>
        @endforeach
      </ul>
    </section>
  </div>

  <div class="dash-full">
    <section class="gs-panel">
      <div class="gs-panel-head">
        <div class="gs-panel-title">
          <span class="gs-panel-mark" aria-hidden="true"><i class="bi bi-arrow-left-right"></i></span>
          <div>
            <h2>Recent Transactions</h2>
            <p>Latest sample activity across your merchant tools</p>
          </div>
        </div>
        <a class="menu-link" href="{{ route('transactions') }}">View all</a>
      </div>
      <div class="table-wrap">
        <table class="dash-table">
          <thead>
            <tr>
              <th scope="col">Reference</th>
              <th scope="col">Channel</th>
              <th scope="col">Customer / Source</th>
              <th scope="col">Amount</th>
              <th scope="col">Date</th>
              <th scope="col">Status</th>
            </tr>
          </thead>
          <tbody>
            @foreach ($transactions as $transaction)
              <tr>
                <td class="dash-ref">{{ $transaction['reference'] }}</td>
                <td>{{ $transaction['zainbox'] }}</td>
                <td>
                  <b>{{ $transaction['sender'] }}</b>
                  <small class="tx-ref">{{ $transaction['type'] }}</small>
                </td>
                <td class="{{ $transaction['out'] ? 'amount-out' : 'amount-in' }}">{{ $naira($transaction['amount']) }}</td>
                <td>
                  <b class="dash-when">{{ $transaction['date']->format('M j, Y') }}</b>
                  <small class="tx-ref">{{ $transaction['date']->format('H:i') }}</small>
                </td>
                <td><span class="status-pill {{ $transaction['tone'] }}">{{ $transaction['status'] }}</span></td>
              </tr>
            @endforeach
          </tbody>
        </table>
      </div>
    </section>
  </div>
</section>
@endsection
