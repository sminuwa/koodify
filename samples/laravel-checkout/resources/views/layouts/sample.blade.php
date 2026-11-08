<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="csrf-token" content="{{ csrf_token() }}">
    <title>@yield('title', 'Koodify merchant')</title>
    <link rel="icon" href="{{ asset('brand/favicon/favicon.ico') }}" sizes="any">
    <link rel="icon" type="image/png" sizes="32x32" href="{{ asset('brand/favicon/favicon-32x32.png') }}">
    <link rel="apple-touch-icon" href="{{ asset('brand/favicon/apple-touch-icon.png') }}">
    <link rel="manifest" href="{{ asset('brand/favicon/site.webmanifest') }}">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500&family=IBM+Plex+Sans:wght@400;500;600;700&family=Outfit:wght@500;600;700&display=swap" rel="stylesheet">
    <style>
        :root {
            --navy: #070b1c;
            --ink: #0f172a;
            --muted: #64748b;
            --line: #e2e8f0;
            --card: #ffffff;
            --bg: #f1f5f9;
            --accent: #1d4ed8;
            --accent-ink: #ffffff;
            --cyan: #38c6e8;
            --ok: #047857;
            --warn: #9a6700;
            --danger: #b91c1c;
            --font-display: "Outfit", system-ui, sans-serif;
            --font-body: "IBM Plex Sans", system-ui, sans-serif;
            --font-mono: "IBM Plex Mono", ui-monospace, monospace;
            --ease: cubic-bezier(0.23, 1, 0.32, 1);
            --header-h: 4.25rem;
        }
        * { box-sizing: border-box; }
        body {
            margin: 0;
            min-height: 100vh;
            font-family: var(--font-body);
            color: var(--ink);
            background:
                radial-gradient(900px 420px at 0% -10%, rgba(56, 198, 232, 0.14), transparent 50%),
                radial-gradient(800px 380px at 100% 0%, rgba(29, 78, 216, 0.10), transparent 45%),
                var(--bg);
            -webkit-font-smoothing: antialiased;
        }
        .site-header {
            position: sticky;
            top: 0;
            z-index: 20;
            backdrop-filter: blur(14px);
            background: rgba(255, 255, 255, 0.86);
            border-bottom: 1px solid color-mix(in srgb, var(--line) 80%, transparent);
        }
        .site-header-inner {
            width: min(1080px, calc(100% - 2rem));
            margin: 0 auto;
            min-height: var(--header-h);
            display: flex;
            flex-wrap: wrap;
            gap: .75rem 1rem;
            align-items: center;
            justify-content: space-between;
            padding: .65rem 0;
        }
        .brand {
            display: inline-flex;
            align-items: center;
            gap: .75rem;
            text-decoration: none;
            color: var(--ink);
        }
        .brand img { height: 28px; width: auto; display: block; }
        .brand .who {
            font-size: .82rem;
            color: var(--muted);
            border-left: 1px solid var(--line);
            padding-left: .75rem;
            max-width: 16rem;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }
        .nav {
            display: flex;
            gap: .2rem;
            flex-wrap: wrap;
            align-items: center;
        }
        .nav a, .nav button.linkish {
            text-decoration: none;
            color: var(--muted);
            font-size: .84rem;
            font-weight: 600;
            padding: .45rem .75rem;
            border-radius: 999px;
            border: 1px solid transparent;
            background: transparent;
            cursor: pointer;
            font: inherit;
            font-weight: 600;
            transition: background .2s var(--ease), color .2s var(--ease), border-color .2s var(--ease);
        }
        .nav a:hover, .nav button.linkish:hover {
            color: var(--ink);
            background: #fff;
            border-color: var(--line);
        }
        .nav a.on {
            color: var(--accent);
            background: #fff;
            border-color: color-mix(in srgb, var(--accent) 28%, white);
        }
        .wrap {
            width: min(920px, calc(100% - 2rem));
            margin: 0 auto;
            padding: 1.75rem 0 4rem;
        }
        .card {
            background: var(--card);
            border: 1px solid var(--line);
            border-radius: 20px;
            padding: 1.65rem 1.75rem;
            box-shadow: 0 18px 50px rgba(15, 23, 42, .06);
            animation: rise .45s var(--ease) both;
        }
        @keyframes rise {
            from { opacity: 0; transform: translateY(8px); }
            to { opacity: 1; transform: none; }
        }
        h1 {
            margin: 0 0 .45rem;
            font-family: var(--font-display);
            font-size: clamp(1.65rem, 3vw, 2.05rem);
            letter-spacing: -0.03em;
            line-height: 1.15;
        }
        .lede {
            margin: 0 0 1.5rem;
            color: var(--muted);
            line-height: 1.55;
            max-width: 52ch;
        }
        label {
            display: block;
            margin: 0 0 .35rem;
            font-size: .82rem;
            font-weight: 600;
            color: #334155;
        }
        .field { margin-bottom: 1rem; }
        input, select, textarea {
            width: 100%;
            border: 1px solid var(--line);
            border-radius: 12px;
            padding: .8rem .95rem;
            font: inherit;
            background: #fff;
            transition: border-color .2s var(--ease), box-shadow .2s var(--ease);
        }
        input:focus, select:focus, textarea:focus {
            outline: none;
            border-color: var(--accent);
            box-shadow: 0 0 0 4px rgba(29, 78, 216, 0.14);
        }
        .grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 1rem;
        }
        @media (max-width: 640px) {
            .grid { grid-template-columns: 1fr; }
            .brand .who { display: none; }
        }
        .actions {
            display: flex;
            flex-wrap: wrap;
            gap: .75rem;
            align-items: center;
            margin-top: 1.25rem;
        }
        button, .btn {
            appearance: none;
            border: 0;
            border-radius: 999px;
            background: linear-gradient(135deg, #1d4ed8 0%, #2563eb 100%);
            color: var(--accent-ink);
            font: inherit;
            font-weight: 600;
            padding: .8rem 1.25rem;
            cursor: pointer;
            text-decoration: none;
            display: inline-flex;
            align-items: center;
            box-shadow: 0 12px 28px rgba(29, 78, 216, 0.22);
            transition: transform .2s var(--ease);
        }
        button:hover, .btn:hover { transform: translateY(-1px); }
        button.secondary, .btn.secondary {
            background: #fff;
            color: var(--ink);
            border: 1px solid var(--line);
            box-shadow: none;
        }
        button:disabled { opacity: .55; cursor: not-allowed; transform: none; }
        .hint { color: var(--muted); font-size: .88rem; }
        .banner {
            margin-bottom: 1rem;
            padding: .85rem 1rem;
            border-radius: 12px;
            border: 1px solid #f0d48a;
            background: #fff8e6;
            color: var(--warn);
            font-size: .92rem;
        }
        .banner.ok {
            border-color: #a7f3d0;
            background: #ecfdf5;
            color: var(--ok);
        }
        .banner.err {
            border-color: #fecaca;
            background: #fef2f2;
            color: var(--danger);
        }
        .status {
            margin-top: 1.25rem;
            padding: 1rem;
            border-radius: 12px;
            border: 1px solid var(--line);
            background: #f8fafc;
            font-family: var(--font-mono);
            font-size: .85rem;
            white-space: pre-wrap;
            word-break: break-word;
            min-height: 4.5rem;
            color: var(--muted);
        }
        .status.ok { border-color: #a7f3d0; background: #ecfdf5; color: var(--ok); }
        .status.err { border-color: #fecaca; background: #fef2f2; color: var(--danger); }
        .meta {
            margin-top: 1.25rem;
            color: var(--muted);
            font-size: .82rem;
            line-height: 1.5;
        }
        code, .mono {
            font-family: var(--font-mono);
            font-size: .84em;
            background: #eef2ff;
            padding: .1rem .35rem;
            border-radius: 5px;
        }
        .pill {
            display: inline-flex;
            align-items: center;
            padding: .25rem .6rem;
            border-radius: 999px;
            font-size: .72rem;
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: .05em;
            background: #eef2ff;
            color: var(--muted);
        }
        .pill.ok { background: #d1fae5; color: var(--ok); }
        .pill.warn { background: #fef3c7; color: var(--warn); }
        .stack { display: grid; gap: 1rem; }
        .keyrow {
            display: flex;
            flex-wrap: wrap;
            gap: .5rem;
            align-items: center;
            justify-content: space-between;
            padding: 1rem 1.1rem;
            border: 1px solid var(--line);
            border-radius: 14px;
            background: linear-gradient(180deg, #fff 0%, #f8fafc 100%);
        }
        .keyrow code { word-break: break-all; background: transparent; padding: 0; }
        .keyrow .hint { margin-bottom: .2rem; }
    </style>
    @stack('head')
</head>
<body>
    <header class="site-header">
        <div class="site-header-inner">
            <a class="brand" href="{{ session()->has('merchant.token') ? route('dashboard') : route('login') }}">
                <img src="{{ asset('brand/koodify-logo.png') }}" alt="Koodify" width="140" height="34">
                @if(session('merchant.display_name'))
                    <span class="who">{{ session('merchant.display_name') }}</span>
                @endif
            </a>
            <nav class="nav" aria-label="Merchant">
                @if(session()->has('merchant.token'))
                    <a href="{{ route('dashboard') }}" @class(['on' => request()->routeIs('dashboard')])>Dashboard</a>
                    <a href="{{ route('compliance.show') }}" @class(['on' => request()->routeIs('compliance.*')])>KYC</a>
                    <a href="{{ route('credentials.show') }}" @class(['on' => request()->routeIs('credentials.*')])>Credentials</a>
                    <a href="{{ route('checkout') }}" @class(['on' => request()->routeIs('checkout')])>Checkout</a>
                    <a href="{{ route('virtual-accounts.index') }}" @class(['on' => request()->routeIs('virtual-accounts.index')])>Virtual Accounts</a>
                    <a href="{{ route('payment-simulation') }}" @class(['on' => request()->routeIs('payment-simulation')])>Simulation</a>
                    <form method="post" action="{{ route('logout') }}" style="display:inline;margin:0">
                        @csrf
                        <button class="linkish" type="submit">Log out</button>
                    </form>
                @else
                    <a href="{{ route('login') }}" @class(['on' => request()->routeIs('login')])>Log in</a>
                    <a href="{{ route('register') }}" @class(['on' => request()->routeIs('register')])>Register</a>
                @endif
            </nav>
        </div>
    </header>

    <div class="wrap">
        @if (session('status'))
            <div class="banner ok">{{ session('status') }}</div>
        @endif
        @if (session('error'))
            <div class="banner err">{{ session('error') }}</div>
        @endif
        @if ($errors->any())
            <div class="banner err">{{ $errors->first() }}</div>
        @endif

        @yield('content')
    </div>
    @stack('scripts')
</body>
</html>
