<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="csrf-token" content="{{ csrf_token() }}">
  <meta name="theme-color" content="#171a35">
  <title>Koodify Admin Portal</title>
  <link rel="icon" href="{{ asset('brand/favicon/favicon.ico') }}" sizes="any">
  <link rel="icon" type="image/png" sizes="32x32" href="{{ asset('brand/favicon/favicon-32x32.png') }}">
  <link rel="apple-touch-icon" href="{{ asset('brand/favicon/apple-touch-icon.png') }}">
  <link rel="manifest" href="{{ asset('brand/favicon/site.webmanifest') }}">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=DM+Sans:wght@400;500;600;700&family=Manrope:wght@400;500;600;700;800&display=swap" rel="stylesheet">
  <link rel="stylesheet" href="{{ asset('admin/css/styles.css') }}">
  <style>
    .brand.brand-logo{display:inline-flex;align-items:center;text-decoration:none}
    .brand.brand-logo img{height:34px;width:auto;display:block}
    .mobile-brand img{height:28px;width:auto;display:block}
  </style>
</head>
<body>
  <main class="auth-shell">
    <section class="auth-brand">
      <a class="brand brand-logo" href="{{ route('admin.login') }}" aria-label="Koodify home">
        <img src="{{ asset('brand/koodify-logo-white.png') }}" alt="Koodify" width="166" height="40">
      </a>
      <span class="eyebrow">ADMIN PORTAL</span>
      <div class="brand-copy">
        <h1 id="brandHeadline">Manage, integrate<br>and grow.</h1>
        <p id="brandDescription">Your command centre for merchants, transactions and the services powering your business.</p>
      </div>
      <div class="visual-orbit">
        <div class="orbit orbit-one"></div><div class="orbit orbit-two"></div>
        <div class="glass-panel">
          <div class="panel-top"><i></i><i></i><i></i><span>KOODIFY / OVERVIEW</span></div>
          <div class="mini-metrics"><div><small>TRANSACTION VOLUME</small><b>₦24.8M</b><span class="up">↗ 18.6%</span></div><div class="bars"><i></i><i></i><i></i><i></i><i></i><i></i><i></i></div></div>
          <svg viewBox="0 0 360 100" class="mini-chart" role="img" aria-label="Decorative upward analytics chart"><defs><linearGradient id="fill" x1="0" x2="0" y1="0" y2="1"><stop offset="0" stop-color="#9c83ff" stop-opacity=".42"/><stop offset="1" stop-color="#9c83ff" stop-opacity="0"/></linearGradient></defs><path d="M0 82 C35 70 40 75 70 58 S120 72 150 45 S205 55 235 30 S290 48 360 8 L360 100 L0 100Z" fill="url(#fill)"/><path d="M0 82 C35 70 40 75 70 58 S120 72 150 45 S205 55 235 30 S290 48 360 8" fill="none" stroke="#a88cff" stroke-width="3" stroke-linecap="round"/></svg>
        </div>
        <div class="floating-chip chip-a"><span class="chip-icon">↗</span><span><small>Success rate</small><b>99.8%</b></span></div>
        <div class="floating-chip chip-b"><span class="chip-icon purple">✦</span><span><small>Platform status</small><b>All systems live</b></span></div>
      </div>
      <div class="brand-footer"><span>SECURE ACCESS</span><span>BUILT FOR OPERATIONS <b>✦</b></span></div>
    </section>

    <section class="auth-side">
      <div class="auth-top">
        <span class="secure-label"><span class="secure-dot"></span> Secure administrator access</span>
        <span class="help-link">Need help? <a href="mailto:support@koodify.ng">Contact support</a></span>
      </div>
      <div class="auth-card-wrap">
        <section class="auth-card" id="loginView">
          <div class="mobile-brand">
            <img src="{{ asset('brand/koodify-logo.png') }}" alt="Koodify" width="140" height="34">
          </div>
          <span class="card-kicker">WELCOME BACK</span>
          <h2>Sign in to your account<span class="accent-dot">.</span></h2>
          <p class="muted">Enter your credentials to access the admin portal.</p>
          <form method="post" action="{{ route('admin.login.submit') }}" id="loginForm">
            @csrf
            <label for="loginEmail">Email address</label>
            <div class="input-wrap"><span class="input-icon">✉</span><input id="loginEmail" name="email" type="email" value="{{ old('email', config('koodify.admin_email')) }}" placeholder="you@company.com" autocomplete="username" required></div>
            <div class="label-row"><label for="loginPassword">Password</label><button type="button" class="text-button" id="forgotBtn">Forgot password?</button></div>
            <div class="input-wrap"><span class="input-icon">⌑</span><input id="loginPassword" name="password" type="password" placeholder="Enter your password" autocomplete="current-password" required><button class="show-password" type="button" data-target="loginPassword" aria-label="Show password">◉</button></div>
            <div class="form-options"><label class="check-label"><input type="checkbox" id="rememberMe" checked><span class="checkmark"></span>Remember me</label><span class="muted tiny">Protected by encryption</span></div>
            @if(session('status'))
              <p class="form-message success" aria-live="polite">{{ session('status') }}</p>
            @elseif(session('error'))
              <p class="form-message" aria-live="polite">{{ session('error') }}</p>
            @elseif($errors->any())
              <p class="form-message" aria-live="polite">{{ $errors->first() }}</p>
            @else
              <p class="form-message" id="loginMessage" aria-live="polite"></p>
            @endif
            <button class="primary-button" type="submit"><span>Sign in to portal</span><b>→</b></button>
          </form>
          <div class="auth-switch">Merchant account? <a class="text-button" href="{{ route('login') }}" style="border:0;background:none;padding:0;color:inherit;font:inherit;font-weight:700;cursor:pointer">Open merchant portal</a></div>
          <div class="auth-legal">By continuing, you agree to our <a href="#">Terms</a> and <a href="#">Privacy Policy</a>.</div>
        </section>
        <p class="auth-footnote">© <span id="year"></span> Koodify Technologies. All rights reserved.</p>
      </div>
    </section>
  </main>
  <div class="toast" id="toast" role="status"></div>
  <script>
    document.getElementById('year').textContent = new Date().getFullYear();
    document.querySelectorAll('.show-password').forEach(btn => {
      btn.addEventListener('click', () => {
        const input = document.getElementById(btn.dataset.target);
        input.type = input.type === 'password' ? 'text' : 'password';
        btn.textContent = input.type === 'password' ? '◉' : '◎';
      });
    });
    const toast = document.getElementById('toast');
    document.getElementById('forgotBtn')?.addEventListener('click', () => {
      toast.textContent = 'Password reset will use your ops email flow.';
      toast.classList.add('show');
      setTimeout(() => toast.classList.remove('show'), 2800);
    });
  </script>
</body>
</html>
