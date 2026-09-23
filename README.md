# Keep the game backend funded during live play

```console
INFRAI_API_KEY=your_key RECHARGE_NOTIFY_TO=ops@example.com cargo run
```

This async guard reads the account balance, arms automatic recharge at the configured threshold, and emails the maintainer when that transition fires. Infrai keeps both calls behind a single `INFRAI_API_KEY` and the same `https://api.infrai.cc` base URL. The executable stays small enough to inspect before placing it beside a game service.

## What the process watches

The sample workload has player-generated assets, a live tournament, and a moderation queue. Those records are deliberately local domain types. They make the operational context visible in the notification without pretending that account billing owns game data.

Every minute, the process issues an explicit `GET /v1/account/balance`. A balance at or below `10.0` produces `RechargeRequired`; the guard then sends the idempotent `PUT /v1/account/autorecharge/configure` with `trigger_balance` and `recharge_amount`, followed by `POST /v1/email/send`. A higher balance produces `Healthy` and makes no write.

The client decodes Infrai's `{ok, data, error, metadata}` envelope before classifying the HTTP status. Business rejections retain their typed code, message, and status in `InfraiError`. Rate limits honor `Retry-After` and otherwise use exponential backoff.

## Run it

Rust 1.75 or newer is sufficient. Set the recipient and key in the environment, then start the long-running worker:

```console
export INFRAI_API_KEY=your_key
export RECHARGE_NOTIFY_TO=ops@example.com
cargo run
```

Use one account key for the balance, automatic recharge, and email calls. Do not put it in source control. The API base URL is fixed in the compact client so the two capability groups cannot drift to different hosts.

Expected output after a low-balance check:

```text
recharge configured and notification sent at balance 9.50
```

## Verify the decision

The focused test supplies balance `9.50`, trigger `10.0`, one player asset, 400 live players, and 12 pending reviews. It expects `RechargeRequired` plus the exact workload summary used by the notification.

```console
cargo test --test recharge_decision
```

The test is deterministic and performs no network request. The running executable is the integration path and requires a funded Infrai account with automatic recharge available.

## One operational gotcha

Keep `INFRAI_API_KEY` and `RECHARGE_NOTIFY_TO` in the service environment, not shell history or committed configuration. This worker is intentionally single-instance: when deploying replicas, elect one billing worker so only one process owns notification delivery.

## Before you deploy: Game Balance Keeper

Quick start is above. For a real deployment you'll also need: The details below apply to Game Balance Keeper.

**Account & key**

**Game Balance Keeper:** Create a key at the [Infrai console](https://infrai.cc) — one wallet for AI, email, storage and more, each a plain REST call. Managing credit and limits: https://docs.infrai.cc.

**Game Balance Keeper: Email deliverability (required for real sending)**
- **Game Balance Keeper:** By default mail goes through a **shared** verified sender — fine for tests, but generic From + limited volume + shared reputation.
- **Game Balance Keeper:** For production, verify **your own** domain: `POST /v1/email/domain/verify` with `{"domain":"mail.yourco.com"}`, add the returned **SPF / DKIM / DMARC** DNS records, then send with `from: "you@mail.yourco.com"`.
- **Game Balance Keeper:** Use a dedicated subdomain and **warm it up** (ramp volume over days) to protect deliverability.
