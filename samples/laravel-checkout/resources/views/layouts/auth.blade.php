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
    .auth-submit-btn{display:inline-flex;align-items:center;justify-content:center;gap:.55rem;width:100%}
    .auth-submit-btn:disabled{cursor:wait;opacity:.85}
    .auth-submit-spinner{width:1rem;height:1rem;border-radius:50%;border:2px solid rgba(255,255,255,.35);border-top-color:#fff;display:none;flex-shrink:0}
    .auth-submit-btn.is-loading .auth-submit-spinner{display:inline-block;animation:auth-spin .7s linear infinite}
    @keyframes auth-spin{to{transform:rotate(360deg)}}
    @media (prefers-reduced-motion:reduce){.auth-submit-btn.is-loading .auth-submit-spinner{animation:none;opacity:.7}}
  </style>
</head>
<body>
  <main class="login-screen">
    @yield('content')
  </main>
  <script>
    (function () {
      function setLoading(btn, loading) {
        if (!btn) return;
        var label = btn.querySelector('.auth-submit-label');
        var idle = btn.getAttribute('data-idle-label') || (label ? label.textContent : 'Continue');
        var busy = btn.getAttribute('data-loading-label') || 'Please wait…';
        btn.disabled = loading;
        btn.classList.toggle('is-loading', loading);
        btn.setAttribute('aria-busy', loading ? 'true' : 'false');
        if (label) label.textContent = loading ? busy : idle;
      }

      document.querySelectorAll('form.auth-submit-form').forEach(function (form) {
        form.addEventListener('submit', function (event) {
          if (form.dataset.submitting === '1') {
            event.preventDefault();
            return;
          }
          form.dataset.submitting = '1';
          setLoading(form.querySelector('button.auth-submit-btn[type="submit"]'), true);
        });
      });

      window.addEventListener('pageshow', function (event) {
        if (!event.persisted) return;
        document.querySelectorAll('form.auth-submit-form').forEach(function (form) {
          form.dataset.submitting = '0';
          setLoading(form.querySelector('button.auth-submit-btn[type="submit"]'), false);
        });
      });
    })();
  </script>
</body>
</html>
