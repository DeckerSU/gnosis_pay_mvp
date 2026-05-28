mod sdk;

use anyhow::{Context, Result};
use sdk::{
    build_siwe_message, create_evm_wallet, sign_message, GnosisPayClient, SignupRequest,
    VerifySiweSignatureRequest,
};
use std::io::{self, Write};

const DEFAULT_BASE_URL: &str = "https://api.gnosispay.com";
const DEFAULT_SIWE_DOMAIN: &str = "app.gnosispay.com";
const DEFAULT_SIWE_URI: &str = "https://app.gnosispay.com";
const DEFAULT_CHAIN_ID: u64 = 100;
const DEFAULT_TTL_SECONDS: u64 = 3600;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Gnosis Pay MVP Authentication demo");
    println!("===================================\n");

    let base_url = prompt_with_default("Gnosis Pay API base URL", DEFAULT_BASE_URL)?;
    let siwe_domain = prompt_with_default("SIWE domain", DEFAULT_SIWE_DOMAIN)?;
    let siwe_uri = prompt_with_default("SIWE URI", DEFAULT_SIWE_URI)?;
    let chain_id = prompt_u64_with_default("SIWE chain ID", DEFAULT_CHAIN_ID)?;
    let ttl_seconds = prompt_u64_with_default("JWT TTL in seconds", DEFAULT_TTL_SECONDS)?;

    let client = GnosisPayClient::new(base_url);

    println!("\n1. Generating a new EVM wallet...");
    let generated_wallet = create_evm_wallet();
    println!("Address: {}", generated_wallet.address_checksum);
    println!("Private key: {}", generated_wallet.private_key_hex);
    println!("\nStore the private key securely. It is printed only because this is an MVP demo.\n");

    println!("2. Requesting a nonce from Gnosis Pay...");
    let nonce = client
        .generate_nonce()
        .await
        .context("Could not generate nonce")?;
    println!("Nonce: {nonce}\n");

    println!("3. Building and signing a SIWE message...");
    let siwe_message = build_siwe_message(
        &siwe_domain,
        &generated_wallet.address_checksum,
        &siwe_uri,
        chain_id,
        &nonce,
        "Sign in to the Gnosis Pay API MVP CLI application.",
    );
    println!("\nSIWE message:\n---\n{siwe_message}\n---\n");

    let signature = sign_message(&generated_wallet.wallet, &siwe_message)
        .await
        .context("Could not sign SIWE message")?;
    println!("Signature: {signature}\n");

    println!("4. Verifying SIWE signature and receiving an authentication token...");
    let challenge_response = client
        .verify_siwe_signature(&VerifySiweSignatureRequest {
            message: siwe_message,
            signature,
            ttl_in_seconds: ttl_seconds,
        })
        .await
        .context("Could not verify SIWE signature")?;
    println!("Received pre-signup JWT token.\n");

    println!("5. Requesting email verification OTP...");
    let email = prompt_required("Email for the new user")?;
    let otp_response = client
        .request_signup_otp(&email)
        .await
        .context("Could not request email OTP")?;
    println!("OTP request accepted: {}", otp_response.ok);
    println!("Check the email inbox, then paste the 6-digit OTP below.");

    let otp = prompt_required("OTP")?;
    let marketing_campaign = optional_prompt("Marketing campaign (optional)")?;
    let partner_id = optional_prompt("Partner ID (optional)")?;

    println!("\n6. Creating a new Gnosis Pay user...");
    let signup_response = client
        .signup(
            &challenge_response.token,
            &SignupRequest {
                auth_email: email,
                otp: Some(otp),
                marketing_campaign,
                partner_id,
            },
        )
        .await
        .context("Could not create a new Gnosis Pay user")?;

    println!("User ID: {}", signup_response.id);
    println!("hasSignedUp: {}", signup_response.has_signed_up);
    println!("Received final JWT token for subsequent requests.\n");

    println!("7. Retrieving Source of Funds questions as a demo authenticated request...");
    let locale = optional_prompt("Locale for Source of Funds questions, e.g. UK or BR (optional)")?;
    let questions = client
        .retrieve_source_of_funds_questions(&signup_response.token, locale.as_deref())
        .await
        .context("Could not retrieve Source of Funds questions")?;

    println!("\nSource of Funds questions:");
    println!("{}", serde_json::to_string_pretty(&questions)?);

    println!("\nDone. The final JWT token is printed below for demo purposes:");
    println!("{}", signup_response.token);

    Ok(())
}

fn prompt_required(label: &str) -> Result<String> {
    loop {
        let value = prompt(label)?;
        if !value.trim().is_empty() {
            return Ok(value.trim().to_string());
        }
        println!("The value is required. Please try again.");
    }
}

fn prompt_with_default(label: &str, default: &str) -> Result<String> {
    let value = prompt(&format!("{label} [{default}]"))?;
    if value.trim().is_empty() {
        Ok(default.to_string())
    } else {
        Ok(value.trim().to_string())
    }
}

fn prompt_u64_with_default(label: &str, default: u64) -> Result<u64> {
    loop {
        let value = prompt(&format!("{label} [{default}]"))?;
        if value.trim().is_empty() {
            return Ok(default);
        }
        match value.trim().parse::<u64>() {
            Ok(parsed) => return Ok(parsed),
            Err(_) => println!("Please enter a valid positive integer."),
        }
    }
}

fn optional_prompt(label: &str) -> Result<Option<String>> {
    let value = prompt(label)?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

fn prompt(label: &str) -> Result<String> {
    print!("{label}: ");
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read user input")?;

    Ok(input.trim_end_matches(['\r', '\n']).to_string())
}
