<?php

namespace App\Http\Controllers;

use App\Services\KoodifyClient;
use App\Support\ComplianceProgress;
use App\Support\MerchantSession;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\View\View;

class KycController extends Controller
{
    public function showForm(): View|RedirectResponse
    {
        $profile = $this->freshProfile();
        if ($profile instanceof RedirectResponse) {
            return $profile;
        }

        $compliance = ComplianceProgress::fromGateway(
            is_array($profile['compliance'] ?? null) ? $profile['compliance'] : null
        );

        return view('kyc.form', [
            'profile' => $profile,
            'email' => $profile['email'] ?? session('merchant.email'),
            'mobile' => $profile['mobileNumber'] ?? '',
            'compliance' => $compliance,
            'openSection' => session('compliance_open'),
        ]);
    }

    public function showVerify(): RedirectResponse
    {
        return redirect()->route('compliance.show', ['section' => 'representative']);
    }

    public function submitSection(Request $request): RedirectResponse
    {
        $section = (string) $request->input('section');
        if (! in_array($section, ComplianceProgress::SECTIONS, true)) {
            return back()->with('error', 'Unknown compliance section.');
        }

        $client = KoodifyClient::withMerchantToken((string) session('merchant.token'));

        if ($section === 'representative') {
            $validated = $request->validate([
                'firstName' => ['required', 'string', 'max:80'],
                'middleName' => ['nullable', 'string', 'max:80'],
                'surname' => ['required', 'string', 'max:80'],
                'email' => ['required', 'email', 'max:190'],
                'mobileNumber' => ['required', 'string', 'max:32'],
                'nin' => ['required', 'string', 'size:11', 'regex:/^\d{11}$/'],
                'dob' => ['required', 'string', 'max:20'],
                'gender' => ['required', 'in:M,F'],
                'address' => ['required', 'string', 'max:255'],
                'state' => ['required', 'string', 'max:80'],
            ]);

            $nin = (string) $validated['nin'];
            $title = ($validated['gender'] ?? 'M') === 'F' ? 'Ms' : 'Mr';
            $payload = [
                'title' => $title,
                'firstName' => $validated['firstName'],
                'middleName' => $validated['middleName'] ?? null,
                'surname' => $validated['surname'],
                'email' => $validated['email'],
                'mobileNumber' => $validated['mobileNumber'],
                // Gateway accepts `nin` (aliased onto kyc_profiles.bvn) — do not also send `bvn`.
                'nin' => $nin,
                'dob' => $validated['dob'],
                'gender' => $validated['gender'],
                'address' => $validated['address'],
                'state' => $validated['state'],
                'metadata' => [
                    'source' => 'laravel-checkout-compliance',
                    'identityType' => 'nin',
                ],
            ];

            $result = $client->submitKyc($payload);
            if (($result['code'] ?? '') !== '00') {
                return back()
                    ->withInput()
                    ->with('error', $result['description'] ?? 'KYC submission failed.')
                    ->with('compliance_open', 'representative');
            }

            $data = is_array($result['data'] ?? null) ? $result['data'] : [];
            MerchantSession::captureSecrets($data);
            $status = (string) ($data['status'] ?? 'pending');
            $message = $status === 'approved'
                ? 'Representative KYC approved.'
                : 'Company Representative submitted and waiting for approval.';

            return redirect()->route('compliance.show')->with('status', $message);
        }

        if ($section === 'documents') {
            $request->validate([
                'cacCertificate' => ['required', 'file', 'mimes:pdf,jpg,jpeg,png', 'max:5120'],
                'utilityBill' => ['required', 'file', 'mimes:pdf,jpg,jpeg,png', 'max:5120'],
                'notes' => ['nullable', 'string', 'max:500'],
            ]);

            $result = $client->submitComplianceDocuments([
                'cacCertificate' => $request->file('cacCertificate'),
                'utilityBill' => $request->file('utilityBill'),
                'notes' => (string) $request->input('notes', ''),
            ]);

            if (($result['code'] ?? '') !== '00') {
                return back()
                    ->withInput()
                    ->with('error', $result['description'] ?? 'Document upload failed.')
                    ->with('compliance_open', 'documents');
            }

            return redirect()
                ->route('compliance.show')
                ->with('status', 'Business Documents submitted and waiting for approval.');
        }

        $rules = match ($section) {
            'business' => [
                'businessName' => ['required', 'string', 'max:160'],
                'businessType' => ['required', 'string', 'max:80'],
                'preferredPayment' => ['required', 'string', 'max:80'],
                'monthlyVolume' => ['required', 'string', 'max:80'],
            ],
            'declaration' => [
                'fullName' => ['required', 'string', 'max:160'],
                'accepted' => ['accepted'],
            ],
            default => [],
        };

        $validated = $request->validate($rules);
        $result = $client->submitComplianceSection($section, $validated);

        if (($result['code'] ?? '') !== '00') {
            return back()
                ->withInput()
                ->with('error', $result['description'] ?? 'Compliance submission failed.')
                ->with('compliance_open', $section);
        }

        if ($section === 'business') {
            $name = trim((string) ($validated['businessName'] ?? ''));
            if ($name !== '') {
                session([
                    'merchant.display_name' => $name,
                    'merchant.profile.displayName' => $name,
                ]);
            }
        }

        $labels = [
            'business' => 'Business Information',
            'declaration' => 'Declaration',
        ];

        return redirect()
            ->route('compliance.show')
            ->with('status', ($labels[$section] ?? 'Section').' submitted and waiting for approval.');
    }

    /** @deprecated Prefer submitSection. */
    public function submit(Request $request): RedirectResponse
    {
        $request->merge(['section' => 'representative']);

        return $this->submitSection($request);
    }

    /**
     * @return array<string, mixed>|RedirectResponse
     */
    private function freshProfile(): array|RedirectResponse
    {
        $result = KoodifyClient::withMerchantToken((string) session('merchant.token'))->me();

        if (($result['code'] ?? '') !== '00' || ! is_array($result['data'] ?? null)) {
            MerchantSession::clear();

            return redirect()
                ->route('login')
                ->with('error', $result['description'] ?? 'Session expired. Log in again.');
        }

        $profile = $result['data'];
        session([
            'merchant.profile' => $profile,
            'merchant.display_name' => $profile['displayName'] ?? session('merchant.display_name'),
            'merchant.public_key' => $profile['publicKey'] ?? session('merchant.public_key'),
        ]);

        return $profile;
    }
}
