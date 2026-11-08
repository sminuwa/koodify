<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use Illuminate\View\View;

class MerchantPageController extends Controller
{
    public function show(Request $request, ?string $page = null): View
    {
        $page = $page ?: (string) $request->route('page');
        $entry = config('merchant.pages.'.$page);
        abort_unless(is_array($entry), 404);

        return view('merchant.page', [
            'slug' => $page,
            'title' => $entry[0],
            'subtitle' => $entry[1],
            'description' => $entry[2],
        ]);
    }
}
