<?php

namespace App\Http\Controllers;

use Illuminate\View\View;

class CheckoutController extends Controller
{
    public function __invoke(): View
    {
        $callbackUrl = config('koodify.callback_url') ?: url('/webhooks/koodify');
        $publicKey = session('merchant.public_key') ?: config('koodify.public_key');

        return view('checkout', [
            'publicKey' => $publicKey,
            'inlineJs' => config('koodify.inline_js'),
            'callbackUrl' => $callbackUrl,
            'demoAmount' => config('koodify.demo_amount'),
            'configured' => filled($publicKey),
        ]);
    }
}
