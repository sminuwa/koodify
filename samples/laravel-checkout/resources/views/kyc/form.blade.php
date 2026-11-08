@extends('layouts.merchant')

@section('title', 'Compliance · Koodify')
@section('crumb', 'Compliance')

@section('content')
@php
  $sectionMeta = [
    'business' => [
      'icon' => 'bi-building',
      'title' => 'Business Information',
      'detail' => 'Provide your business details such as business name, preferred payment method, and monthly transaction details.',
    ],
    'representative' => [
      'icon' => 'bi-people',
      'title' => 'Company Representative',
      'detail' => 'Provide details of the authorised representative such as full name, address, phone number, NIN and identification details.',
    ],
    'documents' => [
      'icon' => 'bi-file-earmark-text',
      'title' => 'Business Documents',
      'detail' => 'Upload required documents such as CAC certificate and utility bill.',
    ],
    'declaration' => [
      'icon' => 'bi-shield-check',
      'title' => 'Declaration',
      'detail' => 'Read and confirm the declaration to complete your compliance verification.',
    ],
  ];

  $sections = [];
  foreach ($sectionMeta as $key => $meta) {
      $status = $compliance[$key]['status'] ?? 'none';
      if ($status === 'approved') {
          $state = 'done';
          $label = 'Approved';
          $cta = 'View';
      } elseif ($status === 'pending') {
          $state = 'progress';
          $label = 'Waiting for approval';
          $cta = 'View';
      } else {
          $state = 'pending';
          $label = 'Not started';
          $cta = 'Start now';
      }
      $sections[] = array_merge($meta, [
          'key' => $key,
          'state' => $state,
          'status' => $label,
          'cta' => $cta,
      ]);
  }

  $done = collect($sections)->whereIn('state', ['done', 'progress'])->count();
  $total = count($sections);
  $percent = (int) round(($done / max($total, 1)) * 100);

  $questions = [
    ['q' => 'Why do I need to complete compliance?', 'a' => 'Live payments stay off until the business is verified. Compliance confirms who owns the account and where settlements should go.'],
    ['q' => 'What documents are accepted?', 'a' => 'PDF or image uploads of your CAC certificate and utility bill. The company representative also needs an 11-digit NIN.'],
    ['q' => 'How long does verification take?', 'a' => 'Review starts after each section is submitted. The account stays on sandbox until review is finished.'],
    ['q' => 'Can I still use the sandbox?', 'a' => 'Yes. Sandbox collections and payouts stay available while compliance is in progress.'],
    ['q' => 'What happens after verification?', 'a' => 'A verified account can issue live API keys and start collecting. Keep the business details current after that.'],
  ];

  $openSection = old('section', $openSection ?? request('section'));
  $nextPending = collect($sections)->first(fn ($s) => $s['state'] === 'pending');
  $nextSection = is_array($nextPending) ? $nextPending['key'] : 'business';
@endphp

