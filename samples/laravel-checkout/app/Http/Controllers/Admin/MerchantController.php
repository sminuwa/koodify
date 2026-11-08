<?php

namespace App\Http\Controllers\Admin;

use App\Http\Controllers\Controller;
use App\Services\KoodifyAdminClient;
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
            $result = KoodifyAdminClient::fromConfig()->listCustomers();
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
            $client = KoodifyAdminClient::fromConfig();
            $detail = $client->getCustomer($id);
            // Detail may be enveloped by middleware or raw CustomerDetail.
            $profile = $detail['data'] ?? $detail;
            if (! is_array($profile) || ! isset($profile['id'])) {
                return redirect()
                    ->route('admin.merchants.index')
                    ->with('error', $detail['description'] ?? 'Merchant not found.');
            }

            $kyc = $profile['kyc'] ?? null;
            $fullKyc = null;
            try {
                $kycResp = $client->getKycByCustomer($id);
                $fullKyc = $kycResp['data'] ?? $kycResp;
                if (! is_array($fullKyc) || ! isset($fullKyc['id'])) {
                    $fullKyc = null;
                }
            } catch (Throwable) {
                $fullKyc = null;
            }

            return view('admin.merchants.show', [
                'profile' => $profile,
                'kyc' => $kyc,
                'fullKyc' => $fullKyc,
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
            'kycId' => ['required', 'uuid'],
            'status' => ['required', 'in:approved,rejected,pending'],
        ]);

        try {
            $result = KoodifyAdminClient::fromConfig()
                ->updateKycStatus($validated['kycId'], $validated['status']);

            $ok = ($result['code'] ?? '') === '00'
                || isset($result['id'])
                || isset($result['data']['id']);

            if (! $ok && isset($result['description'])) {
                return back()->with('error', (string) $result['description']);
            }

            return redirect()
                ->route('admin.merchants.show', $id)
                ->with('status', 'KYC status updated to '.$validated['status'].'.');
        } catch (Throwable $e) {
            return back()->with('error', $e->getMessage());
        }
    }

    public function provision(int $id): RedirectResponse
    {
        try {
            $result = KoodifyAdminClient::fromConfig()->provisionCustomer($id);
            $desc = $result['description'] ?? $result['data']['description'] ?? 'Provision requested.';

            return redirect()
                ->route('admin.merchants.show', $id)
                ->with('status', is_string($desc) ? $desc : 'Provision complete.');
        } catch (Throwable $e) {
            return back()->with('error', $e->getMessage());
        }
    }
}
