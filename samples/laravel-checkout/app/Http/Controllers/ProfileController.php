<?php

namespace App\Http\Controllers;

use App\Services\KoodifyClient;
use App\Support\MerchantSession;
use Illuminate\Http\RedirectResponse;
use Illuminate\View\View;

class ProfileController extends Controller
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

        return view('merchant.profile', [
            'profile' => $result['data'],
        ]);
    }
}
