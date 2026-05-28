#!/usr/bin/env python3
"""Create a temporary mail.tm mailbox for the Gnosis Pay MVP OTP flow.

The script creates a mailbox using the public mail.tm API and writes the
credentials to a local file. The Rust CLI can then use the printed email address,
and poll_mailtm_otp.py can read the mailbox to extract the OTP.
"""

from __future__ import annotations

import argparse
import json
import secrets
import string
import time
from pathlib import Path
from urllib import request

MAILTM_BASE_URL = "https://api.mail.tm"
DEFAULT_CREDENTIALS_PATH = Path("mailtm_credentials.txt")


def http_json(url: str, method: str = "GET", payload: dict | None = None) -> dict:
    data = None
    headers = {"Accept": "application/json"}
    if payload is not None:
        data = json.dumps(payload).encode("utf-8")
        headers["Content-Type"] = "application/json"
    req = request.Request(url, data=data, headers=headers, method=method)
    with request.urlopen(req, timeout=30) as resp:
        body = resp.read().decode("utf-8")
        return json.loads(body) if body else {}


def list_domains() -> list[str]:
    response = http_json(f"{MAILTM_BASE_URL}/domains")
    members = response.get("hydra:member", []) if isinstance(response, dict) else response
    domains = [item["domain"] for item in members if item.get("domain")]
    if not domains:
        raise RuntimeError("mail.tm did not return any available domains")
    return domains


def random_local_part(prefix: str = "gp") -> str:
    alphabet = string.ascii_lowercase + string.digits
    suffix = "".join(secrets.choice(alphabet) for _ in range(10))
    return f"{prefix}{int(time.time())}{suffix}"


def create_account(domain: str | None = None) -> tuple[str, str]:
    selected_domain = domain or list_domains()[0]
    address = f"{random_local_part()}@{selected_domain}"
    password = "GnosisPayMvp-" + secrets.token_urlsafe(18)
    http_json(
        f"{MAILTM_BASE_URL}/accounts",
        method="POST",
        payload={"address": address, "password": password},
    )
    return address, password


def main() -> int:
    parser = argparse.ArgumentParser(description="Create a temporary mail.tm mailbox for OTP testing.")
    parser.add_argument("--domain", help="Optional mail.tm domain. If omitted, the first available domain is used.")
    parser.add_argument(
        "--credentials-file",
        default=str(DEFAULT_CREDENTIALS_PATH),
        help="File where address and password will be stored, one per line.",
    )
    args = parser.parse_args()

    address, password = create_account(args.domain)
    credentials_path = Path(args.credentials_file)
    credentials_path.write_text(f"{address}\n{password}\n", encoding="utf-8")

    print(f"EMAIL={address}")
    print(f"PASSWORD={password}")
    print(f"Credentials saved to: {credentials_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
