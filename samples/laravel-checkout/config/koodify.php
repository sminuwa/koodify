<?php

return [

    /*
    |--------------------------------------------------------------------------
    | Publishable public key (browser)
    |--------------------------------------------------------------------------
    |
    | Used only by the inline checkout script. Never put sk_… keys here.
    |
    */
    'public_key' => env('KOODIFY_PUBLIC_KEY', ''),

    /*
    |--------------------------------------------------------------------------
    | Secret key (server)
    |--------------------------------------------------------------------------
    |
    | Merchant sk_… for:
    | - X-Api-Key on server routes (VA create, wallet, …)
    | - verifying X-Koodify-Signature on inbound webhooks (HMAC-SHA256 of raw body)
    |
    | Never expose in browser / JS.
    |
    */
    'secret_key' => env('KOODIFY_SECRET_KEY', ''),

    /*
    |--------------------------------------------------------------------------
    | API base URL
    |--------------------------------------------------------------------------
    */
    'api_base' => env('KOODIFY_API_BASE', 'https://api.koodify.ng'),

    /*
    |--------------------------------------------------------------------------
    | Inline checkout script
    |--------------------------------------------------------------------------
    */
    // Sample ships public/vendor/koodify-inline.js. Override with the CDN URL in production.
    'inline_js' => env('KOODIFY_INLINE_JS', '/vendor/koodify-inline.js'),

    /*
    |--------------------------------------------------------------------------
    | Merchant webhook callback
    |--------------------------------------------------------------------------
    |
    | Must be a publicly reachable HTTPS URL in production. For local demos,
    | expose `php artisan serve` with a tunnel (e.g. ngrok) and set APP_URL /
    | KOODIFY_CALLBACK_URL to that public origin.
    |
    */
    'callback_url' => env('KOODIFY_CALLBACK_URL'),

    /*
    |--------------------------------------------------------------------------
    | Demo defaults
    |--------------------------------------------------------------------------
    */
    'demo_amount' => env('KOODIFY_DEMO_AMOUNT', '50000'),

];
