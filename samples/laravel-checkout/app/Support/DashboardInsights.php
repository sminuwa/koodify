<?php

namespace App\Support;

use Carbon\Carbon;
use Carbon\CarbonPeriod;

final class DashboardInsights
{
    /**
     * @param  array<string, mixed>  $activity   Composite zainbox transactions payload
     * @param  array<string, mixed>  $wallet     Composite zainbox wallet payload
     * @return array{
     *   totalCollected: float,
     *   totalPayouts: float,
     *   transactionCount: int,
     *   availableBalance: float,
     *   points: list<array{label: string, in: float, out: float}>,
     *   boxes: list<array{name: string, codeName: string, collected: float}>,
     *   transactions: list<array<string, mixed>>
     * }
     */
    /**
     * Build insights from merchant-scoped VA list + balances + per-account transactions.
     * Works on gateways that still require admin for `/api/v1/transactions`.
     *
     * @param  list<array<string, mixed>>  $accounts
     * @param  list<array<string, mixed>>  $balanceRows
     * @param  list<array<string, mixed>>  $txRows
     * @return array{
     *   totalCollected: float,
     *   totalPayouts: float,
     *   transactionCount: int,
     *   availableBalance: float,
     *   points: list<array{label: string, in: float, out: float}>,
     *   boxes: list<array{name: string, codeName: string, collected: float}>,
     *   transactions: list<array<string, mixed>>
     * }
     */
    public static function fromMerchantAccounts(
        array $accounts,
        array $balanceRows,
        array $txRows,
        string $range,
    ): array {
        return self::fromApi(
            [
                'transactions' => ['data' => $txRows],
                'depositSummary' => ['data' => []],
            ],
            [
                'virtualAccounts' => ['data' => $accounts],
                'balances' => ['data' => $balanceRows],
            ],
            $range,
        );
    }

    public static function fromApi(array $activity, array $wallet, string $range): array
    {
        $txRows = self::unwrapList(
            data_get($activity, 'transactions.data', data_get($activity, 'transactions', []))
        );
        $balances = self::unwrapList(
            data_get($wallet, 'balances.data', data_get($wallet, 'balances', []))
        );
        $accounts = self::unwrapList(
            data_get($wallet, 'virtualAccounts.data', data_get($wallet, 'virtualAccounts', []))
        );

        $depositTotal = self::money(data_get($activity, 'depositSummary.data.totalDeposits'));
        $available = 0.0;
        foreach ($balances as $row) {
            $available += self::money($row['balanceAmount'] ?? $row['balance'] ?? $row['availableBalance'] ?? 0);
        }

        $collected = 0.0;
        $payouts = 0.0;
        $channelTotals = [];
        $normalized = [];

        foreach ($txRows as $row) {
            if (! is_array($row)) {
                continue;
            }
            $amount = abs(self::money($row['amount'] ?? $row['txnAmount'] ?? 0));
            $type = strtolower((string) ($row['txnType'] ?? $row['type'] ?? $row['transactionType'] ?? 'credit'));
            $out = self::isOutflow($type, $row);
            if ($out) {
                $payouts += $amount;
            } else {
                $collected += $amount;
            }

            $channel = (string) ($row['paymentChannel'] ?? $row['channel'] ?? $row['source'] ?? 'Other');
            if ($channel === '') {
                $channel = 'Other';
            }
            if (! $out) {
                $channelTotals[$channel] = ($channelTotals[$channel] ?? 0) + $amount;
            }

            $when = self::parseDate(
                $row['transactionDate'] ?? $row['createdAt'] ?? $row['dateCreated'] ?? $row['date'] ?? null
            );

            $normalized[] = [
                'reference' => (string) ($row['txnRef'] ?? $row['reference'] ?? $row['id'] ?? '—'),
                'zainbox' => (string) ($row['zainboxCode'] ?? $row['zainbox'] ?? $channel),
                'sender' => (string) ($row['accountName'] ?? $row['sender'] ?? $row['customerName'] ?? $row['narration'] ?? '—'),
                'type' => (string) ($row['paymentChannel'] ?? $row['txnType'] ?? $type),
                'amount' => $amount,
                'out' => $out,
                'date' => $when,
                'status' => self::statusLabel($row['status'] ?? $row['txnStatus'] ?? 'Successful'),
                'tone' => self::statusTone($row['status'] ?? $row['txnStatus'] ?? 'Successful'),
            ];
        }

        if ($depositTotal > 0) {
            $collected = max($collected, $depositTotal);
        }

        usort($normalized, static function (array $a, array $b): int {
            return ($b['date']?->timestamp ?? 0) <=> ($a['date']?->timestamp ?? 0);
        });

        arsort($channelTotals);
        $boxes = [];
        foreach (array_slice($channelTotals, 0, 6, true) as $name => $total) {
            $boxes[] = [
                'name' => $name,
                'codeName' => strtolower(preg_replace('/\s+/', '-', $name) ?: 'channel'),
                'collected' => (float) $total,
            ];
        }
        if ($boxes === [] && $accounts !== []) {
            foreach (array_slice($accounts, 0, 4) as $account) {
                if (! is_array($account)) {
                    continue;
                }
                $boxes[] = [
                    'name' => (string) ($account['accountName'] ?? $account['accountNumber'] ?? 'Virtual account'),
                    'codeName' => (string) ($account['accountNumber'] ?? 'va'),
                    'collected' => 0.0,
                ];
            }
        }
        if ($boxes === []) {
            $boxes = [
                ['name' => 'Collections', 'codeName' => 'collections', 'collected' => $collected],
            ];
        }

        return [
            'totalCollected' => $collected,
            'totalPayouts' => $payouts,
            'transactionCount' => count($normalized),
            'availableBalance' => $available,
            'points' => self::buildPoints($normalized, $range),
            'boxes' => $boxes,
            'transactions' => array_slice($normalized, 0, 8),
        ];
    }

