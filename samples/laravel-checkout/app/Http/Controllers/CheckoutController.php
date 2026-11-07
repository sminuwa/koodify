<?php

namespace App\Http\Controllers;

use Illuminate\View\View;

class CheckoutController extends Controller
{
    public function __invoke(): View
    {
        $callbackUrl = config('koodify.callback_url') ?: url('/webhooks/koodify');

        return view('checkout', [
            'publicKey' => config('koodify.public_key'),
            'inlineJs' => config('koodify.inline_js'),
            'callbackUrl' => $callbackUrl,
            'demoAmount' => config('koodify.demo_amount'),
            'configured' => filled(config('koodify.public_key')),
        ]);
    }
}
