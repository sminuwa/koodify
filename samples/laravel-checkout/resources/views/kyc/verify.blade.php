@extends('layouts.merchant')

@section('title', 'Company Representative · Koodify')
@section('crumb', 'Company Representative')

@section('content')
<div class="gs-panel" style="max-width:760px">
  <div class="gs-panel-title">
    <span class="gs-panel-mark" aria-hidden="true"><i class="bi bi-people"></i></span>
    <div>
      <h2>Company Representative</h2>
      <p>Provide identity details so we can provision your payment account. Status stays pending until approved.</p>
    </div>
  </div>

  <form class="mt-3 compliance-submit-form" method="post" action="{{ route('compliance.submit') }}">
    @csrf
    <div class="row g-3">
      <div class="col-md-6">
        <label class="field" for="title">Title
          <select id="title" name="title" required>
            @foreach (['Mr', 'Mrs', 'Ms', 'Dr'] as $t)
              <option value="{{ $t }}" @selected(old('title', 'Mr') === $t)>{{ $t }}</option>
            @endforeach
          </select>
        </label>
      </div>
      <div class="col-md-6">
        <label class="field" for="gender">Gender
          <select id="gender" name="gender" required>
            <option value="M" @selected(old('gender', 'M') === 'M')>Male</option>
            <option value="F" @selected(old('gender') === 'F')>Female</option>
          </select>
        </label>
      </div>
      <div class="col-md-6">
        <label class="field" for="firstName">First name
          <input id="firstName" name="firstName" value="{{ old('firstName') }}" required>
        </label>
      </div>
      <div class="col-md-6">
        <label class="field" for="middleName">Middle name
          <input id="middleName" name="middleName" value="{{ old('middleName') }}">
        </label>
      </div>
      <div class="col-12">
        <label class="field" for="surname">Surname
          <input id="surname" name="surname" value="{{ old('surname') }}" required>
        </label>
      </div>
      <div class="col-md-6">
        <label class="field" for="email">Email
          <input id="email" type="email" name="email" value="{{ old('email', $email) }}" required>
        </label>
      </div>
      <div class="col-md-6">
        <label class="field" for="mobileNumber">Mobile
          <input id="mobileNumber" name="mobileNumber" value="{{ old('mobileNumber', $mobile) }}" required>
        </label>
      </div>
      <div class="col-md-6">
        <label class="field" for="bvn">BVN (11 digits)
          <input id="bvn" name="bvn" value="{{ old('bvn') }}" required maxlength="11" pattern="\d{11}">
        </label>
      </div>
      <div class="col-md-6">
        <label class="field" for="dob">Date of birth (DD-MM-YYYY)
          <input id="dob" name="dob" value="{{ old('dob', '15-06-1990') }}" required placeholder="15-06-1990">
        </label>
      </div>
      <div class="col-12">
        <label class="field" for="address">Address
          <input id="address" name="address" value="{{ old('address') }}" required>
        </label>
      </div>
      <div class="col-12">
        <label class="field" for="state">State
          <input id="state" name="state" value="{{ old('state', 'Lagos') }}" required>
        </label>
      </div>
    </div>

    <div class="d-flex flex-wrap gap-2 mt-4">
      <button
        class="btn btn-primary compliance-submit-btn"
        type="submit"
        data-idle-label="Submit KYC"
        data-loading-label="Submitting…"
      >
        <span class="compliance-submit-spinner" aria-hidden="true"></span>
        <span class="compliance-submit-label">Submit KYC</span>
      </button>
      <a class="btn btn-outline-secondary" href="{{ route('compliance.show') }}">Back to Compliance</a>
    </div>
  </form>
</div>
@endsection

@push('head')
<style>
  .compliance-submit-btn{display:inline-flex;align-items:center;justify-content:center;gap:.55rem;min-width:9.5rem}
  .compliance-submit-btn:disabled{cursor:wait;opacity:.85}
  .compliance-submit-spinner{width:1rem;height:1rem;border-radius:50%;border:2px solid rgba(255,255,255,.35);border-top-color:#fff;display:none;flex-shrink:0}
  .compliance-submit-btn.is-loading .compliance-submit-spinner{display:inline-block;animation:compliance-spin .7s linear infinite}
  @keyframes compliance-spin{to{transform:rotate(360deg)}}
  @media (prefers-reduced-motion:reduce){.compliance-submit-btn.is-loading .compliance-submit-spinner{animation:none;opacity:.7}}
</style>
@endpush

@push('scripts')
<script>
  document.addEventListener('DOMContentLoaded', function () {
    function setLoading(btn, loading) {
      if (!btn) return;
      var label = btn.querySelector('.compliance-submit-label');
      var idle = btn.getAttribute('data-idle-label') || 'Submit KYC';
      var busy = btn.getAttribute('data-loading-label') || 'Submitting…';
      btn.disabled = loading;
      btn.classList.toggle('is-loading', loading);
      btn.setAttribute('aria-busy', loading ? 'true' : 'false');
      if (label) label.textContent = loading ? busy : idle;
    }

    document.querySelectorAll('form.compliance-submit-form').forEach(function (form) {
      form.addEventListener('submit', function (event) {
        if (form.dataset.submitting === '1') {
          event.preventDefault();
          return;
        }
        form.dataset.submitting = '1';
        setLoading(form.querySelector('button.compliance-submit-btn[type="submit"]'), true);
      });
    });

    window.addEventListener('pageshow', function (event) {
      if (!event.persisted) return;
      document.querySelectorAll('form.compliance-submit-form').forEach(function (form) {
        form.dataset.submitting = '0';
        setLoading(form.querySelector('button.compliance-submit-btn[type="submit"]'), false);
      });
    });
  });
</script>
@endpush
