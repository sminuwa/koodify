@extends('layouts.auth')

@section('title', 'Admin sign in')

@section('content')
<form class="login-card auth-submit-form" method="POST" action="{{ route('admin.login.submit') }}">
  @csrf
  <a href="{{ route('admin.login') }}" class="brand login-brand">
    <img class="brand-logo" src="{{ asset('images/koodify-logo-full.png') }}" alt="Koodify">
  </a>
  <h1>Admin sign in</h1>
  <p>Use your platform admin username to review merchants, KYC, and provisioning.</p>

  @if (session('status'))
    <div class="flash ok">{{ session('status') }}</div>
  @endif
  @if (session('error'))
    <div class="flash err">{{ session('error') }}</div>
  @endif
  @if ($errors->any())
    <div class="flash err">{{ $errors->first() }}</div>
  @endif

  <label class="field" for="username">Username
    <input id="username" name="username" type="text" value="{{ old('username', config('koodify.admin_username')) }}" required autofocus autocomplete="username" placeholder="admin">
  </label>

  <label class="field" for="password">Password
    <input id="password" name="password" type="password" required autocomplete="current-password" placeholder="Enter your password">
  </label>

  <label class="check-row"><input type="checkbox" name="remember" value="1" @checked(old('remember', true))> Keep me signed in</label>

  <button
    class="btn btn-primary login-submit auth-submit-btn"
    type="submit"
    data-idle-label="Sign in"
    data-loading-label="Signing in…"
  >
    <span class="auth-submit-spinner" aria-hidden="true"></span>
    <span class="auth-submit-label">Sign in</span>
  </button>
  <p class="login-alt">Merchant account? <a href="{{ route('login') }}">Open merchant portal</a></p>
  <p class="login-legal">By continuing, you agree to our <a href="#">Terms</a> and <a href="#">Privacy Policy</a>.</p>
</form>
@endsection
