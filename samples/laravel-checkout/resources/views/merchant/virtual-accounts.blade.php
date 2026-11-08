@extends('layouts.merchant')

@section('title', 'Virtual Accounts · Koodify')
@section('crumb', 'Virtual Accounts')

@section('content')
<section class="ops-dashboard">
  <header class="dash-hero">
    <div>
      <h2>
        <span class="page-heading-mark" aria-hidden="true"><i class="bi bi-boxes"></i></span>
        <span>Virtual Accounts</span>
      </h2>
      <p>Accounts issued for this merchant. Create new ones from Payment Simulation.</p>
    </div>
    <a class="btn btn-primary menu-link" href="{{ route('payment-simulation', ['tool' => 'va']) }}">
      <i class="bi bi-plus-lg" aria-hidden="true"></i> Create VA
    </a>
  </header>

  @unless($configured)
    <div class="flash err">
      Issue a merchant <code>sk_…</code> from <a href="{{ route('credentials.show') }}">Credentials</a> to load accounts.
    </div>
  @endunless

  @if ($error)
    <div class="flash err">{{ $error }}</div>
  @endif

  <section class="gs-panel">
    <div class="gs-panel-head">
      <div class="gs-panel-title">
        <span class="gs-panel-mark" aria-hidden="true"><i class="bi bi-list-ul"></i></span>
        <div>
          <h2>Your accounts</h2>
          <p>{{ count($accounts) }} account{{ count($accounts) === 1 ? '' : 's' }} · <code>GET /api/v1/vas</code></p>
        </div>
      </div>
      <a class="menu-link" href="{{ route('payment-simulation', ['tool' => 'balance']) }}">Check balance</a>
    </div>

    @if ($accounts === [])
      <p class="dashboard-empty">No virtual accounts yet. Create one from Payment Simulation.</p>
    @else
      <div class="table-wrap">
        <table class="dash-table">
          <thead>
            <tr>
              <th scope="col">Account number</th>
              <th scope="col">Account name</th>
              <th scope="col">Bank</th>
              <th scope="col">Status</th>
              <th scope="col"></th>
            </tr>
          </thead>
          <tbody>
            @foreach ($accounts as $row)
              @php
                $number = (string) ($row['accountNumber'] ?? $row['account_number'] ?? '—');
                $name = (string) ($row['accountName'] ?? $row['account_name'] ?? '—');
                $bank = (string) ($row['bankName'] ?? $row['bank'] ?? $row['bankType'] ?? '—');
                $status = (string) ($row['status'] ?? $row['accountStatus'] ?? 'Active');
              @endphp
              <tr>
                <td class="dash-ref">{{ $number }}</td>
                <td><b>{{ $name }}</b></td>
                <td>{{ $bank }}</td>
                <td><span class="status-pill is-success">{{ $status }}</span></td>
                <td>
                  @if ($number !== '—')
                    <a class="menu-link" href="{{ route('payment-simulation', ['tool' => 'balance', 'accountNumber' => $number]) }}">Balance</a>
                  @endif
                </td>
              </tr>
            @endforeach
          </tbody>
        </table>
      </div>
    @endif
  </section>
</section>
@endsection
