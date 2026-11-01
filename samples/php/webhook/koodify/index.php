<?php
/**
 * Demo merchant webhook for Koodify checkout `callBackUrl`.
 *
 * URL:  https://demo.koodify.ng/webhooks/koodify/   (trailing slash)
 * Path: /webhooks/koodify/index.php
 *
 * On payment success Koodify POSTs a JSON body, for example:
 * {
 *   "event": "payment.success",
 *   "status": "success",
 *   "reference": "LIVE_…",
 *   "amount": "500",
 *   "currencyCode": "NGN",
 *   "zainboxCode": "…",
 *   "payer": { … },
 *   "customParam": { … },
 *   "paymentSplit": { … }
 * }
 *
 * This script only receives that POST and appends it to webhook-log.jsonl.
 */

declare(strict_types=1);

const LOG_FILE = __DIR__ . '/webhook-log.jsonl';

if (($_SERVER['REQUEST_METHOD'] ?? 'GET') !== 'POST') {
    http_response_code(405);
    header('Content-Type: text/plain; charset=utf-8');
    echo "POST only\n";
    exit;
}

$raw = file_get_contents('php://input');
if ($raw === false || $raw === '') {
    http_response_code(400);
    header('Content-Type: text/plain; charset=utf-8');
    echo "empty body\n";
    exit;
}

$payload = json_decode($raw, true);
if (!is_array($payload)) {
    http_response_code(400);
    header('Content-Type: text/plain; charset=utf-8');
    echo "invalid JSON\n";
    exit;
}

$entry = [
    'receivedAt' => gmdate('c'),
    'reference' => $payload['reference'] ?? null,
    'event' => $payload['event'] ?? null,
    'payload' => $payload,
];

if (file_put_contents(
    LOG_FILE,
    json_encode($entry, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE) . "\n",
    FILE_APPEND | LOCK_EX
) === false) {
    http_response_code(500);
    header('Content-Type: text/plain; charset=utf-8');
    echo "log write failed\n";
    exit;
}

http_response_code(200);
header('Content-Type: text/plain; charset=utf-8');
echo "OK\n";
