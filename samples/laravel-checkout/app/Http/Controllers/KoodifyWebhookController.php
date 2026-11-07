<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use Illuminate\Http\Response;
use Illuminate\Support\Facades\Log;

class KoodifyWebhookController extends Controller
{
    public function __invoke(Request $request): Response
    {
        $rawBody = $request->getContent();
        $signature = $request->header('X-Koodify-Signature');
        $secretKey = (string) config('koodify.secret_key');

        if ($secretKey === '') {
            Log::error('koodify.webhook.missing_secret_key');

            return response('server misconfigured', 500);
        }

        if (! $this->signatureValid($secretKey, $rawBody, $signature)) {
            Log::warning('koodify.webhook.invalid_signature', [
                'has_signature' => filled($signature),
            ]);

            return response('invalid signature', 401);
        }

        /** @var array<string, mixed> $payload */
        $payload = json_decode($rawBody, true) ?? [];
        $event = $request->header('X-Koodify-Event', data_get($payload, 'event'));
        $reference = data_get($payload, 'reference');

        Log::info('koodify.webhook.received', [
            'event' => $event,
            'reference' => $reference,
            'status' => data_get($payload, 'status'),
            'amount' => data_get($payload, 'amount'),
        ]);

        // Mark the order paid using $reference (idempotent on duplicates).

        return response('ok', 200);
    }

    private function signatureValid(string $secret, string $rawBody, ?string $signatureHeader): bool
    {
        if ($signatureHeader === null || $signatureHeader === '') {
            return false;
        }

        $provided = strtolower(trim($signatureHeader));
        $provided = str_starts_with($provided, 'sha256=')
            ? substr($provided, 7)
            : $provided;

        if ($provided === '' || ! ctype_xdigit($provided)) {
            return false;
        }

        // Same sk_… used as X-Api-Key — HMAC over the raw body bytes.
        $expected = hash_hmac('sha256', $rawBody, $secret);

        return hash_equals($expected, $provided);
    }
}
