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
     * @return array{status: int, body: array<string, mixed>}
     */
    public function createVirtualAccount(array $payload): array
    {
        return $this->postJson('/api/v1/va/create', $payload);
    }

    /**
     * @param  array<string, mixed>  $payload
     * @return array{status: int, body: array<string, mixed>}
     */
    public function createDynamicVirtualAccount(array $payload): array
    {
        return $this->postJson('/api/v1/dva/create', $payload);
    }

    /**
     * @param  array<string, mixed>  $payload
     * @return array{status: int, body: array<string, mixed>}
     */
    private function postJson(string $path, array $payload): array
    {
        try {
            $response = Http::baseUrl($this->baseUrl)
                ->acceptJson()
                ->asJson()
                ->withHeaders([
                    'X-Api-Key' => $this->secretKey,
                ])
                ->timeout(45)
                ->post($path, $payload)
                ->throw(fn () => false);
        } catch (RequestException $e) {
            $response = $e->response;
            if ($response === null) {
                throw $e;
            }
        }

        /** @var array<string, mixed> $body */
        $body = $response->json() ?? [
            'description' => $response->body() ?: 'empty response',
        ];

        return [
            'status' => $response->status(),
            'body' => $body,
        ];
    }
}
