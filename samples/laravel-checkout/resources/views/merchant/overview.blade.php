@extends('layouts.merchant')

@section('title', 'Dashboard · Koodify')
@section('crumb', 'Dashboard')

@section('content')
@php
  $display = $profile['displayName'] ?? session('merchant.display_name', 'Merchant');
  $first = explode(' ', trim($display))[0] ?: 'there';
  $hour = (int) now()->format('G');
  $greeting = $hour < 12 ? 'Good morning' : ($hour < 17 ? 'Good afternoon' : 'Good evening');

  $range = $range ?? 'daily';
  $rangeLabels = [
      'daily' => 'Last 14 days',
      'weekly' => 'Last 8 weeks',
      'monthly' => 'Last 6 months',
  ];
  $periodLabel = $rangeLabels[$range] ?? 'Last 14 days';

  $naira = function (float $amount): string {
      return '₦'.number_format($amount, 2);
  };

  $insights = is_array($insights ?? null) ? $insights : [];
  $totalCollected = (float) ($insights['totalCollected'] ?? 0);
  $totalPayouts = (float) ($insights['totalPayouts'] ?? 0);
  $transactionCount = (int) ($insights['transactionCount'] ?? 0);
  $availableBalance = (float) ($insights['availableBalance'] ?? 0);
  $points = is_array($insights['points'] ?? null) ? $insights['points'] : [];
  $boxes = is_array($insights['boxes'] ?? null) ? $insights['boxes'] : [];
  $transactions = is_array($insights['transactions'] ?? null) ? $insights['transactions'] : [];
  $peak = max(1.0, (float) (max(array_column($boxes, 'collected') ?: [0]) ?: 0));
  $moneyIn = array_sum(array_column($points, 'in'));
  $moneyOut = array_sum(array_column($points, 'out'));
  $configured = (bool) ($configured ?? false);
  $emptyReason = $emptyReason ?? null;

  $kpis = [
      [
          'label' => 'Total Collected',
          'value' => $configured ? $naira($totalCollected) : '—',
          'change' => $configured ? null : null,
          'icon' => 'bi-receipt',
          'tone' => 'is-blue',
          'spark' => $configured ? 'in' : null,
          'color' => '#5b4dff',
          'note' => $configured ? 'In selected period' : 'Complete setup to unlock',
      ],
      [
          'label' => 'Total Payouts',
          'value' => $configured ? $naira($totalPayouts) : '—',
          'change' => null,
          'icon' => 'bi-arrow-left-right',
          'tone' => 'is-blue',
          'spark' => $configured ? 'out' : null,
          'color' => '#b7c3d6',
          'note' => $configured ? 'Outflows in period' : 'Complete setup to unlock',
      ],
      [
          'label' => 'Transactions',
          'value' => $configured ? number_format($transactionCount) : '—',
          'change' => null,
          'icon' => 'bi-file-earmark-text',
          'tone' => 'is-violet',
          'spark' => $configured ? 'both' : null,
          'color' => '#7c3aed',
          'note' => $configured ? 'In selected period' : 'Complete setup to unlock',
      ],
      [
          'label' => 'Available to pay out',
          'value' => $configured ? $naira($availableBalance) : '—',
          'change' => null,
          'icon' => 'bi-wallet2',
          'tone' => 'is-green',
          'spark' => null,
          'color' => null,
          'note' => $configured ? 'Wallet balance' : 'Complete setup to unlock',
      ],
  ];

  $emptyCopy = match ($emptyReason) {
      'issue_secret' => [
          'title' => 'Issue your secret key',
          'body' => 'Your publishable key is ready. Generate an sk_… from Credentials so the dashboard can load live collections and payouts.',
          'cta' => 'Open Credentials',
          'href' => route('credentials.show'),
      ],
      'awaiting_provision' => [
          'title' => 'Waiting for account provisioning',
          'body' => 'Compliance is in review or approved. Once your publishable key is provisioned, issue a secret key and this dashboard will fill with live activity.',
          'cta' => 'Check Credentials',
          'href' => route('credentials.show'),
      ],
      default => [
          'title' => 'Complete compliance to unlock your dashboard',
          'body' => 'Finish KYC / compliance, wait for admin approval, then issue API keys. Until then these cards stay empty.',
          'cta' => 'Continue compliance',
          'href' => route('compliance.show'),
      ],
  };
