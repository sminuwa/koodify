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
    public function createForm(): View
    {
        return view('virtual-account', [
            'configured' => filled(config('koodify.secret_key')),
            'defaults' => $this->defaults(),
            'result' => session('va_result'),
            'error' => session('va_error'),
        ]);
    }

    public function store(Request $request): RedirectResponse
    {
        $validated = $request->validate([
            'title' => ['required', 'string', 'max:16'],
            'firstName' => ['required', 'string', 'max:80'],
            'surname' => ['required', 'string', 'max:80'],
            'email' => ['required', 'email', 'max:160'],
            'mobileNumber' => ['required', 'string', 'max:20'],
            'bvn' => ['required', 'string', 'size:11'],
            'dob' => ['required', 'string', 'max:20'],
            'gender' => ['required', 'in:M,F'],
            'address' => ['required', 'string', 'max:255'],
            'state' => ['required', 'string', 'max:80'],
        ]);

        // Merchants do not send zainboxCode (comes from sk_…) or bankType
        // (Koodify Adaptive Bank/DVA Routing picks the rail).
        $payload = [
            ...$validated,
            'metadata' => [
                'source' => 'laravel-checkout-sample',
                'createdAt' => now()->toIso8601String(),
            ],
        ];

        try {
            $client = KoodifyClient::fromConfig();
            $result = $client->createVirtualAccount($payload);
        } catch (RuntimeException $e) {
            return back()->withInput()->with('va_error', $e->getMessage());
        } catch (Throwable $e) {
            report($e);

            return back()->withInput()->with('va_error', 'Request failed: '.$e->getMessage());
        }

        return back()->with('va_result', $result);
    }

    /**
     * @return array<string, string>
     */
    private function defaults(): array
    {
        return [
            'title' => 'Ms',
            'firstName' => 'Ada',
            'surname' => 'Lovelace',
            'email' => 'ada@example.com',
            'mobileNumber' => '08012345678',
            'bvn' => '12345678901',
            'dob' => '10-12-1990',
            'gender' => 'F',
            'address' => '12 Computing Lane, Lagos',
            'state' => 'Lagos',
        ];
    }
}
