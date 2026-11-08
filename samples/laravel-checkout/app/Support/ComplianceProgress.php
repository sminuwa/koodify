<?php

namespace App\Support;

final class ComplianceProgress
{
    public const SECTIONS = [
        'business',
        'representative',
        'documents',
        'declaration',
    ];

    /**
     * Normalize gateway compliance payload into portal shape.
     *
     * @param  array<string, mixed>|null  $raw
     * @return array<string, array{status: string, submittedAt: ?string, data: array<string, mixed>}>
     */
    public static function fromGateway(?array $raw): array
    {
        $out = [];
        foreach (self::SECTIONS as $key) {
            $row = is_array($raw[$key] ?? null) ? $raw[$key] : [];
            $out[$key] = [
                'status' => (string) ($row['status'] ?? 'none'),
                'submittedAt' => isset($row['submittedAt']) ? (string) $row['submittedAt'] : null,
                'data' => is_array($row['data'] ?? null) ? $row['data'] : [],
            ];
        }

        return $out;
    }

    /**
     * @return array<string, array{status: string, submittedAt: ?string, data: array<string, mixed>}>
     */
    public static function empty(): array
    {
        return self::fromGateway(null);
    }

    public static function status(array $compliance, string $section): string
    {
        return $compliance[$section]['status'] ?? 'none';
    }

    /**
     * @return array<string, mixed>
     */
    public static function data(array $compliance, string $section): array
    {
        return $compliance[$section]['data'] ?? [];
    }
}
