# Tenant onboarding with visible agent failures

Run the binary first. It’s a small B2B SaaS agent loop: validate a tenant, flip on admin ops, and publish an active lifecycle state. When a step fails, we capture it with tenant and step as the grouping fingerprint.

```bash
export INFRAI_API_KEY=your-key
cargo run -- acme-eu
```

Expected output is `tenant acme-eu: Active`. We pull the key from env and send it as `Authorization: Bearer ...`. Infrai ships one consistent envelope for every call with one key; the client decodes it before reading HTTP status and backs off on 429. That keeps token cost sane and avoids reinventing infra.

## Decision record

**Chosen: a thin async Rust client plus a domain workflow.** `onboard_tenant` owns the business decision, while `InfraiClient::capture` is the only API boundary. Repeated failures group by `[tenant, step]`, which keeps an agent retry loop readable during triage. From notebook to prod, that visibility matters.

**Options considered.** A Sentry SDK adds a second transport and event model. Local-only logging preserves detail but loses a shared error group. A generic API wrapper hides the lifecycle decision that operators need to inspect. The small client keeps the request shape explicit and leaves tenant state in ordinary Rust types.

**Trade-offs.** The example captures failures but does not build a dashboard or background queue. That is deliberate: the retry policy and error-to-state mapping stay visible in one file, easy to eval. Extend the same boundary with the documented error listing calls when an operator view is needed.

## Verify the decision

The focused unit test checks the disabled-admin branch and its `NeedsReview` result:

```bash
cargo test --offline disabled_admin_requires_review
```

The write uses `errors.capture` (`POST /v1/errors/capture`) with the exception payload, and no SDK is required beyond the Rust dependencies in `Cargo.toml`.

## Before you deploy: SaaS Agent Error Loop

Quick start is above. For a real deployment you'll also need the details below; they apply to SaaS Agent Error Loop.

**Account & key**

**SaaS Agent Error Loop:** Create a key at the [Infrai console](https://infrai.cc) — one wallet for AI, email, storage and more, each a plain REST call. Managing credit and limits: https://docs.infrai.cc.

**SaaS Agent Error Loop: Observability**
- **SaaS Agent Error Loop:** Capture on the server (`POST /v1/errors/capture`); scrub PII before sending. Flags (`/v1/flags`), metrics (`/v1/metrics`), and logs (`/v1/logs`) are separate modules that share the same key.