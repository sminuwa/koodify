<?php

return [

    /*
    |--------------------------------------------------------------------------
    | API base URL
    |--------------------------------------------------------------------------
    |
    | Merchant auth + KYC + credentials call this host. Prefer local gateway
    | while developing: http://127.0.0.1:8080
    |
    */
    'api_base' => env('KOODIFY_API_BASE', 'https://api.koodify.ng'),

    /*
    |--------------------------------------------------------------------------
    | Optional key fallbacks
    |--------------------------------------------------------------------------
    |
    | After register → KYC → Credentials, the app prefers keys from the merchant
    | session. These .env values remain useful for webhooks and offline demos.
    | Never put sk_… in the browser.
    |
    */
    'public_key' => env('KOODIFY_PUBLIC_KEY', ''),
    'secret_key' => env('KOODIFY_SECRET_KEY', ''),

    /*
    |--------------------------------------------------------------------------
    | Platform admin portal
    |--------------------------------------------------------------------------
    |
    | GATEWAY_API_KEY from the Koodify gateway — used as X-Api-Key for
    | /customers, /kyc, provision, etc. Portal login uses admin_email/password.
    |
    */
    'gateway_api_key' => env('KOODIFY_GATEWAY_API_KEY', ''),
    'admin_email' => env('KOODIFY_ADMIN_EMAIL', 'admin@koodify.ng'),
    'admin_password' => env('KOODIFY_ADMIN_PASSWORD', ''),
    'admin_name' => env('KOODIFY_ADMIN_NAME', 'Koodify Admin'),

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
