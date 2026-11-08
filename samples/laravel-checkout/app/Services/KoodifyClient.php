<?php

namespace App\Services;

use Illuminate\Http\Client\ConnectionException;
use Illuminate\Http\Client\RequestException;
use Illuminate\Support\Facades\Http;
use RuntimeException;

class KoodifyClient
{
    public function __construct(
        private readonly string $baseUrl,
        private readonly ?string $secretKey = null,
        private readonly ?string $merchantToken = null,
    ) {}

    public static function fromConfig(): self
    {
        $secret = (string) config('koodify.secret_key');
        $secret = $secret !== '' ? $secret : null;

        return new self(
            rtrim((string) config('koodify.api_base'), '/'),
            $secret,
        );
    }

    public static function withMerchantToken(string $token): self
    {
        return new self(
            rtrim((string) config('koodify.api_base'), '/'),
            null,
            $token,
        );
    }

    public static function public(): self
    {
        return new self(rtrim((string) config('koodify.api_base'), '/'));
    }

    /**
     * Prefer the logged-in merchant's issued sk_…, else .env KOODIFY_SECRET_KEY.
     */
    public static function forMerchantApis(): self
    {
        $sessionSecret = (string) session('merchant.secret_key', '');
        if ($sessionSecret !== '') {
            return new self(
                rtrim((string) config('koodify.api_base'), '/'),
                $sessionSecret,
            );
        }

        return self::fromConfig();
    }

    /** @return array<string, mixed> */
    public function register(array $payload): array
    {
        return $this->send('post', '/api/v1/auth/register', $payload, auth: false);
    }

    /** @return array<string, mixed> */
    public function login(array $payload): array
    {
        return $this->send('post', '/api/v1/auth/login', $payload, auth: false);
    }

    /** @return array<string, mixed> */
    public function me(): array
    {
        return $this->send('get', '/api/v1/auth/me', auth: 'merchant');
    }

    /** @return array<string, mixed> */
    public function submitKyc(array $payload): array
    {
        return $this->send('put', '/api/v1/auth/kyc', $payload, auth: 'merchant');
    }

    /** @return array<string, mixed> */
    public function getCompliance(): array
    {
        return $this->send('get', '/api/v1/auth/compliance', auth: 'merchant');
    }

    /**
     * @param  array<string, mixed>  $data
     * @return array<string, mixed>
     */
    public function submitComplianceSection(string $section, array $data): array
    {
        return $this->send(
            'put',
            '/api/v1/auth/compliance/'.rawurlencode($section),
            ['data' => $data],
            auth: 'merchant',
        );
    }

    /**
     * Multipart document upload (cacCertificate + utilityBill).
     *
     * @param  array<string, \Illuminate\Http\UploadedFile|string|null>  $parts
     * @return array<string, mixed>
     */
    public function submitComplianceDocuments(array $parts): array
    {
        $token = $this->merchantToken ?: (string) session('merchant.token', '');
        if ($token === '') {
            throw new RuntimeException('Merchant session token missing — log in again.');
        }

        try {
            $pending = Http::baseUrl($this->baseUrl)
                ->acceptJson()
                ->timeout(60)
                ->withHeaders(['Authorization' => 'Bearer '.$token]);

            foreach ($parts as $name => $value) {
                if ($value instanceof \Illuminate\Http\UploadedFile) {
                    $pending = $pending->attach(
                        $name,
                        fopen($value->getRealPath(), 'r'),
                        $value->getClientOriginalName(),
                    );
                } elseif (is_string($value) && $value !== '') {
                    $pending = $pending->attach($name, $value);
                }
            }

            $response = $pending->post('/api/v1/auth/compliance/documents');
        } catch (ConnectionException $e) {
            return [
                'code' => '99',
                'status' => 'connection_error',
                'description' => 'Could not reach Koodify at '.$this->baseUrl.'. '.$e->getMessage(),
                'data' => null,
            ];
        }

        /** @var array<string, mixed>|null $json */
        $json = $response->json();
        if (is_array($json)) {
            return $json;
        }

        $body = trim((string) $response->body());
        $status = (int) $response->status();

        return [
            'code' => (string) $status,
            'status' => (string) $status,
            'description' => $body !== ''
                ? $body
                : "Koodify returned HTTP {$status} with an empty body for /api/v1/auth/compliance/documents.",
            'data' => null,
        ];
    }

