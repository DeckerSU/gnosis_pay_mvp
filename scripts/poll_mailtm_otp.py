#!/usr/bin/env python3
"""Poll a mail.tm mailbox and print the first 6-digit OTP found.

Use this helper while the Rust CLI is waiting at the `OTP:` prompt. The script
reads the mailbox credentials created by create_mailtm_account.py and polls
mail.tm until it finds a 6-digit verification code.
"""

from __future__ import annotations

import argparse
import json
import re
import time
from pathlib import Path
from urllib import request

MAILTM_BASE_URL = "https://api.mail.tm"
DEFAULT_CREDENTIALS_PATH = Path("mailtm_credentials.txt")


def http_json(url: str, method: str = "GET", token: str | None = None, payload: dict | None = None) -> dict | list:
    data = None
    headers = {"Accept": "application/json"}
    if payload is not None:
        data = json.dumps(payload).encode("utf-8")
        headers["Content-Type"] = "application/json"
    if token:
        headers["Authorization"] = f"Bearer {token}"
    req = request.Request(url, data=data, headers=headers, method=method)
    with request.urlopen(req, timeout=30) as resp:
        body = resp.read().decode("utf-8")
        return json.loads(body) if body else {}


def read_credentials(path: Path) -> tuple[str, str]:
    lines = [line.strip() for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
    if len(lines) < 2:
        raise ValueError(f"Expected address and password in {path}, one per line")
    return lines[0], lines[1]


def get_token(address: str, password: str) -> str:
    response = http_json(
        f"{MAILTM_BASE_URL}/token",
        method="POST",
        payload={"address": address, "password": password},
    )
    if not isinstance(response, dict) or "token" not in response:
        raise RuntimeError(f"Could not authenticate to mail.tm for {address}")
    return response["token"]


def extract_otp(text: str) -> str | None:
    matches = re.findall(r"(?<!\d)(\d{6})(?!\d)", text)
    return matches[0] if matches else None


def message_list(response: dict | list) -> list[dict]:
    if isinstance(response, dict):
        return response.get("hydra:member", [])
    return response


def poll_otp(token: str, timeout_seconds: int, interval_seconds: int) -> str:
    deadline = time.time() + timeout_seconds
    last_seen = ""
    while time.time() < deadline:
        response = http_json(f"{MAILTM_BASE_URL}/messages", token=token)
        for msg in message_list(response):
            msg_id = msg["id"]
            detail = http_json(f"{MAILTM_BASE_URL}/messages/{msg_id}", token=token)
            if not isinstance(detail, dict):
                continue
            combined = "\n".join(str(detail.get(key, "")) for key in ("subject", "intro", "text", "html"))
            last_seen = combined
            otp = extract_otp(combined)
            if otp:
                return otp
        time.sleep(interval_seconds)
    raise TimeoutError(f"No 6-digit OTP found within {timeout_seconds}s. Last message preview: {last_seen[:500]}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Poll a mail.tm mailbox and print the first 6-digit OTP found.")
    parser.add_argument(
        "--credentials-file",
        default=str(DEFAULT_CREDENTIALS_PATH),
        help="Credentials file produced by create_mailtm_account.py.",
    )
    parser.add_argument("--timeout", type=int, default=180, help="Maximum wait time in seconds.")
    parser.add_argument("--interval", type=int, default=5, help="Polling interval in seconds.")
    args = parser.parse_args()

    address, password = read_credentials(Path(args.credentials_file))
    token = get_token(address, password)
    otp = poll_otp(token, args.timeout, args.interval)
    print(otp)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
