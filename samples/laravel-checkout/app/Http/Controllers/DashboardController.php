<?php

namespace App\Http\Controllers;

use App\Services\KoodifyClient;
use App\Support\DashboardInsights;
use App\Support\MerchantSession;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\View\View;
use RuntimeException;
use Throwable;

class DashboardController extends Controller
{
    public function __invoke(Request $request): View|RedirectResponse
    {
        $result = KoodifyClient::withMerchantToken((string) session('merchant.token'))->me();

        if (($result['code'] ?? '') !== '00' || ! is_array($result['data'] ?? null)) {
            MerchantSession::clear();

            return redirect()
                ->route('login')
                ->with('error', $result['description'] ?? 'Session expired. Log in again.');
        }

        $profile = $result['data'];
        session([
            'merchant.profile' => $profile,
            'merchant.display_name' => $profile['displayName'] ?? session('merchant.display_name'),
            'merchant.public_key' => $profile['publicKey'] ?? session('merchant.public_key'),
        ]);

        $range = $request->query('range', 'daily');
        if (! in_array($range, ['daily', 'weekly', 'monthly'], true)) {
            $range = 'daily';
        }

        $publicKey = (string) ($profile['publicKey'] ?? session('merchant.public_key', ''));
        $secretKey = (string) session('merchant.secret_key', '');
        // Prefer the issued session secret; fall back to env demo key for local samples.
        if ($secretKey === '') {
            $secretKey = (string) config('koodify.secret_key', '');
        }
        $hasPublic = $publicKey !== '';
        $hasSecret = $secretKey !== '';
        $configured = $hasPublic && $hasSecret;

        $zainboxCode = (string) (
            data_get($profile, 'zainbox.zainboxCode')
            ?? data_get($profile, 'zainboxCode')
            ?? ''
        );

        $insights = DashboardInsights::empty($range);
        $apiError = null;

        if ($configured) {
            try {
                $client = new KoodifyClient(
                    rtrim((string) config('koodify.api_base'), '/'),
                    $secretKey,
                );
                $loaded = $this->loadInsights($client, $range, $zainboxCode !== '' ? $zainboxCode : null);
                $insights = $loaded['insights'];
                $apiError = $loaded['error'];
            } catch (RuntimeException $e) {
                $apiError = $e->getMessage();
            } catch (Throwable $e) {
                report($e);
                $apiError = 'Dashboard request failed: '.$e->getMessage();
            }
        }

        $emptyReason = null;
        if (! $configured) {
            if (! $hasPublic && ! $hasSecret) {
                $emptyReason = 'complete_compliance';
            } elseif (! $hasPublic) {
                $emptyReason = 'awaiting_provision';
            } else {
                $emptyReason = 'issue_secret';
            }
        }

        return view('merchant.overview', [
            'profile' => $profile,
            'kyc' => $profile['kyc'] ?? null,
            'credentials' => $profile['credentials'] ?? [],
            'hasSecret' => $hasSecret,
            'hasPublic' => $hasPublic,
            'configured' => $configured,
            'emptyReason' => $emptyReason,
            'range' => $range,
            'insights' => $insights,
            'apiError' => $apiError,
            'zainboxCode' => $zainboxCode !== '' ? $zainboxCode : 'your merchant account',
        ]);
    }

    /**
     * Prefer auto-scoped wallet/transactions; fall back to VA list APIs that already
     * accept merchant sk_… on older gateways (still admin-only on /api/v1/transactions).
     *
     * @return array{insights: array<string, mixed>, error: ?string}
     */
    private function loadInsights(KoodifyClient $client, string $range, ?string $zainboxCode): array
    {
        $window = DashboardInsights::rangeWindow($range);

        $activity = $client->merchantTransactions([
            'dateFrom' => $window['from'],
            'dateTo' => $window['to'],
            'count' => $window['count'],
        ]);
        $wallet = $client->merchantWallet($zainboxCode);

        $activityOk = ($activity['code'] ?? '') === '00' && is_array($activity['data'] ?? null);
        $walletOk = ($wallet['code'] ?? '') === '00' && is_array($wallet['data'] ?? null);

        if ($activityOk || $walletOk) {
            return [
                'insights' => DashboardInsights::fromApi(
                    $activityOk ? $activity['data'] : [],
                    $walletOk ? $wallet['data'] : [],
                    $range,
                ),
                'error' => $this->partialError($activityOk, $walletOk, $activity, $wallet),
            ];
        }

        // Older production gateways reject merchant sk_… on /transactions and /wallet.
        if ($this->isAdminOnly($activity) || $this->isAdminOnly($wallet)) {
            return $this->loadInsightsViaVirtualAccounts($client, $range, $window);
        }

        return [
            'insights' => DashboardInsights::empty($range),
            'error' => (string) (
                $activity['description']
                ?? $wallet['description']
                ?? 'Could not load dashboard activity.'
            ),
        ];
    }

