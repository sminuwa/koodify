<?php

namespace App\Http\Controllers;

use App\Services\KoodifyClient;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\View\View;
use RuntimeException;
use Throwable;

class VirtualAccountController extends Controller
{
    public function index(): View
    {
        $configured = filled(session('merchant.secret_key')) || filled(config('koodify.secret_key'));
        $accounts = [];
        $error = null;
        $result = null;

        if ($configured) {
            try {
                $result = KoodifyClient::forMerchantApis()->listVirtualAccounts();
                $data = $result['data'] ?? null;
                if (is_array($data)) {
                    $accounts = array_values(array_filter($data, 'is_array'));
                } elseif (($result['code'] ?? '') !== '00' && ($result['status'] ?? '') !== '200 OK') {
                    $error = $result['description'] ?? 'Could not load virtual accounts.';
                }
            } catch (RuntimeException $e) {
                $error = $e->getMessage();
            } catch (Throwable $e) {
                report($e);
                $error = 'Request failed: '.$e->getMessage();
            }
        }

        return view('merchant.virtual-accounts', [
            'configured' => $configured,
            'accounts' => $accounts,
            'error' => $error,
            'raw' => $result,
        ]);
    }

    public function createForm(): RedirectResponse
    {
        return redirect()->route('payment-simulation', ['tool' => 'va']);
    }

    public function store(Request $request): RedirectResponse
    {
        $validated = $request->validate([
            'title' => ['required', 'string', 'max:16'],
            'firstName' => ['required', 'string', 'max:80'],
            'surname' => ['required', 'string', 'max:80'],
            'email' => ['required', 'email', 'max:160'],
            'mobileNumber' => ['required', 'string', 'max:20'],
            'address' => ['required', 'string', 'max:255'],
            'state' => ['required', 'string', 'max:80'],
        ]);

        $payload = [
            ...$validated,
            'metadata' => [
                'source' => 'laravel-checkout-sample',
                'createdAt' => now()->toIso8601String(),
            ],
        ];

        $redirect = redirect()->route('payment-simulation', ['tool' => 'va']);

        try {
            $client = KoodifyClient::forMerchantApis();
            $result = $client->createVirtualAccount($payload);
        } catch (RuntimeException $e) {
            return $redirect->withInput()->with('va_error', $e->getMessage());
        } catch (Throwable $e) {
            report($e);

            return $redirect->withInput()->with('va_error', 'Request failed: '.$e->getMessage());
        }

        return $redirect->with('va_result', $result);
    }
}
