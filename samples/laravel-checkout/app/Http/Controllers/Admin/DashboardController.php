<?php

namespace App\Http\Controllers\Admin;

use App\Http\Controllers\Controller;
use App\Services\KoodifyAdminClient;
use Illuminate\View\View;
use Throwable;

class DashboardController extends Controller
{
    public function __invoke(): View
    {
        $stats = [
            'merchants' => 0,
            'pendingKyc' => 0,
            'approvedKyc' => 0,
            'pendingComplianceSections' => 0,
            'awaitingProvision' => 0,
            'pendingQueue' => [],
        ];
        $apiError = null;

        try {
            $result = KoodifyAdminClient::fromSession()->dashboard();
            if (($result['code'] ?? '') === '00' && is_array($result['data'] ?? null)) {
                $stats = array_merge($stats, $result['data']);
            } else {
                $apiError = (string) ($result['description'] ?? 'Dashboard unavailable.');
            }
        } catch (Throwable $e) {
            $apiError = $e->getMessage();
        }

        return view('admin.dashboard', [
            'adminName' => session('admin.name', 'Admin'),
            'merchantCount' => (int) ($stats['merchants'] ?? 0),
            'pendingKyc' => (int) ($stats['pendingKyc'] ?? 0),
            'approvedKyc' => (int) ($stats['approvedKyc'] ?? 0),
            'pendingSections' => (int) ($stats['pendingComplianceSections'] ?? 0),
            'awaitingProvision' => (int) ($stats['awaitingProvision'] ?? 0),
            'pendingQueue' => is_array($stats['pendingQueue'] ?? null) ? $stats['pendingQueue'] : [],
            'apiError' => $apiError,
        ]);
    }
}
