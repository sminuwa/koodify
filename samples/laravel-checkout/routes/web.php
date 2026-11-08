<?php

use App\Http\Controllers\AccountBalanceController;
use App\Http\Controllers\Admin\AuthController as AdminAuthController;
use App\Http\Controllers\Admin\DashboardController as AdminDashboardController;
use App\Http\Controllers\Admin\MerchantController as AdminMerchantController;
use App\Http\Controllers\Admin\SectionController as AdminSectionController;
use App\Http\Controllers\AuthController;
use App\Http\Controllers\CheckoutController;
use App\Http\Controllers\CredentialsController;
use App\Http\Controllers\DashboardController;
use App\Http\Controllers\DynamicVirtualAccountController;
use App\Http\Controllers\GetStartedController;
use App\Http\Controllers\GuidesController;
use App\Http\Controllers\KoodifyWebhookController;
use App\Http\Controllers\KycController;
use App\Http\Controllers\MerchantPageController;
use App\Http\Controllers\PaymentSimulationController;
use App\Http\Controllers\ProfileController;
use App\Http\Controllers\VirtualAccountController;
use App\Http\Middleware\EnsureAdminSession;
use App\Http\Middleware\EnsureMerchantSession;
use Illuminate\Support\Facades\Route;

Route::get('/register', [AuthController::class, 'showRegister'])->name('register');
Route::post('/register', [AuthController::class, 'register']);
Route::get('/login', [AuthController::class, 'showLogin'])->name('login');
Route::post('/login', [AuthController::class, 'login']);
Route::post('/logout', [AuthController::class, 'logout'])->name('logout');

Route::prefix('admin')->name('admin.')->group(function () {
    Route::get('/login', [AdminAuthController::class, 'showLogin'])->name('login');
    Route::post('/login', [AdminAuthController::class, 'login'])->name('login.submit');
    Route::post('/logout', [AdminAuthController::class, 'logout'])->name('logout');

    Route::middleware(EnsureAdminSession::class)->group(function () {
        Route::get('/', AdminDashboardController::class)->name('dashboard');
        Route::get('/merchants', [AdminMerchantController::class, 'index'])->name('merchants.index');
        Route::get('/merchants/{id}', [AdminMerchantController::class, 'show'])->name('merchants.show');
        Route::post('/merchants/{id}/kyc', [AdminMerchantController::class, 'approveKyc'])->name('merchants.kyc');
        Route::post('/merchants/{id}/provision', [AdminMerchantController::class, 'provision'])->name('merchants.provision');
        Route::get('/{section}', AdminSectionController::class)
            ->whereIn('section', ['transactions', 'customers', 'services', 'reports', 'settings', 'activity'])
            ->name('section');
    });
});

Route::post('/webhooks/koodify', KoodifyWebhookController::class)
    ->name('webhooks.koodify');

Route::middleware(EnsureMerchantSession::class)->group(function () {
    Route::get('/', GetStartedController::class)->name('get-started');
    Route::get('/dashboard', DashboardController::class)->name('dashboard');
    Route::get('/profile', ProfileController::class)->name('profile');

    Route::get('/transactions', [MerchantPageController::class, 'show'])->defaults('page', 'transactions')->name('transactions');
    Route::get('/transfers', [MerchantPageController::class, 'show'])->defaults('page', 'transfers')->name('transfers');
    Route::get('/guides', GuidesController::class)->name('guides');
    Route::get('/payment-simulation', PaymentSimulationController::class)->name('payment-simulation');
    Route::get('/accept-payments', [MerchantPageController::class, 'show'])->defaults('page', 'accept-payments')->name('accept-payments');
    Route::get('/webhooks', [MerchantPageController::class, 'show'])->defaults('page', 'webhooks')->name('webhooks');

    Route::get('/compliance', [KycController::class, 'showForm'])->name('compliance.show');
    Route::get('/compliance/form', [KycController::class, 'showVerify'])->name('compliance.form');
    Route::post('/compliance', [KycController::class, 'submitSection'])->name('compliance.submit');
    Route::post('/compliance/section', [KycController::class, 'submitSection'])->name('compliance.section');
    Route::redirect('/kyc', '/compliance');
    Route::redirect('/kyc/form', '/compliance/form');

    Route::get('/credentials', [CredentialsController::class, 'show'])->name('credentials.show');
    Route::post('/credentials/issue', [CredentialsController::class, 'issue'])->name('credentials.issue');

    Route::get('/checkout', CheckoutController::class)->name('checkout');

    Route::get('/va', [VirtualAccountController::class, 'index'])->name('virtual-accounts.index');
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
});