<div class="ky">
  <div class="ky-top">
    <section class="ky-hero">
      <div class="ky-hero-copy">
        <p class="ky-kicker">Account verification</p>
        <h2>Complete your KYC</h2>
        <p>Open each compliance section, submit the required details, and track approval status as your account is reviewed.</p>
        <button class="btn btn-primary" type="button" data-bs-toggle="modal" data-bs-target="#compliance-modal-{{ $nextSection }}">
          Continue compliance
          <i class="bi bi-arrow-right" aria-hidden="true"></i>
        </button>
      </div>
      <div class="ky-art" aria-hidden="true">
        <img src="{{ asset('images/compliance-kyc.png') }}" alt="">
      </div>
    </section>

    <aside class="gs-panel ky-progress">
      <div class="gs-panel-title">
        <span class="gs-panel-mark" aria-hidden="true"><i class="bi bi-shield-check"></i></span>
        <div>
          <h2>Verification progress</h2>
        </div>
      </div>
      <div class="gs-progress mt-3">
        <div class="gs-progress-track" role="progressbar" aria-valuenow="{{ $percent }}" aria-valuemin="0" aria-valuemax="100" aria-label="{{ $done }} of {{ $total }} submitted">
          <span style="width: {{ $percent }}%"></span>
        </div>
        <div class="gs-progress-meta">
          <b>{{ $done }} of {{ $total }} submitted</b>
          <b>{{ $percent }}%</b>
        </div>
      </div>
      <div class="gs-steps">
        @foreach ($sections as $section)
          <button class="gs-step is-{{ $section['state'] }} ky-step-open" type="button" data-bs-toggle="modal" data-bs-target="#compliance-modal-{{ $section['key'] }}">
            <span class="gs-step-mark" aria-hidden="true">
              @if ($section['state'] === 'done')
                <i class="bi bi-check-lg"></i>
              @elseif ($section['state'] === 'progress')
                <i class="bi bi-hourglass-split"></i>
              @else
                {{ $loop->iteration }}
              @endif
            </span>
            <span class="gs-step-copy"><b>{{ $section['title'] }}</b></span>
          </button>
        @endforeach
      </div>
    </aside>
  </div>

  <div class="ky-body">
    <section class="gs-panel ky-sections">
      <div class="gs-panel-title">
        <span class="gs-panel-mark" aria-hidden="true"><i class="bi bi-list-check"></i></span>
        <div>
          <h2>Compliance sections</h2>
          <p>Complete each section with the required information and documents.</p>
        </div>
      </div>
      <div class="ky-list mt-3">
        @foreach ($sections as $section)
          <button
            class="ky-row is-{{ $section['state'] }}"
            type="button"
            data-bs-toggle="modal"
            data-bs-target="#compliance-modal-{{ $section['key'] }}"
          >
            <span class="gs-panel-mark" aria-hidden="true"><i class="bi {{ $section['icon'] }}"></i></span>
            <span class="ky-row-copy">
              <strong>{{ $section['title'] }}</strong>
              <small>{{ $section['detail'] }}</small>
            </span>
            <span class="ky-state is-{{ $section['state'] }}">
              <span class="ky-dot" aria-hidden="true"></span>
              {{ $section['status'] }}
            </span>
            <span class="ky-start">{{ $section['cta'] }}</span>
            <i class="bi bi-chevron-right" aria-hidden="true"></i>
          </button>
        @endforeach
      </div>
    </section>

    <aside class="ky-rail">
      <article class="gs-panel ky-help">
        <div class="gs-panel-title">
          <span class="gs-panel-mark is-help" aria-hidden="true"><i class="bi bi-journal-richtext"></i></span>
          <div>
            <h2>Need help?</h2>
            <p>Read our guides for a detailed explanation of the required information and documents.</p>
          </div>
        </div>
        <a class="gs-outline" href="{{ route('guides') }}">View compliance guide <i class="bi bi-arrow-right" aria-hidden="true"></i></a>
      </article>

      <article class="gs-panel ky-faq">
        <div class="gs-panel-title">
          <span class="gs-panel-mark" aria-hidden="true"><i class="bi bi-question-circle"></i></span>
          <div>
            <h2>Frequently asked questions</h2>
          </div>
        </div>
        @foreach ($questions as $question)
          <details class="ky-faq-item">
            <summary>
              <span>{{ $question['q'] }}</span>
              <i class="bi bi-chevron-down" aria-hidden="true"></i>
            </summary>
            <p>{{ $question['a'] }}</p>
          </details>
        @endforeach
      </article>
    </aside>
  </div>
</div>

