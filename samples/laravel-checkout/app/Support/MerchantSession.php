<?php

namespace App\Support;

final class MerchantSession
{
    /**
     * @param  array<string, mixed>  $data  Auth `data` object from Koodify
     */
    public static function storeLogin(array $data): void
    {
        $token = (string) ($data['token'] ?? '');
        if ($token === '') {
            throw new \RuntimeException('Koodify did not return a session token.');
        }

        session([
            'merchant.token' => $token,
            'merchant.id' => $data['id'] ?? null,
            'merchant.display_name' => $data['displayName'] ?? $data['display_name'] ?? '',
            'merchant.email' => $data['email'] ?? '',
            'merchant.public_key' => $data['publicKey'] ?? $data['public_key'] ?? null,
            'merchant.profile' => $data,
        ]);
    }

    /**
     * Capture one-time secrets from KYC provision or issue-secret.
     *
     * @param  array<string, mixed>  $payload
     */
    public static function captureSecrets(array $payload): void
    {
        $minted = $payload['mintedSecrets'] ?? $payload['provision']['mintedSecrets'] ?? null;
        if (is_array($minted)) {
            foreach ($minted as $row) {
                if (! is_array($row)) {
                    continue;
                }
                $env = (string) ($row['environment'] ?? '');
                $secret = (string) ($row['secretKey'] ?? '');
                if ($secret === '') {
                    continue;
                }
                if ($env === 'live' || (! session()->has('merchant.secret_key') && $env === 'test')) {
                    session(['merchant.secret_key' => $secret]);
                }
                session(["merchant.secret_{$env}" => $secret]);
            }
        }

        if (! empty($payload['secretKey'])) {
            $env = (string) ($payload['environment'] ?? 'live');
            session([
                'merchant.secret_key' => (string) $payload['secretKey'],
                "merchant.secret_{$env}" => (string) $payload['secretKey'],
            ]);
        }

        if (! empty($payload['publicKey'])) {
            session(['merchant.public_key' => (string) $payload['publicKey']]);
        } elseif (! empty($payload['provision']['publicKey'])) {
            session(['merchant.public_key' => (string) $payload['provision']['publicKey']]);
        }
    }

    public static function clear(): void
    {
        session()->forget([
            'merchant.token',
            'merchant.id',
            'merchant.display_name',
            'merchant.email',
            'merchant.public_key',
            'merchant.secret_key',
            'merchant.secret_live',
            'merchant.secret_test',
            'merchant.profile',
        ]);
    }

    public static function isLoggedIn(): bool
    {
        return session()->has('merchant.token');
    }
}
