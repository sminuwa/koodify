<?php

namespace App\Http\Controllers;

use App\Services\KoodifyClient;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use RuntimeException;
use Throwable;

class AccountBalanceController extends Controller
{
    public function showForm(): RedirectResponse
    {
        return redirect()->route('payment-simulation', ['tool' => 'balance']);
    }

    public function lookup(Request $request): RedirectResponse
    {
        $validated = $request->validate([
            'accountNumber' => ['required', 'string', 'max:32'],
        ]);

        $accountNumber = preg_replace('/\s+/', '', (string) $validated['accountNumber']) ?? '';
        $redirect = redirect()->route('payment-simulation', ['tool' => 'balance']);

        try {
            $client = KoodifyClient::forMerchantApis();
            $result = $client->getVirtualAccountBalance($accountNumber);
        } catch (RuntimeException $e) {
            return $redirect
                ->withInput()
                ->with('balance_error', $e->getMessage())
                ->with('balance_account', $accountNumber);
        } catch (Throwable $e) {
            report($e);

            return $redirect
                ->withInput()
                ->with('balance_error', 'Request failed: '.$e->getMessage())
                ->with('balance_account', $accountNumber);
        }

        return $redirect
            ->with('balance_result', $result)
            ->with('balance_account', $accountNumber);
    }
}
