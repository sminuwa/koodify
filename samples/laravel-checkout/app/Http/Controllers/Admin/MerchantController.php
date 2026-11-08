<?php

namespace App\Http\Controllers\Admin;

use App\Http\Controllers\Controller;
use App\Services\KoodifyAdminClient;
use App\Support\ComplianceProgress;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\View\View;
use Throwable;

class MerchantController extends Controller
{
    public function index(): View
    {
        $customers = [];
        $error = null;

        try {
            $result = KoodifyAdminClient::fromSession()->listCustomers();
            $rows = $result['data'] ?? $result;
            $customers = is_array($rows) && array_is_list($rows) ? $rows : [];
            if ($customers === [] && isset($result['description']) && ($result['code'] ?? '') !== '00') {
                $error = (string) $result['description'];
            }
        } catch (Throwable $e) {
            $error = $e->getMessage();
        }

        return view('admin.merchants.index', [
            'customers' => $customers,
            'error' => $error,
        ]);
    }

    public function show(int $id): View|RedirectResponse
    {
        try {
            $client = KoodifyAdminClient::fromSession();
            $detail = $client->getCustomer($id);
            $profile = $detail['data'] ?? $detail;
            if (! is_array($profile) || ! isset($profile['id'])) {
                return redirect()
                    ->route('admin.merchants.index')
                    ->with('error', $detail['description'] ?? 'Merchant not found.');
            }

            $compliance = ComplianceProgress::fromGateway(
                is_array($profile['compliance'] ?? null) ? $profile['compliance'] : null
            );

            return view('admin.merchants.show', [
                'profile' => $profile,
                'kyc' => $profile['kyc'] ?? null,
                'fullKyc' => is_array($profile['kyc'] ?? null) ? $profile['kyc'] : null,
                'compliance' => $compliance,
            ]);
        } catch (Throwable $e) {
            return redirect()
                ->route('admin.merchants.index')
                ->with('error', $e->getMessage());
        }
    }

    public function approveKyc(Request $request, int $id): RedirectResponse
    {
        $validated = $request->validate([
            'action' => ['required', 'in:approved,rejected'],
        ]);

        try {
            $client = KoodifyAdminClient::fromSession();
            $result = $validated['action'] === 'approved'
                ? $client->approveMerchantKyc($id)
                : $client->rejectMerchantKyc($id);

            if (($result['code'] ?? '') !== '00') {
                return back()->with('error', $result['description'] ?? 'KYC update failed.');
            }

            $msg = $validated['action'] === 'approved'
                ? 'KYC approved. Merchant emailed (if Resend is configured) and zainbox provisioned. They can generate sk_… from Credentials.'
                : 'KYC rejected.';

            return redirect()
                ->route('admin.merchants.show', $id)
                ->with('status', $msg);
        } catch (Throwable $e) {
            return back()->with('error', $e->getMessage());
        }
    }

    public function approveSection(Request $request, int $id): RedirectResponse
    {
        $validated = $request->validate([
            'section' => ['required', 'in:business,representative,documents,declaration'],
            'status' => ['required', 'in:approved,rejected,pending'],
        ]);

        try {
            $result = KoodifyAdminClient::fromSession()->setComplianceSection(
                $id,
                $validated['section'],
                $validated['status'],
            );

            if (($result['code'] ?? '') !== '00') {
                return back()->with('error', $result['description'] ?? 'Section update failed.');
            }

            return back()->with(
                'status',
                ucfirst($validated['section']).' marked '.$validated['status'].'.'
            );
        } catch (Throwable $e) {
            return back()->with('error', $e->getMessage());
        }
    }

    public function provision(int $id): RedirectResponse
    {
        try {
            $result = KoodifyAdminClient::fromSession()->provisionCustomer($id);
            $desc = $result['description'] ?? $result['data']['description'] ?? 'Provision requested.';

            return redirect()
                ->route('admin.merchants.show', $id)
                ->with('status', is_string($desc) ? $desc : 'Provision complete.');
        } catch (Throwable $e) {
            return back()->with('error', $e->getMessage());
        }
    }
}
