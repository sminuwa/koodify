<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="csrf-token" content="{{ csrf_token() }}">
    <title>@yield('title', 'Koodify sample')</title>
    <style>
        :root {
            --bg: #f4f7fb;
            --ink: #0f1b2d;
            --muted: #5b6b7c;
            --card: #ffffff;
            --line: #d7e0ea;
            --accent: #0b6bcb;
            --accent-ink: #ffffff;
            --ok: #0f7a43;
            --warn: #9a6700;
        }
        * { box-sizing: border-box; }
        body {
            margin: 0;
            min-height: 100vh;
            font-family: "Segoe UI", system-ui, -apple-system, sans-serif;
            color: var(--ink);
            background:
                radial-gradient(1200px 500px at 10% -10%, #d9ebff 0%, transparent 55%),
                radial-gradient(900px 400px at 100% 0%, #e8f6ef 0%, transparent 45%),
                var(--bg);
        }
        .wrap {
            width: min(760px, calc(100% - 2rem));
            margin: 0 auto;
            padding: 2rem 0 4rem;
        }
        .top {
            display: flex;
            flex-wrap: wrap;
            gap: .75rem 1.25rem;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 1.25rem;
        }
        .brand {
            display: flex;
            align-items: baseline;
            gap: .6rem;
        }
        .brand strong { letter-spacing: .02em; }
        .brand span { color: var(--muted); font-size: .92rem; }
        .nav {
            display: flex;
            gap: .35rem;
            flex-wrap: wrap;
        }
        .nav a {
            text-decoration: none;
            color: var(--muted);
            font-size: .9rem;
            font-weight: 600;
            padding: .45rem .8rem;
            border-radius: 999px;
            border: 1px solid transparent;
        }
        .nav a:hover { color: var(--ink); background: #fff; border-color: var(--line); }
        .nav a.on {
            color: var(--accent);
            background: #fff;
            border-color: color-mix(in srgb, var(--accent) 30%, white);
        }
        .card {
            background: var(--card);
            border: 1px solid var(--line);
            border-radius: 16px;
            padding: 1.5rem;
            box-shadow: 0 18px 40px rgba(15, 27, 45, .06);
        }
        h1 {
            margin: 0 0 .5rem;
            font-size: clamp(1.55rem, 3vw, 1.95rem);
            line-height: 1.15;
        }
        .lede {
            margin: 0 0 1.5rem;
            color: var(--muted);
            line-height: 1.55;
        }
        label {
            display: block;
            margin: 0 0 .35rem;
            font-size: .85rem;
            font-weight: 600;
        }
        .field { margin-bottom: 1rem; }
        input, select {
            width: 100%;
            border: 1px solid var(--line);
            border-radius: 10px;
            padding: .75rem .9rem;
            font: inherit;
            background: #fff;
        }
        input:focus, select:focus {
            outline: 2px solid color-mix(in srgb, var(--accent) 35%, white);
            border-color: var(--accent);
        }
        .grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 1rem;
        }
        @media (max-width: 640px) {
            .grid { grid-template-columns: 1fr; }
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
            background: var(--accent);
            color: var(--accent-ink);
            font: inherit;
            font-weight: 600;
            padding: .8rem 1.25rem;
            cursor: pointer;
            text-decoration: none;
            display: inline-flex;
            align-items: center;
        }
        button:disabled {
            opacity: .55;
            cursor: not-allowed;
        }
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
        .banner.err {
            border-color: #f0c2c2;
            background: #fff5f5;
            color: #9b1c1c;
        }
        .status {
            margin-top: 1.25rem;
            padding: 1rem;
            border-radius: 12px;
            border: 1px solid var(--line);
            background: #f8fafc;
            font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
            font-size: .85rem;
            white-space: pre-wrap;
            word-break: break-word;
            min-height: 4.5rem;
            color: var(--muted);
        }
        .status.ok { border-color: #b7e4c7; background: #f0faf4; color: var(--ok); }
        .status.err { border-color: #f0c2c2; background: #fff5f5; color: #9b1c1c; }
        .meta {
            margin-top: 1.25rem;
            color: var(--muted);
            font-size: .82rem;
            line-height: 1.5;
        }
        code {
            font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
            font-size: .84em;
            background: #eef3f8;
            padding: .1rem .3rem;
            border-radius: 4px;
        }
    </style>
    @stack('head')
</head>
<body>
    <div class="wrap">
        <div class="top">
            <div class="brand">
                <strong>Koodify</strong>
                <span>Laravel sample</span>
            </div>
            <nav class="nav" aria-label="Sample pages">
                <a href="{{ route('checkout') }}" @class(['on' => request()->routeIs('checkout')])>Inline checkout</a>
                <a href="{{ route('virtual-accounts.create') }}" @class(['on' => request()->routeIs('virtual-accounts.*')])>Static VA</a>
                <a href="{{ route('dynamic-virtual-accounts.create') }}" @class(['on' => request()->routeIs('dynamic-virtual-accounts.*')])>Dynamic VA</a>
            </nav>
        </div>

        @yield('content')
    </div>
    @stack('scripts')
</body>
</html>
