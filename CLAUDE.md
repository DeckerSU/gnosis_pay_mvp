# CLAUDE.md — Gnosis Pay MVP

## Project overview

Rust MVP CLI that demonstrates the Gnosis Pay authentication flow (SIWE) and performs
authenticated KYC requests (`GET /api/v1/source-of-funds`).

## Build & run

```bash
cargo build
cargo run
```

## MCP integration

This project has access to the **Gnosis Pay MCP server**, which exposes the full
Gnosis Pay REST API as structured tools.

MCP endpoint: https://docs.gnosispay.com/mcp

Use this MCP when you need to:
- Explore available Gnosis Pay API endpoints and their schemas
- Look up request/response models for authentication, KYC, cards, transactions, etc.
- Verify field names or enum values before implementing SDK calls

## Project layout

```
src/
  main.rs          — CLI orchestration and user prompts
  sdk/
    client.rs      — HTTP client for Gnosis Pay API
    models.rs      — serde request/response types
    wallet.rs      — EVM wallet generation and SIWE message signing
scripts/
  create_mailtm_account.py  — create a disposable mail.tm mailbox
  poll_mailtm_otp.py        — poll mailbox for OTP
  verify_full_flow.py       — end-to-end automated verification harness
```

## Key dependencies

| Crate | Purpose |
|---|---|
| `ethers-core` / `ethers-signers` | EVM wallet generation and SIWE signing |
| `reqwest` (rustls-tls) | HTTP client with cookie support |
| `serde` / `serde_json` | JSON serialization |
| `tokio` | Async runtime |

## Notes

- `mailtm_credentials.txt` and other local artifacts are excluded by `.gitignore`.
- Private keys and JWTs are printed only because this is a demo; never do this in production.