    /**
     * @param  array{from: string, to: string, count: int}  $window
     * @return array{insights: array<string, mixed>, error: ?string}
     */
    private function loadInsightsViaVirtualAccounts(
        KoodifyClient $client,
        string $range,
        array $window,
    ): array {
        $listed = $client->listVirtualAccounts();
        if (($listed['code'] ?? '') !== '00') {
            return [
                'insights' => DashboardInsights::empty($range),
                'error' => (string) ($listed['description'] ?? 'Could not load virtual accounts for dashboard.'),
            ];
        }

        $accounts = [];
        $raw = $listed['data'] ?? null;
        if (is_array($raw)) {
            $accounts = array_is_list($raw) ? $raw : (is_array($raw['data'] ?? null) ? $raw['data'] : []);
        }

        $balances = [];
        $transactions = [];
        $errors = [];

        foreach (array_slice($accounts, 0, 8) as $account) {
            if (! is_array($account)) {
                continue;
            }
            $number = trim((string) ($account['accountNumber'] ?? $account['account_number'] ?? ''));
            if ($number === '') {
                continue;
            }

            $bal = $client->getVirtualAccountBalance($number);
            if (($bal['code'] ?? '') === '00' && is_array($bal['data'] ?? null)) {
                $row = $bal['data'];
                if (! isset($row['accountNumber'])) {
                    $row['accountNumber'] = $number;
                }
                $balances[] = $row;
            } elseif (! empty($bal['description'])) {
                $errors[] = (string) $bal['description'];
            }

            $tx = $client->getVirtualAccountTransactions($number, [
                'dateFrom' => $window['from'],
                'dateTo' => $window['to'],
                'count' => min(50, (int) $window['count']),
            ]);
            if (($tx['code'] ?? '') === '00') {
                $rows = $tx['data'] ?? [];
                if (is_array($rows)) {
                    if (! array_is_list($rows) && is_array($rows['data'] ?? null)) {
                        $rows = $rows['data'];
                    }
                    foreach ($rows as $item) {
                        if (is_array($item)) {
                            $transactions[] = $item;
                        }
                    }
                }
            } elseif (! empty($tx['description'])) {
                $errors[] = (string) $tx['description'];
            }
        }

        return [
            'insights' => DashboardInsights::fromMerchantAccounts(
                array_values(array_filter($accounts, 'is_array')),
                $balances,
                $transactions,
                $range,
            ),
            'error' => $accounts === [] && $errors !== []
                ? $errors[0]
                : null,
        ];
    }

    /**
     * @param  array<string, mixed>  $activity
     * @param  array<string, mixed>  $wallet
     */
    private function partialError(bool $activityOk, bool $walletOk, array $activity, array $wallet): ?string
    {
        if ($activityOk && $walletOk) {
            return null;
        }
        if (! $activityOk && ! $walletOk) {
            return (string) ($activity['description'] ?? $wallet['description'] ?? 'Could not load dashboard activity.');
        }
        if (! $activityOk) {
            return (string) ($activity['description'] ?? 'Could not load transactions.');
        }

        return (string) ($wallet['description'] ?? 'Could not load wallet balances.');
    }

    /**
     * @param  array<string, mixed>  $result
     */
    private function isAdminOnly(array $result): bool
    {
        $hay = strtolower((string) ($result['description'] ?? $result['message'] ?? ''));

        return str_contains($hay, 'platform admin')
            || str_contains($hay, 'admin api key')
            || (($result['code'] ?? '') === '403');
    }
}
