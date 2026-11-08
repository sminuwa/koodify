<?php

namespace App\Http\Controllers;

use App\Services\KoodifyClient;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Str;
use RuntimeException;
use Throwable;

class DynamicVirtualAccountController extends Controller
{
    public function createForm(): RedirectResponse
    {
        return redirect()->route('payment-simulation', ['tool' => 'dva']);
    }

    public function store(Request $request): RedirectResponse
    {
        $validated = $request->validate([
            'email' => ['required', 'email', 'max:160'],
            'amount' => ['required', 'string', 'max:32'],
            'txnRef' => ['nullable', 'string', 'max:80'],
            'duration' => ['required', 'integer', 'min:300', 'max:259200'],
            'callBackUrl' => ['nullable', 'url', 'max:500'],
        ]);

        $txnRef = filled($validated['txnRef'] ?? null)
            ? (string) $validated['txnRef']
            : 'DVA_'.Str::upper(Str::random(12));

        $callBackUrl = filled($validated['callBackUrl'] ?? null)
            ? (string) $validated['callBackUrl']
            : (string) (config('koodify.callback_url') ?: url('/webhooks/koodify'));

        $payload = [
            'email' => $validated['email'],
            'amount' => (string) $validated['amount'],
            'txnRef' => $txnRef,
            'duration' => (int) $validated['duration'],
            'callBackUrl' => $callBackUrl,
            'metadata' => [
                'source' => 'laravel-checkout-sample',
                'createdAt' => now()->toIso8601String(),
            ],
        ];

        $redirect = redirect()->route('payment-simulation', ['tool' => 'dva']);

        try {
            $client = KoodifyClient::forMerchantApis();
            $result = $client->createDynamicVirtualAccount($payload);
        } catch (RuntimeException $e) {
            return $redirect->withInput()->with('dva_error', $e->getMessage());
        } catch (Throwable $e) {
            report($e);

            return $redirect->withInput()->with('dva_error', 'Request failed: '.$e->getMessage());
        }

        return $redirect->with('dva_result', $result);
    }

    /**
     * @return array<string, string>
     */
    private function defaults(): array
    {
        return [
            'email' => 'ada@example.com',
            'amount' => (string) config('koodify.demo_amount', '50000'),
            'txnRef' => 'DVA_'.Str::upper(Str::random(10)),
            'duration' => '900',
            'callBackUrl' => (string) (config('koodify.callback_url') ?: url('/webhooks/koodify')),
        ];
    }
}
