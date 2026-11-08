<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

class EnsureMerchantSession
{
    public function handle(Request $request, Closure $next): Response
    {
        if (! session()->has('merchant.token')) {
            return redirect()
                ->route('login')
                ->with('error', 'Log in to continue.');
        }

        return $next($request);
    }
}
