<?php

namespace App\Services;

use Illuminate\Http\Client\RequestException;
use Illuminate\Support\Facades\Http;
use RuntimeException;

class KoodifyClient
{
    public function __construct(
        private readonly string $baseUrl,
        private readonly string $secretKey,
    ) {}

    public static function fromConfig(): self
    {
        $secret = (string) config('koodify.secret_key');
        if ($secret === '') {
            throw new RuntimeException('Set KOODIFY_SECRET_KEY in .env (sk_… server key).');
        }

        return new self(
            rtrim((string) config('koodify.api_base'), '/'),
            $secret,
        );
    }

    /**
     * @param  array<string, mixed>  $payload
     * @return array<string, mixed> Koodify JSON envelope `{ code, status, description, data }`
     */
    public function createVirtualAccount(array $payload): array
    {
        return $this->postJson('/api/v1/va/create', $payload);
    }

    /**
     * @param  array<string, mixed>  $payload
     * @return array<string, mixed> Koodify JSON envelope `{ code, status, description, data }`
     */
    public function createDynamicVirtualAccount(array $payload): array
    {
        return $this->postJson('/api/v1/dva/create', $payload);
    }

    /**
     * @return array<string, mixed> Koodify JSON envelope `{ code, status, description, data }`
     */
    public function getVirtualAccountBalance(string $accountNumber): array
    {
        $accountNumber = trim($accountNumber);
        if ($accountNumber === '') {
            throw new RuntimeException('accountNumber is required.');
        }

        return $this->getJson('/api/v1/va/balance/'.rawurlencode($accountNumber));
    }

    /**
     * @param  array<string, mixed>  $payload
     * @return array<string, mixed>
     */
    private function postJson(string $path, array $payload): array
    {
        return $this->send('post', $path, $payload);
    }

    /**
     * @return array<string, mixed>
     */
    private function getJson(string $path): array
    {
        return $this->send('get', $path);
    }

    /**
     * @param  array<string, mixed>|null  $payload
     * @return array<string, mixed>
     */
    private function send(string $method, string $path, ?array $payload = null): array
    {
        try {
            $pending = Http::baseUrl($this->baseUrl)
                ->acceptJson()
                ->asJson()
                ->withHeaders([
                    'X-Api-Key' => $this->secretKey,
                ])
                ->timeout(45);

            $response = match ($method) {
                'get' => $pending->get($path)->throw(fn () => false),
                'post' => $pending->post($path, $payload ?? [])->throw(fn () => false),
                default => throw new RuntimeException("Unsupported HTTP method: {$method}"),
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
        ];
    }
}
