<?php

namespace App\Http\Controllers\Admin;

use App\Http\Controllers\Controller;
use Illuminate\View\View;

/** Placeholder sections from the admin portal prototype. */
class SectionController extends Controller
{
    public function __invoke(string $section): View
    {
        $titles = [
            'transactions' => 'Transactions',
            'customers' => 'Customers',
            'services' => 'Services',
            'reports' => 'Reports',
            'settings' => 'Settings',
            'activity' => 'Activity log',
        ];

        abort_unless(isset($titles[$section]), 404);

        return view('admin.section', [
            'section' => $section,
            'title' => $titles[$section],
        ]);
    }
}
