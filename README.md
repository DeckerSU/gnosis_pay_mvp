# Gnosis Pay Rust MVP CLI

This repository contains a Rust MVP command-line application that demonstrates the Gnosis Pay authentication flow and then performs an authenticated KYC demo request to **Retrieve Source of Funds questions**. The application generates a new EVM wallet, prints the address and private key for demonstration purposes, obtains an authentication nonce, builds and signs a SIWE message, receives a pre-signup JWT, requests an email OTP, creates a new user, and finally uses the resulting JWT to call `GET /api/v1/source-of-funds`.

> **Security note.** The private key and JWT are printed only because this is an MVP/demo application. In a real product, private keys, JWTs, OTPs, and email mailbox credentials must never be printed to stdout, committed to source control, or stored unencrypted.

## Implemented scope

| Area | Implementation |
|---|---|
| Wallet generation | Creates a new EVM address and private key with `ethers-signers`. |
| SIWE signing | Builds a Sign-In with Ethereum message containing the nonce, domain, URI, chain ID, version, and issued-at timestamp following the EIP-4361 format, then signs it with the generated wallet. |
| Authentication API | Implements `GET /api/v1/auth/nonce`, `POST /api/v1/auth/challenge`, `POST /api/v1/auth/signup/otp`, and `POST /api/v1/auth/signup`. |
| KYC demo request | Implements `GET /api/v1/source-of-funds` with `Authorization: Bearer <JWT>`. |
| SDK module | Keeps API types, HTTP calls, and wallet/SIWE helpers under `src/sdk`. |
| User input | Requests email, OTP, optional marketing campaign, optional partner ID, and optional Source of Funds locale through the terminal. |
| Temporary email helpers | Includes Python scripts under `scripts/` for creating a `mail.tm` mailbox, polling for the OTP, and running the full verification harness. |

## Project structure

```text
gnosis_pay_mvp/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── rust-toolchain.toml
├── scripts
│   ├── create_mailtm_account.py
│   ├── poll_mailtm_otp.py
│   └── verify_full_flow.py
└── src
    ├── main.rs
    └── sdk
        ├── client.rs
        ├── mod.rs
        ├── models.rs
        └── wallet.rs
```

The `src/sdk` directory is intentionally separated from the CLI orchestration code. `client.rs` contains the Gnosis Pay API client, `models.rs` contains request and response models, and `wallet.rs` contains EVM wallet and SIWE helper logic.

## Requirements

Install the Rust stable toolchain. If Rust is not installed yet, use the official installer:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

The repository includes `rust-toolchain.toml`, so environments that use `rustup` will select the stable toolchain automatically. The Python helper scripts use only the Python standard library and therefore do not require extra Python packages.

## Build

From the project root, run:

```bash
cargo build
```

For an optimized build, run:

```bash
cargo build --release
```

## Run the CLI manually

Start the MVP with:

```bash
cargo run
```

The CLI asks for several technical values first. For a basic demo, press Enter to accept the defaults shown in brackets.

| Prompt | Default value | Purpose |
|---|---:|---|
| `Gnosis Pay API base URL` | `https://api.gnosispay.com` | Production API base URL used by the SDK client. |
| `SIWE domain` | `app.gnosispay.com` | The domain placed in the first line of the SIWE message and bound to the signature. |
| `SIWE URI` | `https://app.gnosispay.com` | The URI placed in the SIWE message, representing the application origin/resource for which the user signs in. |
| `SIWE chain ID` | `100` | Gnosis Chain ID. |
| `JWT TTL in seconds` | `3600` | Requested token lifetime for the challenge flow. |

After the SIWE challenge succeeds, the CLI asks for an email address. It then requests an OTP and waits at the `OTP:` prompt. Open the inbox for that email address, copy the 6-digit code, paste it into the terminal, and continue. The CLI then asks for optional `marketingCampaign`, optional `partnerId`, and optional Source of Funds `locale`, before printing the final questions JSON.

## Using the temporary email helper scripts

The project includes Python scripts that can make OTP testing easier. They use the public `mail.tm` temporary mailbox API to create a disposable inbox and read incoming emails. These scripts are included for development and verification only.

