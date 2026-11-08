@extends('layouts.auth')

@section('title', 'Create account')

@section('content')
<form class="login-card auth-submit-form" method="POST" action="{{ route('register') }}">
  @csrf
  <a href="{{ route('register') }}" class="brand login-brand">
    <img class="brand-logo" src="{{ asset('images/koodify-logo-full.png') }}" alt="Koodify">
  </a>
  <h1>Create account</h1>
  <p>Register your business, complete KYC, then receive publishable and secret keys.</p>

  @if (session('status'))
    <div class="flash ok">{{ session('status') }}</div>
  @endif
  @if (session('error'))
    <div class="flash err">{{ session('error') }}</div>
  @endif
  @if ($errors->any())
    <div class="flash err">{{ $errors->first() }}</div>
  @endif

  <label class="field" for="displayName">Business name
    <input id="displayName" name="displayName" type="text" value="{{ old('displayName') }}" required autofocus placeholder="Acme Schools Ltd">
  </label>

  <div class="field-grid">
    <label class="field" for="email">Work email
      <input id="email" name="email" type="email" value="{{ old('email') }}" required autocomplete="username" placeholder="finance@acme.com">
    </label>
    <label class="field" for="mobileNumber">Phone number
      <input id="mobileNumber" name="mobileNumber" type="tel" value="{{ old('mobileNumber') }}" autocomplete="tel" placeholder="+234 800 000 0000">
    </label>
  </div>

  <div class="field-grid">
    <label class="field" for="password">Password
      <input id="password" name="password" type="password" required minlength="8" autocomplete="new-password" placeholder="Min. 8 characters">
    </label>
    <label class="field" for="password_confirmation">Confirm password
      <input id="password_confirmation" name="password_confirmation" type="password" required minlength="8" autocomplete="new-password" placeholder="Repeat password">
    </label>
  </div>

  <label class="check-row"><input type="checkbox" name="accept_terms" value="1" required> I agree to the Terms and Privacy Policy</label>

  <button
    class="btn btn-primary login-submit auth-submit-btn"
    type="submit"
    data-idle-label="Create merchant account"
    data-loading-label="Creating account…"
  >
    <span class="auth-submit-spinner" aria-hidden="true"></span>
    <span class="auth-submit-label">Create merchant account</span>
  </button>
  <p class="login-alt">Already registered? <a href="{{ route('login') }}">Sign in</a></p>
  <p class="login-legal">By continuing, you agree to our <a href="#">Terms</a> and <a href="#">Privacy Policy</a>.</p>
</form>
@endsection
