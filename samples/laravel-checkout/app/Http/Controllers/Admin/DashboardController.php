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
        $merchantCount = null;
        $pendingKyc = 0;
        $customers = [];
        $apiError = null;

        try {
            $client = KoodifyAdminClient::fromConfig();
            $result = $client->listCustomers();
            // Admin list returns a JSON array (not always enveloped).
            $rows = $result['data'] ?? $result;
            if (is_array($rows) && array_is_list($rows)) {
                $customers = $rows;
                $merchantCount = count($rows);
                foreach ($rows as $row) {
                    if (($row['status'] ?? '') === 'pending') {
                        $pendingKyc++;
                    }
                }
            } elseif (isset($result['description'])) {
                $apiError = (string) $result['description'];
            }
        } catch (Throwable $e) {
            $apiError = $e->getMessage();
        }

        return view('admin.dashboard', [
            'adminName' => session('admin.name', 'Admin'),
            'merchantCount' => $merchantCount,
            'pendingKyc' => $pendingKyc,
            'recentCustomers' => array_slice(array_reverse($customers), 0, 6),
            'apiError' => $apiError,
        ]);
    }
}
