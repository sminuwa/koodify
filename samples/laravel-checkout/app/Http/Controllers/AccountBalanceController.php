<?php

namespace App\Http\Controllers;

use App\Services\KoodifyClient;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\View\View;
use RuntimeException;
use Throwable;

class AccountBalanceController extends Controller
{
    public function showForm(): View
    {
        return view('account-balance', [
            'configured' => filled(config('koodify.secret_key')),
            'result' => session('balance_result'),
            'error' => session('balance_error'),
            'accountNumber' => old('accountNumber', session('balance_account')),
        ]);
    }

    public function lookup(Request $request): RedirectResponse
    {
        $validated = $request->validate([
            'accountNumber' => ['required', 'string', 'max:32'],
        ]);

        $accountNumber = preg_replace('/\s+/', '', (string) $validated['accountNumber']) ?? '';

        try {
            $client = KoodifyClient::fromConfig();
            $result = $client->getVirtualAccountBalance($accountNumber);
        } catch (RuntimeException $e) {
            return back()
                ->withInput()
                ->with('balance_error', $e->getMessage())
                ->with('balance_account', $accountNumber);
        } catch (Throwable $e) {
            report($e);

            return back()
                ->withInput()
                ->with('balance_error', 'Request failed: '.$e->getMessage())
                ->with('balance_account', $accountNumber);
        }

        return back()
            ->with('balance_result', $result)
            ->with('balance_account', $accountNumber);
    }
}