    /**
     * @return array{
     *   totalCollected: float,
     *   totalPayouts: float,
     *   transactionCount: int,
     *   availableBalance: float,
     *   points: list<array{label: string, in: float, out: float}>,
     *   boxes: list<array{name: string, codeName: string, collected: float}>,
     *   transactions: list<array<string, mixed>>
     * }
     */
    public static function empty(string $range): array
    {
        return [
            'totalCollected' => 0.0,
            'totalPayouts' => 0.0,
            'transactionCount' => 0,
            'availableBalance' => 0.0,
            'points' => self::emptyPoints($range),
            'boxes' => [
                ['name' => 'Checkout', 'codeName' => 'checkout', 'collected' => 0.0],
                ['name' => 'Virtual accounts', 'codeName' => 'va', 'collected' => 0.0],
                ['name' => 'Transfers', 'codeName' => 'transfers', 'collected' => 0.0],
            ],
            'transactions' => [],
        ];
    }

    /**
     * @return array{from: string, to: string, count: int}
     */
    public static function rangeWindow(string $range): array
    {
        $to = now();
        $from = match ($range) {
            'weekly' => now()->subWeeks(8)->startOfDay(),
            'monthly' => now()->subMonths(6)->startOfDay(),
            default => now()->subDays(13)->startOfDay(),
        };

        return [
            'from' => $from->format('Y-m-d'),
            'to' => $to->format('Y-m-d'),
            'count' => match ($range) {
                'weekly' => 200,
                'monthly' => 400,
                default => 100,
            },
        ];
    }

