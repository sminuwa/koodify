<?php

namespace App\Http\Controllers;

use App\Services\KoodifyClient;
use App\Support\MerchantSession;
use Illuminate\Http\RedirectResponse;
use Illuminate\View\View;

class GetStartedController extends Controller
{
    public function __invoke(): View|RedirectResponse
    {
        $result = KoodifyClient::withMerchantToken((string) session('merchant.token'))->me();

        if (($result['code'] ?? '') !== '00' || ! is_array($result['data'] ?? null)) {
            MerchantSession::clear();

            return redirect()
                ->route('login')
                ->with('error', $result['description'] ?? 'Session expired. Log in again.');
        }

        $profile = $result['data'];
        session([
            'merchant.profile' => $profile,
            'merchant.display_name' => $profile['displayName'] ?? session('merchant.display_name'),
            'merchant.public_key' => $profile['publicKey'] ?? session('merchant.public_key'),
        ]);

        return view('dashboard', [
            'profile' => $profile,
            'kyc' => $profile['kyc'] ?? null,
            'credentials' => $profile['credentials'] ?? [],
            'hasSecret' => (string) session('merchant.secret_key', '') !== '',
        ]);
    }
}
