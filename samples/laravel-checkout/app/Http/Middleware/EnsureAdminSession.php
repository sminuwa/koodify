<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

class EnsureAdminSession
{
    public function handle(Request $request, Closure $next): Response
    {
        if (! session('admin.token') || ! session('admin.authenticated')) {
            return redirect()
                ->route('admin.login')
                ->with('error', 'Sign in to the admin portal.');
        }

        return $next($request);
    }
}
