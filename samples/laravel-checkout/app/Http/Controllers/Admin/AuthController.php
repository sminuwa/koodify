<?php

namespace App\Http\Controllers\Admin;

use App\Http\Controllers\Controller;
use App\Services\KoodifyAdminClient;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\View\View;
use Throwable;

class AuthController extends Controller
{
    public function showLogin(): View|RedirectResponse
    {
        if (session('admin.token')) {
            return redirect()->route('admin.dashboard');
        }

        return view('admin.auth.login');
    }

    public function login(Request $request): RedirectResponse
    {
        $validated = $request->validate([
            'username' => ['required', 'string', 'max:120'],
            'password' => ['required', 'string'],
        ]);

        try {
            $result = KoodifyAdminClient::login(
                $validated['username'],
                $validated['password'],
            );
        } catch (Throwable $e) {
            return back()
                ->withInput()
                ->with('error', 'Could not reach Koodify: '.$e->getMessage());
        }

        if (($result['code'] ?? '') !== '00' || ! is_array($result['data'] ?? null)) {
            return back()
                ->withInput()
                ->with('error', $result['description'] ?? 'We could not verify those details.');
        }

        $data = $result['data'];
        $user = is_array($data['user'] ?? null) ? $data['user'] : [];

        session([
            'admin.authenticated' => true,
            'admin.token' => (string) ($data['token'] ?? ''),
            'admin.username' => (string) ($user['username'] ?? $validated['username']),
            'admin.name' => (string) ($user['displayName'] ?? $user['username'] ?? 'Admin'),
            'admin.email' => (string) ($user['email'] ?? ''),
        ]);

        return redirect()->route('admin.dashboard');
    }

    public function logout(): RedirectResponse
    {
        session()->forget([
            'admin.authenticated',
            'admin.token',
            'admin.username',
            'admin.name',
            'admin.email',
        ]);

        return redirect()->route('admin.login')->with('status', 'Signed out.');
    }
}