| Script | Purpose | Typical command |
|---|---|---|
| `scripts/create_mailtm_account.py` | Creates a temporary mailbox and writes `mailtm_credentials.txt` with the email and password. | `python3 scripts/create_mailtm_account.py` |
| `scripts/poll_mailtm_otp.py` | Polls the mailbox from `mailtm_credentials.txt` and prints the first 6-digit OTP it finds. | `python3 scripts/poll_mailtm_otp.py` |
| `scripts/verify_full_flow.py` | Runs the Rust CLI end-to-end, feeds the generated email, waits for the OTP, submits it, and verifies that Source of Funds questions are printed. | `python3 scripts/verify_full_flow.py` |

A convenient manual OTP flow is as follows. First, create a mailbox:

```bash
python3 scripts/create_mailtm_account.py
```

The script prints `EMAIL=<address>` and saves the credentials to `mailtm_credentials.txt`. Next, start the Rust CLI in another terminal and paste the printed email when prompted:

```bash
cargo run
```

When the Rust CLI reaches the `OTP:` prompt, return to the first terminal and poll the mailbox:

```bash
python3 scripts/poll_mailtm_otp.py --timeout 180
```

Paste the printed 6-digit OTP into the Rust CLI. If the code has not arrived yet, rerun the polling command or increase `--timeout`.

For a fully automated development check, create a mailbox first and then run the verification harness:

```bash
python3 scripts/create_mailtm_account.py
python3 scripts/verify_full_flow.py
```

The harness writes `verification_full_flow.log` locally. The archive excludes generated credentials, logs, and build artifacts through `.gitignore`.

## What SIWE domain and SIWE URI mean here

In this MVP, `SIWE domain` and `SIWE URI` are explicit CLI inputs because they are part of the signed SIWE message. The generated EVM wallet signs a message that says, in effect, that a specific Ethereum address is signing in to a specific domain and URI. Gnosis Pay then verifies the signature and issues a JWT if the message and signature are acceptable.

| Field | In this MVP | In a real project |
|---|---|---|
| `SIWE domain` | Defaults to `app.gnosispay.com` and appears in the SIWE message line `<domain> wants you to sign in with your Ethereum account:`. | Should be the relying party domain that the user intentionally signs into, typically the web app domain or the domain expected by the API provider. It should not be user-controlled input. |
| `SIWE URI` | Defaults to `https://app.gnosispay.com` and appears as the `URI:` field in the SIWE message. | Should identify the application origin or resource associated with the login. It should be stable, use HTTPS in production, and match the security expectations of the backend. |

The MVP uses `https://app.gnosispay.com` rather than `http://localhost` as the default URI because the tested Gnosis Pay API/WAF rejected challenge requests containing a localhost HTTP URI in the SIWE message body. In a production integration, the domain and URI should be part of trusted server-side configuration rather than values typed by the end user.

## Expected output

A successful run prints sections similar to the following:

```text
Received pre-signup JWT token.
OTP request accepted: true
...
User ID: <id>
hasSignedUp: true
Received final JWT token for subsequent requests.
...
Source of Funds questions:
[
  {
    "question": "What is your employment status?",
    "answers": ["Employed", "Self-employed", "Unemployed", "Retired", "Student", "Homemaker"]
  }
]
```

## Verification performed before delivery

The MVP was compiled with `cargo build` and tested end-to-end using a temporary `mail.tm` mailbox. The verified flow covered EVM wallet generation, nonce retrieval, SIWE challenge verification, pre-signup JWT issuance, OTP email delivery, signup, final JWT issuance, and `GET /api/v1/source-of-funds`. The Source of Funds endpoint returned the expected questions and answer options.

## Production hardening notes

This repository is intentionally small and demonstration-oriented. A production implementation should move key management to a wallet or secure signing component, avoid printing secrets, add structured error handling and retry/backoff policies, use configuration management for API URLs and SIWE values, add tests against a mock server, and store audit logs without sensitive data.

## References

1. [Gnosis Pay API Reference: Generate nonce](https://docs.gnosispay.com/api-reference/authentication/generate-nonce)
2. [Gnosis Pay API Reference: Retrieve Source of Funds questions](https://docs.gnosispay.com/api-reference/kyc/retrieve-source-of-funds-questions)
3. [Sign-In with Ethereum documentation](https://docs.login.xyz/)
4. [EIP-4361: Sign-In with Ethereum](https://eips.ethereum.org/EIPS/eip-4361)
