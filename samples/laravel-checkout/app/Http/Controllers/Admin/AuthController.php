<?php

namespace App\Http\Controllers\Admin;

use App\Http\Controllers\Controller;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\View\View;

class AuthController extends Controller
{
    public function showLogin(): View|RedirectResponse
    {
        if (session('admin.authenticated')) {
            return redirect()->route('admin.dashboard');
        }

        return view('admin.auth.login');
    }

    public function login(Request $request): RedirectResponse
    {
        $validated = $request->validate([
            'email' => ['required', 'email'],
            'password' => ['required', 'string'],
        ]);

        $email = strtolower(trim($validated['email']));
        $password = $validated['password'];
        $expectedEmail = strtolower((string) config('koodify.admin_email'));
        $expectedPassword = (string) config('koodify.admin_password');

        if ($expectedEmail === '' || $expectedPassword === '') {
            return back()
                ->withInput()
                ->with('error', 'Configure KOODIFY_ADMIN_EMAIL and KOODIFY_ADMIN_PASSWORD in .env.');
        }

        if ($email !== $expectedEmail || ! hash_equals($expectedPassword, $password)) {
            return back()
                ->withInput()
                ->with('error', 'We could not verify those details.');
        }

        if ((string) config('koodify.gateway_api_key') === '') {
            return back()
                ->withInput()
                ->with('error', 'Set KOODIFY_GATEWAY_API_KEY so the portal can call Koodify admin APIs.');
        }

        session([
            'admin.authenticated' => true,
            'admin.email' => $email,
            'admin.name' => (string) config('koodify.admin_name', 'Koodify Admin'),
        ]);

        return redirect()->route('admin.dashboard');
    }

    public function logout(): RedirectResponse
    {
        session()->forget(['admin.authenticated', 'admin.email', 'admin.name']);

        return redirect()->route('admin.login')->with('status', 'Signed out.');
    }
}
