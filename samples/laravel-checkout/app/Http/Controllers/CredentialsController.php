<?php

namespace App\Http\Controllers;

use App\Services\KoodifyClient;
use App\Support\MerchantSession;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\View\View;

class CredentialsController extends Controller
{
    public function show(): View
    {
        $me = KoodifyClient::withMerchantToken((string) session('merchant.token'))->me();
        $profile = is_array($me['data'] ?? null) ? $me['data'] : [];

        return view('credentials', [
            'profile' => $profile,
            'credentials' => $profile['credentials'] ?? [],
            'publicKey' => $profile['publicKey'] ?? session('merchant.public_key'),
            'secretLive' => session('merchant.secret_live'),
            'secretTest' => session('merchant.secret_test'),
            'secretKey' => session('merchant.secret_key'),
        ]);
    }

    public function issue(Request $request): RedirectResponse
    {
        $environment = $request->validate([
            'environment' => ['required', 'in:test,live'],
        ])['environment'];

        $result = KoodifyClient::withMerchantToken((string) session('merchant.token'))
            ->issueSecret($environment);

        if (($result['code'] ?? '') !== '00' || ! is_array($result['data'] ?? null)) {
            return back()->with('error', $result['description'] ?? 'Could not issue secret key.');
        }

        MerchantSession::captureSecrets($result['data']);

        $rotated = ! empty($result['data']['rotated']);
        $msg = $rotated
            ? 'Secret key rotated. Copy it now — it will not be shown again.'
            : 'Secret key generated. Copy it now — it will not be shown again.';

        return back()->with('status', $msg);
    }
}