    /**
     * @param  list<array<string, mixed>>  $transactions
     * @return list<array{label: string, in: float, out: float}>
     */
    private static function buildPoints(array $transactions, string $range): array
    {
        $buckets = [];
        $labels = [];

        if ($range === 'monthly') {
            $start = now()->subMonths(5)->startOfMonth();
            foreach (CarbonPeriod::create($start, '1 month', now()->endOfMonth()) as $day) {
                $key = $day->format('Y-m');
                $labels[$key] = $day->format('M');
                $buckets[$key] = ['in' => 0.0, 'out' => 0.0];
            }
        } elseif ($range === 'weekly') {
            $start = now()->subWeeks(7)->startOfWeek();
            $i = 1;
            foreach (CarbonPeriod::create($start, '1 week', now()->endOfWeek()) as $day) {
                $key = $day->format('o-W');
                $labels[$key] = 'W'.$i;
                $buckets[$key] = ['in' => 0.0, 'out' => 0.0];
                $i++;
            }
        } else {
            $start = now()->subDays(13)->startOfDay();
            foreach (CarbonPeriod::create($start, '1 day', now()->endOfDay()) as $day) {
                $key = $day->format('Y-m-d');
                $labels[$key] = $day->format('M j');
                $buckets[$key] = ['in' => 0.0, 'out' => 0.0];
            }
        }

        foreach ($transactions as $tx) {
            /** @var Carbon|null $when */
            $when = $tx['date'] ?? null;
            if (! $when instanceof Carbon) {
                continue;
            }
            $key = match ($range) {
                'monthly' => $when->format('Y-m'),
                'weekly' => $when->format('o-W'),
                default => $when->format('Y-m-d'),
            };
            if (! isset($buckets[$key])) {
                continue;
            }
            if (! empty($tx['out'])) {
                $buckets[$key]['out'] += (float) $tx['amount'];
            } else {
                $buckets[$key]['in'] += (float) $tx['amount'];
            }
        }

        $points = [];
        foreach ($buckets as $key => $values) {
            $points[] = [
                'label' => $labels[$key] ?? $key,
                'in' => round($values['in'], 2),
                'out' => round($values['out'], 2),
            ];
        }

        return $points;
    }

    /**
     * @return list<array{label: string, in: float, out: float}>
     */
    private static function emptyPoints(string $range): array
    {
        return self::buildPoints([], $range);
    }

    /**
     * @param  mixed  $value
     * @return list<mixed>
     */
    private static function unwrapList(mixed $value): array
    {
        if (! is_array($value)) {
            return [];
        }
        if (array_is_list($value)) {
            return $value;
        }
        // Some envelopes nest again under data.
        if (isset($value['data']) && is_array($value['data'])) {
            return array_is_list($value['data']) ? $value['data'] : [];
        }

        return [];
    }

    private static function money(mixed $value): float
    {
        if (is_int($value) || is_float($value)) {
            return (float) $value;
        }
        if (! is_string($value)) {
            return 0.0;
        }
        $clean = preg_replace('/[^\d.\-]/', '', $value) ?? '0';

        return (float) $clean;
    }

    /**
     * @param  array<string, mixed>  $row
     */
    private static function isOutflow(string $type, array $row): bool
    {
        if (in_array($type, ['debit', 'withdrawal', 'transfer', 'payout', 'out'], true)) {
            return true;
        }
        $direction = strtolower((string) ($row['direction'] ?? $row['flow'] ?? ''));

        return in_array($direction, ['out', 'debit', 'payout'], true);
    }

    private static function parseDate(mixed $value): ?Carbon
    {
        if ($value instanceof Carbon) {
            return $value;
        }
        if (! is_string($value) || trim($value) === '') {
            return null;
        }
        try {
            return Carbon::parse($value);
        } catch (\Throwable) {
            return null;
        }
    }

    private static function statusLabel(mixed $status): string
    {
        $raw = strtolower((string) $status);
        if (in_array($raw, ['success', 'successful', 'completed', '00', 'paid'], true)) {
            return 'Successful';
        }
        if (in_array($raw, ['pending', 'processing', 'initiated'], true)) {
            return 'Pending';
        }
        if (in_array($raw, ['failed', 'error', 'declined', 'reversed'], true)) {
            return 'Failed';
        }

        return $status ? ucfirst((string) $status) : 'Successful';
    }

    private static function statusTone(mixed $status): string
    {
        return match (self::statusLabel($status)) {
            'Successful' => 'is-success',
            'Pending' => 'is-pending',
            default => 'is-failed',
        };
    }
}