    /** @return array<string, mixed> */
    public function issueSecret(string $environment = 'live'): array
    {
        $environment = $environment === 'test' ? 'test' : 'live';

        return $this->send(
            'post',
            '/api/v1/auth/credentials/'.$environment.'/issue-secret',
            auth: 'merchant',
        );
    }

    /**
     * @param  array<string, mixed>  $payload
     * @return array<string, mixed>
     */
    public function createVirtualAccount(array $payload): array
    {
        return $this->send('post', '/api/v1/va/create', $payload, auth: 'secret');
    }

    /**
     * @param  array<string, mixed>  $payload
     * @return array<string, mixed>
     */
    public function createDynamicVirtualAccount(array $payload): array
    {
        return $this->send('post', '/api/v1/dva/create', $payload, auth: 'secret');
    }

    /** @return array<string, mixed> */
    public function listVirtualAccounts(): array
    {
        return $this->send('get', '/api/v1/vas', auth: 'secret');
    }

    /** @return array<string, mixed> */
    public function getVirtualAccountBalance(string $accountNumber): array
    {
        $accountNumber = trim($accountNumber);
        if ($accountNumber === '') {
            throw new RuntimeException('accountNumber is required.');
        }

        return $this->send(
            'get',
            '/api/v1/va/balance/'.rawurlencode($accountNumber),
            auth: 'secret',
        );
    }

    /**
     * @param  array<string, mixed>|null  $payload
     * @return array<string, mixed>
     */
    private function send(
        string $method,
        string $path,
        ?array $payload = null,
        bool|string $auth = 'secret',
    ): array {
        try {
            $pending = Http::baseUrl($this->baseUrl)
                ->acceptJson()
                ->asJson()
                ->timeout(45);

            if ($auth === 'merchant') {
                $token = $this->merchantToken ?: (string) session('merchant.token', '');
                if ($token === '') {
                    throw new RuntimeException('Merchant session token missing — log in again.');
                }
                $pending = $pending->withHeaders([
                    'Authorization' => 'Bearer '.$token,
                ]);
            } elseif ($auth === 'secret' || $auth === true) {
                $secret = $this->secretKey ?: (string) session('merchant.secret_key', '');
                if ($secret === '') {
                    throw new RuntimeException('Set KOODIFY_SECRET_KEY or issue a merchant sk_… from Credentials.');
                }
                $pending = $pending->withHeaders([
                    'X-Api-Key' => $secret,
                ]);
            }

            $response = match ($method) {
                'get' => $pending->get($path),
                'post' => $pending->post($path, $payload ?? []),
                'put' => $pending->put($path, $payload ?? []),
                'patch' => $pending->patch($path, $payload ?? []),
                default => throw new RuntimeException("Unsupported HTTP method: {$method}"),
            };
        } catch (ConnectionException $e) {
            return [
                'code' => '99',
                'status' => 'connection_error',
                'description' => 'Could not reach Koodify at '.$this->baseUrl.'. '.$e->getMessage(),
                'data' => null,
            ];
        } catch (RequestException $e) {
            $response = $e->response;
            if ($response === null) {
                return [
                    'code' => '99',
                    'status' => 'request_error',
                    'description' => $e->getMessage(),
                    'data' => null,
                ];
            }
        }

        /** @var array<string, mixed>|null $json */
        $json = $response->json();
        if (is_array($json)) {
            // Axum plain-text errors sometimes land as JSON strings; normal envelopes are arrays.
            return $json;
        }

        $body = trim((string) $response->body());
        $status = (int) $response->status();

        return [
            'code' => (string) $status,
            'status' => (string) $status,
            'description' => $body !== ''
                ? $body
                : "Koodify returned HTTP {$status} with an empty body for {$path}. Check KOODIFY_API_BASE ({$this->baseUrl}).",
            'data' => null,
        ];
    }
}
