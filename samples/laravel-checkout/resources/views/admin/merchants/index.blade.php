@extends('layouts.admin')

@section('title', 'Merchants · Koodify Admin')
@section('breadcrumb', 'Merchants')

@section('content')
<div class="page-heading">
  <div>
    <span class="card-kicker">REGISTRY</span>
    <h1>Merchants</h1>
    <p class="muted">Customers from the Koodify registry — review status and KYC.</p>
  </div>
  <div class="heading-actions">
    <a class="outline-button" href="{{ route('admin.dashboard') }}" style="text-decoration:none">← Overview</a>
  </div>
</div>

@if($error)
  <div class="flash err">{{ $error }}</div>
@endif

<section class="surface transactions-card">
  <div class="surface-heading">
    <div>
      <h3>All merchants</h3>
      <p class="muted">{{ count($customers) }} account{{ count($customers) === 1 ? '' : 's' }} in the registry.</p>
    </div>
  </div>
  @if(count($customers))
    <div class="table-scroll">
      <table>
        <thead>
          <tr>
            <th>ID</th>
            <th>MERCHANT</th>
            <th>EMAIL</th>
            <th>MOBILE</th>
            <th>STATUS</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          @foreach($customers as $c)
            @php
              $name = $c['displayName'] ?? $c['display_name'] ?? 'Merchant';
              $parts = preg_split('/\s+/', trim($name)) ?: [];
              $av = collect($parts)->take(2)->map(fn ($p) => mb_strtoupper(mb_substr($p, 0, 1)))->implode('') ?: 'M';
              $colors = ['ma-purple', 'ma-blue', 'ma-orange', 'ma-green'];
              $color = $colors[($c['id'] ?? 0) % 4];
              $st = $c['status'] ?? '—';
            @endphp
            <tr>
              <td><b>#{{ $c['id'] ?? '—' }}</b></td>
              <td><span class="merchant-avatar {{ $color }}">{{ $av }}</span> {{ $name }}</td>
              <td>{{ $c['email'] ?? '—' }}</td>
              <td>{{ $c['mobileNumber'] ?? $c['mobile_number'] ?? '—' }}</td>
              <td><span class="status {{ $st === 'active' || $st === 'approved' ? 'success' : ($st === 'pending' ? 'pending' : 'failed') }}">● {{ ucfirst($st) }}</span></td>
              <td><a class="text-button" href="{{ route('admin.merchants.show', $c['id']) }}">Review →</a></td>
            </tr>
          @endforeach
        </tbody>
      </table>
    </div>
  @else
    <div class="empty-state">
      <h3>No merchants found</h3>
      <p>Have a business register at <a href="{{ route('register') }}">/register</a>.</p>
    </div>
  @endif
</section>
@endsection
