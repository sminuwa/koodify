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

  <form class="mt-3" method="post" action="{{ route('compliance.submit') }}">
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
      <button class="btn btn-primary" type="submit">Submit KYC</button>
      <a class="btn btn-outline-secondary" href="{{ route('compliance.show') }}">Back to Compliance</a>
    </div>
  </form>
</div>
@endsection
