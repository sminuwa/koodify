<!doctype html>
<html lang="en" @class(['is-dashboard' => request()->routeIs('dashboard')])>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="csrf-token" content="{{ csrf_token() }}">
  <title>@yield('title', 'Account') — Koodify</title>
  <link rel="icon" href="{{ asset('brand/favicon/favicon.ico') }}" sizes="any">
  <link rel="icon" type="image/png" sizes="32x32" href="{{ asset('brand/favicon/favicon-32x32.png') }}">
  <link rel="apple-touch-icon" href="{{ asset('brand/favicon/apple-touch-icon.png') }}">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Instrument+Sans:wght@400;500;600;700&display=swap" rel="stylesheet">
  <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css" rel="stylesheet">
  <link href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.3/font/bootstrap-icons.min.css" rel="stylesheet">
  <link href="{{ asset('vendor/datatables/dataTables.bootstrap5.min.css') }}" rel="stylesheet">
  <link href="{{ asset('vendor/select2/select2.min.css') }}" rel="stylesheet">
  <link href="{{ asset('vendor/select2/select2-bootstrap-5-theme.min.css') }}" rel="stylesheet">
  <link href="{{ asset('vendor/flatpickr/flatpickr.min.css') }}" rel="stylesheet">
  <link href="https://cdn.jsdelivr.net/npm/toastr@2.1.4/build/toastr.min.css" rel="stylesheet">
  <link href="{{ asset('vendor/sweetalert2/sweetalert2.min.css') }}" rel="stylesheet">
  <link href="{{ asset('css/style.css') }}?t={{ @filemtime(public_path('css/style.css')) }}" rel="stylesheet">
  <style>
    form.nav-logout{margin:0}
    form.nav-logout .nav-link{width:100%;border:0;background:transparent;text-align:left;cursor:pointer}
    .merchant-panel{background:#fff;border:1px solid var(--line,#e4eaf2);border-radius:16px;padding:22px;box-shadow:0 8px 24px rgba(11,27,51,.06)}
    .merchant-panel .lede{color:var(--muted,#546578);margin:8px 0 0}
    .merchant-panel .field{margin-bottom:14px}
    .merchant-panel label{display:block;font-size:.82rem;font-weight:600;margin-bottom:6px}
    .merchant-panel .grid{display:grid;grid-template-columns:1fr 1fr;gap:14px}
    @media(max-width:760px){.merchant-panel .grid{grid-template-columns:1fr}}
    .merchant-panel .actions{display:flex;flex-wrap:wrap;gap:10px;margin-top:18px}
    .flash{margin:0 0 16px;padding:12px 14px;border-radius:12px;font-size:.92rem}
    .flash.ok{background:#ecfdf5;color:#047857;border:1px solid #a7f3d0}
    .flash.err{background:#fef2f2;color:#b91c1c;border:1px solid #fecaca}
  </style>
  @stack('head')
</head>
<body @if (session('status')) data-toast="{{ session('status') }}" @endif>
@php
  $merchantName = session('merchant.display_name') ?: (session('merchant.profile')['displayName'] ?? 'Merchant');
  $merchantEmail = session('merchant.email') ?: (session('merchant.profile')['email'] ?? '');
  $nameParts = preg_split('/\s+/', trim($merchantName)) ?: [];
  $initials = strtoupper(mb_substr($nameParts[0] ?? 'M', 0, 1).mb_substr($nameParts[1] ?? '', 0, 1));
  $apiBase = (string) config('koodify.api_base');
  $isLive = ! str_contains($apiBase, '127.0.0.1') && ! str_contains($apiBase, 'localhost') && ! str_contains($apiBase, 'sandbox');
@endphp
<div class="app-shell" id="appShell">
  <header class="topbar">
    <button class="icon-btn mobile-menu" id="mobileMenu" type="button" aria-label="Open menu"><i class="bi bi-list"></i></button>
    <a href="{{ route('get-started') }}" class="brand"><img class="brand-logo" src="{{ asset('images/koodify-logo-full.png') }}" alt="Koodify"></a>
    <label class="search-box"><i class="bi bi-search"></i><input id="globalSearch" type="search" placeholder="Search accounts, transactions and documentation..."><kbd>⌘ K</kbd></label>
    <div class="top-actions">
      <label class="env-switch">
        <span class="env-dot" aria-hidden="true"></span>
        <span class="visually-hidden">Environment</span>
        <select id="envSelect" aria-label="Environment" data-env="{{ $isLive ? 'live' : 'sandbox' }}" data-width="168px">
          <option value="sandbox" @selected(! $isLive)>Sandbox</option>
          <option value="live" @selected($isLive)>Production</option>
        </select>
      </label>
      <button class="icon-btn" id="themeToggle" type="button" aria-label="Toggle theme" title="Toggle theme"><i class="bi bi-moon"></i></button>
      <div class="dropdown">
        <button class="icon-btn" type="button" data-bs-toggle="dropdown" aria-label="Notifications" aria-expanded="false"><i class="bi bi-bell"></i></button>
        <div class="dropdown-menu dropdown-menu-end notify-menu">
          <div class="dropdown-header">Notifications</div>
          <p class="notify-empty">You're all caught up.</p>
        </div>
      </div>
      <div class="dropdown">
        <button class="profile" type="button" data-bs-toggle="dropdown" aria-expanded="false" aria-label="Account menu">
          <div class="avatar">{{ $initials }}</div>
          <div class="profile-copy"><strong>{{ $merchantName }}</strong><small>Owner</small></div>
          <i class="bi bi-chevron-down"></i>
        </button>
        <ul class="dropdown-menu dropdown-menu-end account-menu">
          <li class="account-menu-head">
            <div class="avatar">{{ $initials }}</div>
            <div>
              <strong>{{ $merchantName }}</strong>
              <small>{{ $merchantEmail }}</small>
            </div>
          </li>
          <li><hr class="dropdown-divider"></li>
          <li><a class="dropdown-item" href="{{ route('profile') }}"><i class="bi bi-person"></i> Profile</a></li>
          <li><a class="dropdown-item" href="{{ route('credentials.show') }}"><i class="bi bi-key"></i> Credentials</a></li>
          <li><a class="dropdown-item" href="{{ route('compliance.show') }}"><i class="bi bi-shield-check"></i> Compliance</a></li>
          <li><hr class="dropdown-divider"></li>
          <li>
            <form method="POST" action="{{ route('logout') }}">
              @csrf
              <button class="dropdown-item is-danger" type="submit"><i class="bi bi-box-arrow-right"></i> Logout</button>
            </form>
          </li>
        </ul>
      </div>
    </div>
  </header>

  <div class="app-frame">
    <aside class="sidebar" id="sidebar">
      <nav class="side-nav" aria-label="Account">
        @foreach (config('merchant.nav') as $group)
          <div class="nav-group">
            <div class="nav-heading">{{ $group['heading'] }}</div>
            @foreach ($group['items'] as $item)
              @if (($item['action'] ?? null) === 'logout')
                <form method="POST" action="{{ route('logout') }}" class="nav-logout">
                  @csrf
                  <button class="nav-link" type="submit"><i class="bi {{ $item['icon'] }}"></i><span>{{ $item['label'] }}</span></button>
                </form>
              @else
                @php
                  $external = isset($item['url']);
                  $href = $external ? $item['url'] : route($item['route']);
                  $match = $item['match'] ?? ($item['route'] ?? null);
                  $active = ! $external && $match && request()->routeIs($match);
                @endphp
                <a href="{{ $href }}" @class(['nav-link', 'active' => $active]) @if ($external) target="_blank" rel="noopener noreferrer" @endif><i class="bi {{ $item['icon'] }}"></i><span>{{ $item['label'] }}</span></a>
              @endif
            @endforeach
          </div>
        @endforeach
      </nav>
      <div class="help-card">
        <strong>Need a hand?</strong>
        <p>We're here to help you.</p>
        <a class="help-support" href="mailto:support@koodify.ng">Contact Support <i class="bi bi-arrow-right"></i></a>
        <svg class="help-art" viewBox="0 0 88 72" aria-hidden="true">
          <circle cx="58" cy="46" r="22" fill="#d7e7ff"></circle>
          <circle cx="54" cy="28" r="10" fill="#8eb6f5"></circle>
          <path d="M40 44c2-8 8-12 14-12s12 4 14 12" fill="#2f6bff"></path>
          <rect x="44" y="22" width="20" height="8" rx="4" fill="#12327a"></rect>
          <path d="M42 24c-4 1-6 5-5 9" fill="none" stroke="#12327a" stroke-width="2" stroke-linecap="round"></path>
          <path d="M66 24c4 1 6 5 5 9" fill="none" stroke="#12327a" stroke-width="2" stroke-linecap="round"></path>
        </svg>
      </div>
    </aside>

    <main class="main-area">
      <div class="content-wrap">
        @unless (request()->routeIs('get-started'))
          <div class="breadcrumb">
            <a href="{{ route('get-started') }}">Home</a>
            <i class="bi bi-chevron-right"></i>
            <span>@yield('crumb', 'Dashboard')</span>
          </div>
        @endunless
        @if(session('status') && !request()->routeIs('get-started'))
          <div class="flash ok">{{ session('status') }}</div>
        @endif
        @if(session('error'))
          <div class="flash err">{{ session('error') }}</div>
        @endif
        @yield('content')
        <footer class="footer">
          <span>© {{ date('Y') }} Koodify. All rights reserved.</span>
          <span>
            <a href="https://docs.koodify.ng/">API Docs</a>
            <a href="mailto:support@koodify.ng">Help & Support</a>
            <a href="#">Terms</a>
            <a href="#">Privacy</a>
          </span>
        </footer>
      </div>
    </main>
  </div>
</div>
<script src="https://code.jquery.com/jquery-3.7.1.min.js"></script>
<script src="https://cdn.jsdelivr.net/npm/toastr@2.1.4/build/toastr.min.js"></script>
<script src="{{ asset('vendor/sweetalert2/sweetalert2.all.min.js') }}"></script>
<script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js"></script>
<script src="{{ asset('vendor/datatables/dataTables.min.js') }}"></script>
<script src="{{ asset('vendor/datatables/dataTables.bootstrap5.min.js') }}"></script>
<script src="{{ asset('vendor/select2/select2.min.js') }}"></script>
<script src="{{ asset('vendor/chartjs/chart.umd.min.js') }}"></script>
<script src="{{ asset('js/account.js') }}"></script>
@stack('scripts')
</body>
</html>
