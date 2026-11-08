<?php

return [
    'nav' => [
        [
            'heading' => 'Account',
            'items' => [
                ['label' => 'Get Started', 'route' => 'get-started', 'icon' => 'bi-compass'],
                ['label' => 'Dashboard', 'route' => 'dashboard', 'icon' => 'bi-grid'],
                ['label' => 'Transactions', 'route' => 'transactions', 'icon' => 'bi-receipt'],
                ['label' => 'Virtual Accounts', 'route' => 'virtual-accounts.index', 'match' => 'virtual-accounts.index', 'icon' => 'bi-boxes'],
                ['label' => 'Transfers', 'route' => 'transfers', 'icon' => 'bi-arrow-left-right'],
            ],
        ],
        [
            'heading' => 'Developer',
            'items' => [
                ['label' => 'Compliance', 'route' => 'compliance.show', 'match' => 'compliance.*', 'icon' => 'bi-shield-check'],
                ['label' => 'Credentials', 'route' => 'credentials.show', 'match' => 'credentials.*', 'icon' => 'bi-key'],
                ['label' => 'Guides & Tutorials', 'route' => 'guides', 'icon' => 'bi-journal-check'],
                ['label' => 'Payment Simulation', 'route' => 'payment-simulation', 'icon' => 'bi-play-circle'],
                ['label' => 'API Docs', 'url' => 'https://docs.koodify.ng/', 'icon' => 'bi-journal-text'],
                ['label' => 'Webhooks', 'route' => 'webhooks', 'icon' => 'bi-cloud-arrow-up'],
            ],
        ],
        [
            'heading' => 'Payments',
            'items' => [
                ['label' => 'Accept Payments', 'route' => 'accept-payments', 'icon' => 'bi-credit-card'],
                ['label' => 'Checkout demo', 'route' => 'checkout', 'icon' => 'bi-qr-code'],
            ],
        ],
        [
            'heading' => 'Account settings',
            'items' => [
                ['label' => 'Profile', 'route' => 'profile', 'icon' => 'bi-person'],
                ['label' => 'Logout', 'action' => 'logout', 'icon' => 'bi-box-arrow-right'],
            ],
        ],
    ],

    'pages' => [
        'transactions' => [
            'Transactions',
            'Recent payment activity.',
            'Transaction history will appear here once collection APIs are wired for this merchant.',
        ],
        'transfers' => [
            'Transfers',
            'Move funds from this account.',
            'Send money and follow each transfer through to a final status. Coming next for the Koodify merchant portal.',
        ],
        'guides' => [
            'Guides & Tutorials',
            'Learn the integration path.',
            'Walk through KYC, credentials, checkout, and virtual accounts with short tutorials.',
        ],
        'accept-payments' => [
            'Accept Payments',
            'Request a payment with a link or QR code.',
            'Share a checkout link or use the inline demo to collect from your customers.',
        ],
        'webhooks' => [
            'Webhooks',
            'Hear about events on this account.',
            'Point Koodify at your endpoint and validate each delivery. Configure callbacks from your integration.',
        ],
        'profile' => [
            'Profile',
            'Your merchant account details.',
            'Review business name, email, and session status for this portal login.',
        ],
    ],
];
