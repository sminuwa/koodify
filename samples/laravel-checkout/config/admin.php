<?php

return [
    'nav' => [
        [
            'heading' => 'Workspace',
            'items' => [
                ['label' => 'Overview', 'route' => 'admin.dashboard', 'match' => 'admin.dashboard', 'icon' => 'bi-grid'],
                ['label' => 'Merchants', 'route' => 'admin.merchants.index', 'match' => 'admin.merchants.*', 'icon' => 'bi-shop'],
                ['label' => 'Transactions', 'route' => 'admin.section', 'params' => ['section' => 'transactions'], 'match' => null, 'section' => 'transactions', 'icon' => 'bi-receipt'],
                ['label' => 'Customers', 'route' => 'admin.section', 'params' => ['section' => 'customers'], 'section' => 'customers', 'icon' => 'bi-people'],
                ['label' => 'Services', 'route' => 'admin.section', 'params' => ['section' => 'services'], 'section' => 'services', 'icon' => 'bi-boxes'],
                ['label' => 'Reports', 'route' => 'admin.section', 'params' => ['section' => 'reports'], 'section' => 'reports', 'icon' => 'bi-bar-chart'],
            ],
        ],
        [
            'heading' => 'Preferences',
            'items' => [
                ['label' => 'Settings', 'route' => 'admin.section', 'params' => ['section' => 'settings'], 'section' => 'settings', 'icon' => 'bi-gear'],
                ['label' => 'Activity log', 'route' => 'admin.section', 'params' => ['section' => 'activity'], 'section' => 'activity', 'icon' => 'bi-clock-history'],
                ['label' => 'Logout', 'action' => 'logout', 'icon' => 'bi-box-arrow-right'],
            ],
        ],
    ],
];
