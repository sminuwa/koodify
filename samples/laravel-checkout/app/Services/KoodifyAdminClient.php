<?php

namespace App\Services;

use Illuminate\Http\Client\RequestException;
use Illuminate\Support\Facades\Http;
use RuntimeException;

/**
 * Admin portal client — prefers Bearer at_… session; falls back to GATEWAY_API_KEY.
 */
class KoodifyAdminClient
{
    public function __construct(
        private readonly string $baseUrl,
        private readonly ?string $adminToken = null,
        private readonly ?string $gatewayApiKey = null,
    ) {}

    public static function fromSession(): self
    {
        $token = (string) session('admin.token', '');
        $key = (string) config('koodify.gateway_api_key');
        if ($token === '' && $key === '') {
            throw new RuntimeException('Admin session expired. Sign in again.');
        }

        return new self(
            rtrim((string) config('koodify.api_base'), '/'),
            $token !== '' ? $token : null,
            $key !== '' ? $key : null,
        );
    }

    /** @deprecated Use fromSession() after /admin/auth login. */
    public static function fromConfig(): self
    {
        return self::fromSession();
    }

    /**
     * Public login (no prior auth).
     *
     * @return array<string, mixed>
     */
    public static function login(string $username, string $password): array
    {
        $client = new self(rtrim((string) config('koodify.api_base'), '/'));

        return $client->send('post', '/api/v1/admin/auth/login', [
            'username' => $username,
            'password' => $password,
        ]);
    }

    /** @return array<string, mixed> */
    public function me(): array
    {
        return $this->send('get', '/api/v1/admin/auth/me');
    }

    /** @return array<string, mixed> */
    public function dashboard(): array
    {
        return $this->send('get', '/api/v1/admin/dashboard');
    }

    /** @return array<string, mixed> */
    public function listCustomers(?string $status = null): array
    {
        return $this->send('get', '/api/v1/admin/merchants');
    }

    /** @return array<string, mixed> */
    public function getCustomer(int $id): array
    {
        return $this->send('get', '/api/v1/admin/merchants/'.$id);
    }

    /** @return array<string, mixed> */
    public function getKycByCustomer(int $customerId): array
    {
        return $this->send('get', '/kyc/by-customer/'.$customerId);
    }

    /** @return array<string, mixed> */
    public function updateKycStatus(string $kycId, string $status): array
    {
        return $this->send('patch', '/kyc/'.$kycId.'/status', ['status' => $status]);
    }

    /** @return array<string, mixed> */
    public function approveMerchantKyc(int $customerId): array
    {
        return $this->send('post', '/api/v1/admin/merchants/'.$customerId.'/kyc/approve');
    }

    /** @return array<string, mixed> */
    public function rejectMerchantKyc(int $customerId): array
    {
        return $this->send('post', '/api/v1/admin/merchants/'.$customerId.'/kyc/reject');
    }

    /** @return array<string, mixed> */
    public function setComplianceSection(int $customerId, string $section, string $status): array
    {
        return $this->send(
            'patch',
            '/api/v1/admin/merchants/'.$customerId.'/compliance/'.$section,
            ['status' => $status],
        );
    }

    /** @return array<string, mixed> */
    public function provisionCustomer(int $id): array
    {
        return $this->send('post', '/api/v1/customers/'.$id.'/provision');
    }

    /**
     * @param  array<string, mixed>|null  $payload
     * @param  array<string, mixed>  $query
     * @return array<string, mixed>
     */
    private function send(string $method, string $path, ?array $payload = null, array $query = []): array
    {
        try {
            $headers = [];
            if ($this->adminToken) {
                $headers['Authorization'] = 'Bearer '.$this->adminToken;
            }
            if ($this->gatewayApiKey) {
                $headers['X-Api-Key'] = $this->gatewayApiKey;
            }

            $pending = Http::baseUrl($this->baseUrl)
                ->acceptJson()
                ->asJson()
                ->withHeaders($headers)
                ->timeout(60);

            $response = match ($method) {
                'get' => $pending->get($path, $query),
                'post' => $pending->post($path, $payload ?? []),
                'put' => $pending->put($path, $payload ?? []),
                'patch' => $pending->patch($path, $payload ?? []),
                default => throw new RuntimeException("Unsupported method: {$method}"),
            };
        } catch (RequestException $e) {
            $response = $e->response;
            if ($response === null) {
                throw $e;
            }
        }

        /** @var array<string, mixed>|null $json */
        $json = $response->json();
        if (is_array($json)) {
            return $json;
        }

        return [
            'code' => (string) $response->status(),
            'status' => (string) $response->status(),
            'description' => $response->body() ?: 'empty response',
            'data' => null,
        ];
    }
}