@foreach ($sections as $section)
  @php
    $key = $section['key'];
    $data = $compliance[$key]['data'] ?? [];
    $locked = in_array($compliance[$key]['status'] ?? 'none', ['pending', 'approved'], true);
    $formId = 'compliance-form-'.$key;
    $uploaded = is_array($data['files'] ?? null) ? $data['files'] : [];
  @endphp
  <div
    class="modal fade compliance-modal"
    id="compliance-modal-{{ $key }}"
    tabindex="-1"
    aria-labelledby="compliance-title-{{ $key }}"
    aria-hidden="true"
  >
    <div class="modal-dialog modal-dialog-centered modal-lg">
      <div class="modal-content compliance-dialog">
        <div class="modal-header">
          <div class="compliance-dialog-head">
            <span class="gs-panel-mark" aria-hidden="true"><i class="bi {{ $section['icon'] }}"></i></span>
            <div>
              <h2 class="modal-title fs-5" id="compliance-title-{{ $key }}">{{ $section['title'] }}</h2>
              <p>{{ $section['detail'] }}</p>
            </div>
          </div>
          <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
        </div>

        <div class="modal-body">
          @if ($locked)
            <div class="compliance-waiting">
              <span class="compliance-waiting-icon" aria-hidden="true"><i class="bi bi-hourglass-split"></i></span>
              <div>
                <strong>{{ ($compliance[$key]['status'] ?? '') === 'approved' ? 'Approved' : 'Submitted and waiting for approval' }}</strong>
                <p>You can review the details below. Our team will notify you when this section is reviewed.</p>
              </div>
            </div>
          @endif

          <form
            id="{{ $formId }}"
            method="post"
            action="{{ route('compliance.section') }}"
            @if ($key === 'documents') enctype="multipart/form-data" @endif
          >
            @csrf
            <input type="hidden" name="section" value="{{ $key }}">

            <fieldset @disabled($locked)>
              @if ($key === 'business')
                <div class="row g-3">
                  <div class="col-12">
                    <label class="field" for="businessName">Business name
                      <input class="form-control" id="businessName" name="businessName" placeholder="e.g. Hyde Commerce Ltd" value="{{ old('businessName', $data['businessName'] ?? ($profile['displayName'] ?? '')) }}" required>
                    </label>
                  </div>
                  <div class="col-md-6">
                    <label class="field" for="businessType">Business type
                      <select class="form-select" id="businessType" name="businessType" required>
                        <option value="" disabled @selected(old('businessType', $data['businessType'] ?? '') === '')>Select business type</option>
                        @foreach (['Limited company', 'Sole proprietorship', 'Partnership', 'NGO / Association'] as $opt)
                          <option value="{{ $opt }}" @selected(old('businessType', $data['businessType'] ?? '') === $opt)>{{ $opt }}</option>
                        @endforeach
                      </select>
                    </label>
                  </div>
                  <div class="col-md-6">
                    <label class="field" for="preferredPayment">Preferred payment method
                      <select class="form-select" id="preferredPayment" name="preferredPayment" required>
                        <option value="" disabled @selected(old('preferredPayment', $data['preferredPayment'] ?? '') === '')>Select payment method</option>
                        @foreach (['Virtual accounts', 'Checkout / QR', 'Bank transfer', 'Mixed'] as $opt)
                          <option value="{{ $opt }}" @selected(old('preferredPayment', $data['preferredPayment'] ?? '') === $opt)>{{ $opt }}</option>
                        @endforeach
                      </select>
                    </label>
                  </div>
                  <div class="col-12">
                    <label class="field" for="monthlyVolume">Expected monthly volume
                      <select class="form-select" id="monthlyVolume" name="monthlyVolume" required>
                        <option value="" disabled @selected(old('monthlyVolume', $data['monthlyVolume'] ?? '') === '')>Select monthly volume</option>
                        @foreach (['Under ₦1m', '₦1m – ₦10m', '₦10m – ₦50m', 'Above ₦50m'] as $opt)
                          <option value="{{ $opt }}" @selected(old('monthlyVolume', $data['monthlyVolume'] ?? '') === $opt)>{{ $opt }}</option>
                        @endforeach
                      </select>
                    </label>
                  </div>
                </div>
              @elseif ($key === 'representative')
                <div class="row g-3">
                  <div class="col-md-4">
                    <label class="field" for="firstName">First name
                      <input class="form-control" id="firstName" name="firstName" placeholder="e.g. Ada" value="{{ old('firstName', $data['firstName'] ?? '') }}" required>
                    </label>
                  </div>
                  <div class="col-md-4">
                    <label class="field" for="middleName">Middle name
                      <input class="form-control" id="middleName" name="middleName" placeholder="Optional" value="{{ old('middleName', $data['middleName'] ?? '') }}">
                    </label>
                  </div>
                  <div class="col-md-4">
                    <label class="field" for="surname">Surname
                      <input class="form-control" id="surname" name="surname" placeholder="e.g. Lovelace" value="{{ old('surname', $data['surname'] ?? '') }}" required>
                    </label>
                  </div>
                  <div class="col-md-6">
                    <label class="field" for="gender">Gender
                      <select class="form-select" id="gender" name="gender" required>
                        <option value="" disabled @selected(old('gender', $data['gender'] ?? '') === '')>Select gender</option>
                        <option value="M" @selected(old('gender', $data['gender'] ?? 'M') === 'M')>Male</option>
                        <option value="F" @selected(old('gender', $data['gender'] ?? '') === 'F')>Female</option>
                      </select>
                    </label>
                  </div>
                  <div class="col-md-6">
                    <label class="field" for="dob">Date of birth (DD-MM-YYYY)
                      <input class="form-control" id="dob" name="dob" placeholder="15-06-1990" value="{{ old('dob', $data['dob'] ?? '') }}" required>
                    </label>
                  </div>
                  <div class="col-md-6">
                    <label class="field" for="mobileNumber">Phone
                      <input class="form-control" id="mobileNumber" name="mobileNumber" placeholder="e.g. 08012345678" value="{{ old('mobileNumber', $data['mobileNumber'] ?? $mobile) }}" required>
                    </label>
                  </div>
                  <div class="col-md-6">
                    <label class="field" for="email">Email
                      <input class="form-control" id="email" type="email" name="email" placeholder="you@company.com" value="{{ old('email', $data['email'] ?? $email) }}" required>
                    </label>
                  </div>
                  <div class="col-md-6">
                    <label class="field" for="nin">NIN (11 digits)
                      <input class="form-control" id="nin" name="nin" placeholder="12345678901" value="{{ old('nin', $data['nin'] ?? '') }}" required maxlength="11" pattern="\d{11}" inputmode="numeric">
                    </label>
                  </div>
                  <div class="col-md-6">
                    <label class="field" for="state">State
                      <input class="form-control" id="state" name="state" placeholder="e.g. Lagos" value="{{ old('state', $data['state'] ?? '') }}" required>
                    </label>
                  </div>
                  <div class="col-12">
                    <label class="field" for="address">Address
                      <input class="form-control" id="address" name="address" placeholder="Street, city" value="{{ old('address', $data['address'] ?? '') }}" required>
                    </label>
                  </div>
                </div>
              @elseif ($key === 'documents')
                <div class="row g-3">
                  @php
                    $docFields = [
                      'cacCertificate' => ['CAC certificate', true],
                      'utilityBill' => ['Utility bill', true],
                    ];
                  @endphp
                  @foreach ($docFields as $field => [$label, $required])
                    <div class="col-12">
                      <label class="field" for="{{ $field }}">{{ $label }}{{ $required ? '' : ' (optional)' }}
                        @if ($locked && isset($uploaded[$field]['name']))
                          <span class="compliance-file-chip"><i class="bi bi-paperclip" aria-hidden="true"></i> {{ $uploaded[$field]['name'] }}</span>
                        @else
                          <input class="form-control" id="{{ $field }}" name="{{ $field }}" type="file" accept=".pdf,.jpg,.jpeg,.png,application/pdf,image/*" @required($required && ! $locked)>
                          <span class="field-hint">PDF, JPG or PNG · max 5&nbsp;MB</span>
                        @endif
                      </label>
                    </div>
                  @endforeach
                  <div class="col-12">
                    <label class="field" for="notes">Notes
                      <textarea class="form-control" id="notes" name="notes" rows="3" placeholder="Anything reviewers should know about these documents">{{ old('notes', $data['notes'] ?? '') }}</textarea>
                    </label>
                  </div>
                </div>
              @else
                <div class="compliance-declaration">
                  <p>I confirm that the information and documents provided for this Koodify merchant account are true and complete. I authorise Koodify to verify these details with relevant agencies and to keep settlements paused until verification is complete.</p>
                  <div class="row g-3 mt-1">
                    <div class="col-12">
                      <label class="field" for="fullName">Full legal name
                        <input class="form-control" id="fullName" name="fullName" placeholder="Type your full legal name" value="{{ old('fullName', $data['fullName'] ?? '') }}" required>
                      </label>
                    </div>
                    <div class="col-12">
                      <label class="compliance-check">
                        <input type="checkbox" name="accepted" value="1" @checked(old('accepted', $locked)) required>
                        <span>I have read and agree to the compliance declaration.</span>
                      </label>
                    </div>
                  </div>
                </div>
              @endif
            </fieldset>
          </form>
        </div>

        <div class="modal-footer">
          <button type="button" class="btn btn-outline-secondary" data-bs-dismiss="modal">Close</button>
          @unless ($locked)
            <button class="btn btn-primary" type="submit" form="{{ $formId }}">Submit for approval</button>
          @endunless
        </div>
      </div>
    </div>
  </div>
