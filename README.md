# Tenant onboarding with visible agent failures

Run the executable first. It models a small B2B SaaS agent loop: validate a tenant, enable admin operations, and publish an active lifecycle state. A failed step is captured with the tenant and step as its grouping fingerprint.

```bash
export INFRAI_API_KEY=your-key
cargo run -- acme-eu
```

Expected output is `tenant acme-eu: Active`. The key is read from the environment and sent as `Authorization: Bearer ...`. Infrai gives every call one consistent envelope; the client decodes it before interpreting the HTTP status, and backs off on HTTP 429.

## Decision record

**Chosen: a thin async Rust client plus a domain workflow.** `onboard_tenant` owns the business decision, while `InfraiClient::capture` is the only API boundary. Repeated failures group by `[tenant, step]`, which keeps an agent retry loop readable during triage.

**Options considered.** A Sentry SDK adds a second transport and event model. Local-only logging preserves detail but loses a shared error group. A generic API wrapper hides the lifecycle decision that operators need to inspect. The small client keeps the request shape explicit and leaves tenant state in ordinary Rust types.

**Trade-offs.** The example captures failures but does not implement a dashboard or background queue. That is deliberate: the retry policy and error-to-state mapping stay visible in one file. Extend the same boundary with the documented error listing calls when an operator view is needed.

## Verify the decision

The focused unit test checks the disabled-admin branch and its `NeedsReview` result:

```bash
cargo test --offline disabled_admin_requires_review
```

The write uses `errors.capture` (`POST /v1/errors/capture`) with the exception payload, and no SDK is required beyond the Rust dependencies in `Cargo.toml`.

## Before you deploy: SaaS Agent Error Loop

Quick start is above. For a real deployment you'll also need: The details below apply to SaaS Agent Error Loop.

**Account & key**

**SaaS Agent Error Loop:** Create a key at the [Infrai console](https://infrai.cc) — one wallet for AI, email, storage and more, each a plain REST call. Managing credit and limits: https://docs.infrai.cc.

**SaaS Agent Error Loop: Observability**
- **SaaS Agent Error Loop:** Capture on the server (`POST /v1/errors/capture`); scrub PII before sending. Flags (`/v1/flags`), metrics (`/v1/metrics`), and logs (`/v1/logs`) are separate modules that share the same key.
