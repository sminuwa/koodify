<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use Illuminate\Support\Str;
use Illuminate\View\View;

class PaymentSimulationController extends Controller
{
    public function __invoke(Request $request): View
    {
        $tool = (string) $request->query('tool', 'va');
        if (! in_array($tool, ['va', 'dva', 'balance'], true)) {
            $tool = 'va';
        }

        $configured = filled(session('merchant.secret_key')) || filled(config('koodify.secret_key'));

        return view('merchant.payment-simulation', [
            'tool' => $tool,
            'configured' => $configured,
            'vaDefaults' => [
                'title' => 'Ms',
                'firstName' => 'Ada',
                'surname' => 'Lovelace',
                'email' => 'ada@example.com',
                'mobileNumber' => '08012345678',
                'address' => '12 Computing Lane, Lagos',
                'state' => 'Lagos',
            ],
            'dvaDefaults' => [
                'email' => 'ada@example.com',
                'amount' => (string) config('koodify.demo_amount', '50000'),
                'txnRef' => 'DVA_'.Str::upper(Str::random(10)),
                'duration' => '900',
                'callBackUrl' => (string) (config('koodify.callback_url') ?: url('/webhooks/koodify')),
            ],
            'vaResult' => session('va_result'),
            'vaError' => session('va_error'),
            'dvaResult' => session('dva_result'),
            'dvaError' => session('dva_error'),
            'balanceResult' => session('balance_result'),
            'balanceError' => session('balance_error'),
            'accountNumber' => old(
                'accountNumber',
                session('balance_account', $request->query('accountNumber'))
            ),
        ]);
    }
}