@endforeach
@endsection

@push('head')
<style>
  .ky-row{width:100%;border:0;background:transparent;text-align:left;cursor:pointer;font:inherit;color:inherit}
  .ky-row.is-progress .ky-state{color:#b45309}
  .ky-row.is-progress .ky-dot{background:#f59e0b;box-shadow:0 0 0 4px rgba(245,158,11,.16)}
  .ky-row.is-done .ky-state{color:#047857}
  .ky-row.is-done .ky-dot{background:#10b981;box-shadow:0 0 0 4px rgba(16,185,129,.16)}
  .ky-step-open{width:100%;border:0;background:transparent;text-align:left;cursor:pointer;font:inherit;padding:6px}
  .compliance-modal .modal-dialog{max-width:720px;width:calc(100% - 1.5rem);margin:0.75rem auto;height:auto;max-height:calc(100dvh - 1.5rem)}
  .compliance-modal .modal-content{border:0;border-radius:18px;overflow:hidden;box-shadow:0 28px 56px rgba(11,27,51,.18);height:100%;max-height:calc(100dvh - 1.5rem);display:flex;flex-direction:column}
  .compliance-modal .modal-header,
  .compliance-modal .modal-footer{flex:0 0 auto;background:#fff}
  .compliance-modal .modal-header{align-items:flex-start;border-bottom:1px solid var(--line,#e4eaf2);padding:20px 22px}
  .compliance-modal .modal-body{padding:20px 22px;overflow-y:auto !important;overscroll-behavior:contain;flex:1 1 auto;min-height:0;max-height:none;-webkit-overflow-scrolling:touch}
  .compliance-modal .modal-footer{border-top:1px solid var(--line,#e4eaf2);padding:14px 22px}
  .compliance-dialog-head{display:flex;gap:12px;align-items:flex-start;padding-right:12px}
  .compliance-dialog-head h2{margin:0;color:var(--navy,#0b1b33);letter-spacing:-.02em}
  .compliance-dialog-head p{margin:4px 0 0;color:var(--muted,#546578);font-size:.9rem;line-height:1.4}
  .compliance-waiting{display:flex;gap:12px;align-items:flex-start;padding:14px 16px;border-radius:14px;background:linear-gradient(135deg,rgba(245,158,11,.1),rgba(91,77,255,.06));border:1px solid rgba(245,158,11,.22);margin-bottom:16px}
  .compliance-waiting-icon{width:36px;height:36px;border-radius:10px;display:grid;place-items:center;background:#fff;color:#b45309;flex-shrink:0}
  .compliance-waiting strong{display:block;color:var(--navy,#0b1b33)}
  .compliance-waiting p{margin:4px 0 0;color:var(--muted,#546578);font-size:.88rem}
  .compliance-declaration p{margin:0;color:var(--muted,#546578);line-height:1.55}
  .compliance-check{display:flex;gap:10px;align-items:flex-start;font-weight:600;color:var(--navy,#0b1b33)}
  .compliance-check input{margin-top:3px}
  .field-hint{display:block;margin-top:6px;font-size:.78rem;color:var(--muted,#546578)}
  .compliance-file-chip{display:inline-flex;align-items:center;gap:6px;margin-top:8px;padding:8px 12px;border-radius:10px;background:var(--blue-soft,#eef0ff);color:var(--blue-deep,#4338ca);font-size:.85rem;font-weight:600}
  .compliance-modal.show .modal-content{animation:compliance-pop .28s cubic-bezier(.23,1,.32,1)}
  @keyframes compliance-pop{from{opacity:0;transform:translateY(14px) scale(.97)}to{opacity:1;transform:none}}
  @media (prefers-reduced-motion:reduce){.compliance-modal.show .modal-content{animation:none}}
</style>
@endpush

@push('scripts')
<script>
  document.addEventListener('DOMContentLoaded', function () {
    var open = @json($openSection);
    if (open) {
      var el = document.getElementById('compliance-modal-' + open);
      if (el && window.bootstrap) {
        bootstrap.Modal.getOrCreateInstance(el).show();
      }
    }
  });
</script>
@endpush