@endphp

<section class="ops-dashboard" id="opsDashboard">
  <header class="dash-hero">
    <div>
      <h2>
        <span class="page-heading-mark" aria-hidden="true"><i class="bi bi-grid"></i></span>
        <span>{{ $greeting }}, {{ $first }} <span class="dash-wave" aria-hidden="true">👋</span></span>
      </h2>
      <p>
        @if ($configured)
          Live activity for {{ $zainboxCode ?: 'your merchant account' }} · {{ $periodLabel }}.
        @else
          Finish setup to load collections, payouts, and recent activity.
        @endif
      </p>
    </div>
    <div class="dropdown">
      <button class="range-pill" type="button" data-bs-toggle="dropdown" aria-expanded="false" @disabled(! $configured)>
        <i class="bi bi-calendar3" aria-hidden="true"></i>
        <span>{{ $periodLabel }}</span>
        <i class="bi bi-chevron-down" aria-hidden="true"></i>
      </button>
      @if ($configured)
        <ul class="dropdown-menu dropdown-menu-end range-menu">
          <li><a class="dropdown-item" href="{{ route('dashboard', ['range' => 'daily']) }}">Last 14 days</a></li>
          <li><a class="dropdown-item" href="{{ route('dashboard', ['range' => 'weekly']) }}">Last 8 weeks</a></li>
          <li><a class="dropdown-item" href="{{ route('dashboard', ['range' => 'monthly']) }}">Last 6 months</a></li>
        </ul>
      @endif
    </div>
  </header>

  @if (! empty($apiError))
    <div class="flash err">{{ $apiError }}</div>
  @endif

  @unless ($configured)
    <section class="gs-panel dash-empty-banner">
      <div class="gs-panel-title">
        <span class="gs-panel-mark" aria-hidden="true"><i class="bi bi-shield-check"></i></span>
        <div>
          <h2>{{ $emptyCopy['title'] }}</h2>
          <p>{{ $emptyCopy['body'] }}</p>
        </div>
      </div>
      <div class="dash-empty-actions">
        <a class="btn btn-primary" href="{{ $emptyCopy['href'] }}">{{ $emptyCopy['cta'] }} <i class="bi bi-arrow-right" aria-hidden="true"></i></a>
        <a class="gs-outline" href="{{ route('get-started') }}">View Get Started</a>
      </div>
    </section>
  @endunless

  <div class="kpi-row">
    <div class="dashboard-figures">
      @foreach ($kpis as $kpi)
        <article class="gs-stat {{ $configured ? '' : 'is-empty' }}">
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
            <p>{{ $configured ? 'Collection and payout trend' : 'Charts unlock after API keys are issued' }}</p>
          </div>
        </div>
        @if ($configured)
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
        @endif
      </div>
      @if ($configured)
        <div class="flow-summary">
          <div>
            <span>Money In</span>
            <strong>{{ $naira((float) $moneyIn) }}</strong>
          </div>
          <div>
            <span>Money Out</span>
            <strong>{{ $naira((float) $moneyOut) }}</strong>
          </div>
          <div class="flow-legend">
            <span><i class="is-in"></i> Money In</span>
            <span><i class="is-out"></i> Money Out</span>
          </div>
        </div>
        <div class="chart-frame">
          <canvas id="dashboardFlow" aria-label="Money movement for {{ $periodLabel }}" data-series='@json($points)'></canvas>
        </div>
      @else
        <div class="dash-empty-panel">
          <i class="bi bi-bar-chart" aria-hidden="true"></i>
          <p>No movement data yet. Complete compliance and issue keys to see collections vs payouts.</p>
        </div>
      @endif
    </section>

    <section class="gs-panel">
      <div class="gs-panel-head">
        <div class="gs-panel-title">
          <span class="gs-panel-mark" aria-hidden="true"><i class="bi bi-box"></i></span>
          <div>
            <h2>Collections by service</h2>
            <p>{{ $configured ? 'Top performing channels' : 'Channels appear after your first collections' }}</p>
          </div>
        </div>
        <a class="menu-link" href="{{ route('payment-simulation') }}">View tools</a>
      </div>
      @if ($configured && $transactionCount > 0)
        <ul class="box-bars">
          @foreach ($boxes as $row)
            <li>
              <span class="box-bars-name">{{ $row['name'] }}</span>
              <span class="box-bars-track" aria-hidden="true"><span style="width: {{ max(4, ($row['collected'] / $peak) * 100) }}%"></span></span>
              <strong>{{ $naira((float) $row['collected']) }}</strong>
            </li>
          @endforeach
        </ul>
      @else
        <div class="dash-empty-panel">
          <i class="bi bi-inbox" aria-hidden="true"></i>
          <p>
            @if ($configured)
              No channel breakdown for this period yet.
            @else
              Empty until compliance is complete and keys are issued.
            @endif
          </p>
        </div>
      @endif
    </section>
  </div>

  <div class="dash-full">
    <section class="gs-panel">
      <div class="gs-panel-head">
        <div class="gs-panel-title">
          <span class="gs-panel-mark" aria-hidden="true"><i class="bi bi-arrow-left-right"></i></span>
          <div>
            <h2>Recent Transactions</h2>
            <p>{{ $configured ? 'Latest activity on your merchant zainbox' : 'Transaction history unlocks after setup' }}</p>
          </div>
        </div>
        @if ($configured)
          <a class="menu-link" href="{{ route('transactions') }}">View all</a>
        @endif
      </div>
      @if ($configured && count($transactions))
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
                @php
                  $when = $transaction['date'] ?? null;
                @endphp
                <tr>
                  <td class="dash-ref">{{ $transaction['reference'] }}</td>
                  <td>{{ $transaction['zainbox'] }}</td>
                  <td>
                    <b>{{ $transaction['sender'] }}</b>
                    <small class="tx-ref">{{ $transaction['type'] }}</small>
                  </td>
                  <td class="{{ ! empty($transaction['out']) ? 'amount-out' : 'amount-in' }}">{{ $naira((float) $transaction['amount']) }}</td>
                  <td>
                    @if ($when)
                      <b class="dash-when">{{ $when->format('M j, Y') }}</b>
                      <small class="tx-ref">{{ $when->format('H:i') }}</small>
                    @else
                      <b class="dash-when">—</b>
                    @endif
                  </td>
                  <td><span class="status-pill {{ $transaction['tone'] }}">{{ $transaction['status'] }}</span></td>
                </tr>
              @endforeach
            </tbody>
          </table>
        </div>
      @else
        <div class="dash-empty-panel">
          <i class="bi bi-receipt" aria-hidden="true"></i>
          <p>
            @if ($configured)
              No transactions in this period.
            @else
              Once you finish compliance and issue <code>pk_…</code> / <code>sk_…</code>, recent payments will show here.
            @endif
          </p>
          @unless ($configured)
            <a class="btn btn-outline-primary" href="{{ $emptyCopy['href'] }}">{{ $emptyCopy['cta'] }}</a>
          @endunless
        </div>
      @endif
    </section>
  </div>
</section>
@endsection

@push('head')
<style>
  .dash-empty-banner{margin-bottom:14px;padding-bottom:16px}
  .dash-empty-actions{display:flex;flex-wrap:wrap;gap:10px;margin:14px 0 4px}
  .dash-empty-panel{display:flex;flex-direction:column;align-items:center;justify-content:center;gap:10px;min-height:180px;padding:28px 18px;text-align:center;color:#64748b}
  .dash-empty-panel i{font-size:28px;color:#94a3b8}
  .dash-empty-panel p{margin:0;max-width:42ch;font-size:14px;line-height:1.45}
  .gs-stat.is-empty{opacity:.92}
  .gs-stat.is-empty strong{color:#94a3b8}
</style>
@endpush
