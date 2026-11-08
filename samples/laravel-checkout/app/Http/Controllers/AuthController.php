<?php

namespace App\Http\Controllers;

use App\Services\KoodifyClient;
use App\Support\MerchantSession;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\View\View;

class AuthController extends Controller
{
    public function showRegister(): View
    {
        return view('auth.register');
    }

    public function register(Request $request): RedirectResponse
    {
        $validated = $request->validate([
            'displayName' => ['required', 'string', 'max:120'],
            'email' => ['required', 'email', 'max:190'],
            'mobileNumber' => ['nullable', 'string', 'max:32'],
            'password' => ['required', 'string', 'min:8', 'confirmed'],
        ]);

        $client = KoodifyClient::public();
        $result = $client->register([
            'displayName' => $validated['displayName'],
            'email' => $validated['email'],
            'mobileNumber' => $validated['mobileNumber'] ?? null,
            'password' => $validated['password'],
        ]);

        if (($result['code'] ?? '') !== '00' || ! is_array($result['data'] ?? null)) {
            return back()
                ->withInput()
                ->with('error', $result['description'] ?? 'Registration failed.');
        }

        MerchantSession::storeLogin($result['data']);

        return redirect()
            ->route('get-started')
            ->with('status', 'Account created. Submit KYC to get your API keys.');
    }

    public function showLogin(): View
    {
        return view('auth.login');
    }

    public function login(Request $request): RedirectResponse
    {
        $validated = $request->validate([
            'email' => ['required', 'email'],
            'password' => ['required', 'string'],
        ]);

        $result = KoodifyClient::public()->login($validated);

        if (($result['code'] ?? '') !== '00' || ! is_array($result['data'] ?? null)) {
            return back()
                ->withInput()
                ->with('error', $result['description'] ?? 'Login failed.');
        }

        MerchantSession::storeLogin($result['data']);

        return redirect()->route('get-started');
    }

    public function logout(): RedirectResponse
    {
        MerchantSession::clear();

        return redirect()->route('login')->with('status', 'Signed out.');
    }
}
