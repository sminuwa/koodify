<?php

use App\Http\Controllers\AccountBalanceController;
use App\Http\Controllers\CheckoutController;
use App\Http\Controllers\DynamicVirtualAccountController;
use App\Http\Controllers\KoodifyWebhookController;
use App\Http\Controllers\VirtualAccountController;
use Illuminate\Support\Facades\Route;

Route::get('/', CheckoutController::class)->name('checkout');

Route::get('/virtual-accounts/create', [VirtualAccountController::class, 'createForm'])
    ->name('virtual-accounts.create');
Route::post('/virtual-accounts', [VirtualAccountController::class, 'store'])
    ->name('virtual-accounts.store');

Route::get('/dynamic-virtual-accounts/create', [DynamicVirtualAccountController::class, 'createForm'])
    ->name('dynamic-virtual-accounts.create');
Route::post('/dynamic-virtual-accounts', [DynamicVirtualAccountController::class, 'store'])
    ->name('dynamic-virtual-accounts.store');

Route::get('/account-balance', [AccountBalanceController::class, 'showForm'])
    ->name('account-balance.show');
Route::post('/account-balance', [AccountBalanceController::class, 'lookup'])
    ->name('account-balance.lookup');

Route::post('/webhooks/koodify', KoodifyWebhookController::class)
    ->name('webhooks.koodify');
