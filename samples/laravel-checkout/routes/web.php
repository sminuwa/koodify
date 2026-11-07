<?php

use App\Http\Controllers\CheckoutController;
use App\Http\Controllers\KoodifyWebhookController;
use App\Http\Controllers\VirtualAccountController;
use Illuminate\Support\Facades\Route;

Route::get('/', CheckoutController::class)->name('checkout');

Route::get('/virtual-accounts/create', [VirtualAccountController::class, 'createForm'])
    ->name('virtual-accounts.create');
Route::post('/virtual-accounts', [VirtualAccountController::class, 'store'])
    ->name('virtual-accounts.store');

Route::post('/webhooks/koodify', KoodifyWebhookController::class)
    ->name('webhooks.koodify');
