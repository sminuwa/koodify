<?php

namespace App\Services;

use Illuminate\Http\Client\RequestException;
use Illuminate\Support\Facades\Http;
use RuntimeException;

/**
 * Platform admin calls using GATEWAY_API_KEY (X-Api-Key).
 */
class KoodifyAdminClient
{
    public function __construct(
        private readonly string $baseUrl,
        private readonly string $gatewayApiKey,
    ) {}

    public static function fromConfig(): self
    {
        $key = (string) config('koodify.gateway_api_key');
        if ($key === '') {
            throw new RuntimeException('Set KOODIFY_GATEWAY_API_KEY in .env (platform admin key).');
        }

        return new self(
            rtrim((string) config('koodify.api_base'), '/'),
            $key,
        );
    }

    /** @return array<string, mixed> */
    public function listCustomers(?string $status = null): array
    {
        $query = array_filter(['status' => $status]);

        return $this->send('get', '/api/v1/customers', null, $query);
    }

    /** @return array<string, mixed> */
    public function getCustomer(int $id): array
    {
        return $this->send('get', '/api/v1/customers/'.$id);
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
            $pending = Http::baseUrl($this->baseUrl)
                ->acceptJson()
                ->asJson()
                ->withHeaders(['X-Api-Key' => $this->gatewayApiKey])
                ->timeout(45);

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
