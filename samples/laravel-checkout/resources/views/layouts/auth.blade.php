<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="csrf-token" content="{{ csrf_token() }}">
  <title>@yield('title', 'Sign in') — Koodify</title>
  <link rel="icon" href="{{ asset('brand/favicon/favicon.ico') }}" sizes="any">
  <link rel="icon" type="image/png" sizes="32x32" href="{{ asset('brand/favicon/favicon-32x32.png') }}">
  <link rel="apple-touch-icon" href="{{ asset('brand/favicon/apple-touch-icon.png') }}">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Instrument+Sans:wght@400;500;600;700&display=swap" rel="stylesheet">
  <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css" rel="stylesheet">
  <link href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.3/font/bootstrap-icons.min.css" rel="stylesheet">
  <link href="{{ asset('css/style.css') }}?t={{ @filemtime(public_path('css/style.css')) }}" rel="stylesheet">
  <style>
    .login-card .flash{margin:0 0 14px;padding:12px 14px;border-radius:12px;font-size:.92rem}
    .login-card .flash.ok{background:#ecfdf5;color:#047857;border:1px solid #a7f3d0}
    .login-card .flash.err{background:#fef2f2;color:#b91c1c;border:1px solid #fecaca}
    .login-card .field-grid{display:grid;grid-template-columns:1fr 1fr;gap:12px}
    @media(max-width:520px){.login-card .field-grid{grid-template-columns:1fr}}
    .login-legal{margin:16px 0 0;text-align:center;font-size:12px;color:var(--muted,#64748b)}
    .login-legal a{color:inherit;font-weight:600}
  </style>
</head>
<body>
  <main class="login-screen">
    @yield('content')
  </main>
</body>
</html>
