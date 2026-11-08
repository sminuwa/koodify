<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="csrf-token" content="{{ csrf_token() }}">
  <meta name="theme-color" content="#171a35">
  <title>@yield('title', 'Koodify Admin Portal')</title>
  <link rel="icon" href="{{ asset('brand/favicon/favicon.ico') }}" sizes="any">
  <link rel="icon" type="image/png" sizes="32x32" href="{{ asset('brand/favicon/favicon-32x32.png') }}">
  <link rel="apple-touch-icon" href="{{ asset('brand/favicon/apple-touch-icon.png') }}">
  <link rel="manifest" href="{{ asset('brand/favicon/site.webmanifest') }}">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=DM+Sans:wght@400;500;600;700&family=Manrope:wght@400;500;600;700;800&display=swap" rel="stylesheet">
  <link rel="stylesheet" href="{{ asset('admin/css/styles.css') }}">
  <style>
    a.nav-item,a.outline-button,a.primary-button,a.text-button{text-decoration:none;color:inherit}
    a.outline-button,a.primary-button{display:inline-flex;align-items:center;gap:8px}
    form.logout-form{margin:0}
    .brand.brand-logo,.brand.sidebar-brand{display:inline-flex;align-items:center;text-decoration:none}
    .brand.brand-logo img,.brand.sidebar-brand img{height:28px;width:auto;display:block}
    .flash{margin:0 0 16px;padding:12px 14px;border-radius:10px;font-size:13px}
    .flash.ok{background:#ecfdf5;color:#047857;border:1px solid #a7f3d0}
    .flash.err{background:#fef2f2;color:#b91c1c;border:1px solid #fecaca}
    .detail-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:14px;margin-bottom:18px}
    @media(max-width:760px){.detail-grid{grid-template-columns:1fr}}
    .detail-item{padding:14px;border:1px solid var(--line);border-radius:12px;background:#fff}
    .detail-item small{display:block;color:#8b90a7;margin-bottom:4px;font-size:11px}
    .detail-item b{font-size:14px;word-break:break-word}
    .actions-inline{display:flex;flex-wrap:wrap;gap:8px;margin-top:12px}
    .actions-inline form{margin:0}
    .badge{display:inline-flex;align-items:center;padding:3px 9px;border-radius:999px;font-size:11px;font-weight:700}
    .badge.pending{background:#fff7e8;color:#9a6700}
    .badge.approved,.badge.active{background:#e8faf1;color:#0f7a43}
    .badge.rejected{background:#fdecec;color:#b91c1c}
    .empty-state{padding:48px 20px;text-align:center;color:#747991}
    .empty-state h3{margin:0 0 8px;font:700 20px Manrope,sans-serif;color:#171a31}
  </style>
  @stack('head')
</head>
<body>
@php
  $adminName = session('admin.name', 'Admin User');
  $initials = collect(preg_split('/\s+/', trim($adminName)))->filter()->take(2)->map(fn ($p) => mb_strtoupper(mb_substr($p, 0, 1)))->implode('') ?: 'AD';
@endphp
<main class="dashboard">
  <aside class="sidebar" id="sidebar">
    <a class="brand sidebar-brand brand-logo" href="{{ route('admin.dashboard') }}" aria-label="Koodify home">
      <img src="{{ asset('brand/koodify-logo-white.png') }}" alt="Koodify" width="140" height="34">
    </a>
    <div class="side-caption">WORKSPACE</div>
    <nav class="side-nav">
      <a class="nav-item @if(request()->routeIs('admin.dashboard')) active @endif" href="{{ route('admin.dashboard') }}"><span>⌂</span> Overview</a>
      <a class="nav-item @if(request()->routeIs('admin.section') && request()->route('section')==='transactions') active @endif" href="{{ route('admin.section', 'transactions') }}"><span>⇄</span> Transactions <i class="nav-count">12</i></a>
      <a class="nav-item @if(request()->routeIs('admin.merchants.*')) active @endif" href="{{ route('admin.merchants.index') }}"><span>▣</span> Merchants</a>
      <a class="nav-item @if(request()->routeIs('admin.section') && request()->route('section')==='customers') active @endif" href="{{ route('admin.section', 'customers') }}"><span>♙</span> Customers</a>
      <a class="nav-item @if(request()->routeIs('admin.section') && request()->route('section')==='services') active @endif" href="{{ route('admin.section', 'services') }}"><span>◇</span> Services</a>
      <a class="nav-item @if(request()->routeIs('admin.section') && request()->route('section')==='reports') active @endif" href="{{ route('admin.section', 'reports') }}"><span>▤</span> Reports</a>
    </nav>
    <div class="side-caption side-caption-lower">PREFERENCES</div>
    <nav class="side-nav">
      <a class="nav-item @if(request()->routeIs('admin.section') && request()->route('section')==='settings') active @endif" href="{{ route('admin.section', 'settings') }}"><span>⚙</span> Settings</a>
      <a class="nav-item @if(request()->routeIs('admin.section') && request()->route('section')==='activity') active @endif" href="{{ route('admin.section', 'activity') }}"><span>◷</span> Activity log</a>
    </nav>
    <div class="sidebar-bottom">
      <div class="system-status"><span class="secure-dot"></span><span><b>All systems operational</b><small>Last checked just now</small></span></div>
      <button class="profile-mini" type="button" id="profileMenu">
        <span class="avatar" id="avatarInitials">{{ $initials }}</span>
        <span class="profile-text"><b id="profileName">{{ $adminName }}</b><small>Super administrator</small></span>
        <span class="more">•••</span>
      </button>
      <form class="logout-form" method="post" action="{{ route('admin.logout') }}">@csrf
        <button class="logout-link" type="submit">↪ Sign out</button>
      </form>
    </div>
  </aside>

  <section class="dashboard-main">
    <header class="topbar">
      <button class="mobile-menu" id="menuBtn" type="button">☰</button>
      <div class="breadcrumb">Workspace <span>/</span> <b id="breadcrumbTitle">@yield('breadcrumb', 'Overview')</b></div>
      <div class="top-actions">
        <div class="search-box"><span>⌕</span><input type="search" placeholder="Search anything..."><kbd>⌘ K</kbd></div>
        <button class="icon-button" type="button" aria-label="Notifications">♧<i></i></button>
        <button class="icon-button" type="button" aria-label="Help">?</button>
        <button class="top-avatar" type="button" id="topAvatar">{{ $initials }}</button>
      </div>
    </header>
    <div class="content-area">
      @if(session('status'))<div class="flash ok">{{ session('status') }}</div>@endif
      @if(session('error'))<div class="flash err">{{ session('error') }}</div>@endif
      @yield('content')
      <footer class="dashboard-footer"><span>© {{ date('Y') }} Koodify Technologies</span><span>Admin Portal <b>·</b> v1.0.0</span></footer>
    </div>
  </section>
  <div class="toast" id="toast" role="status"></div>
</main>
<script>
  document.getElementById('menuBtn')?.addEventListener('click', () => {
    document.getElementById('sidebar')?.classList.toggle('open');
  });
</script>
@stack('scripts')
</body>
</html>
