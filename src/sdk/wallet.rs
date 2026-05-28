use anyhow::{Context, Result};
use chrono::{SecondsFormat, Utc};
use ethers_core::rand::thread_rng;
use ethers_core::utils::to_checksum;
use ethers_signers::{LocalWallet, Signer};

#[derive(Debug, Clone)]
pub struct GeneratedWallet {
    pub wallet: LocalWallet,
    pub address_checksum: String,
    pub private_key_hex: String,
}

pub fn create_evm_wallet() -> GeneratedWallet {
    let wallet = LocalWallet::new(&mut thread_rng());
    let address = wallet.address();
    let private_key_bytes = wallet.signer().to_bytes();

    GeneratedWallet {
        wallet,
        address_checksum: to_checksum(&address, None),
        private_key_hex: format!("0x{}", hex::encode(private_key_bytes)),
    }
}

pub fn build_siwe_message(
    domain: &str,
    address: &str,
    uri: &str,
    chain_id: u64,
    nonce: &str,
    statement: &str,
) -> String {
    let issued_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    format!(
        "{domain} wants you to sign in with your Ethereum account:\n{address}\n\n{statement}\n\nURI: {uri}\nVersion: 1\nChain ID: {chain_id}\nNonce: {nonce}\nIssued At: {issued_at}"
    )
}

pub async fn sign_message(wallet: &LocalWallet, message: &str) -> Result<String> {
    wallet
        .sign_message(message)
        .await
        .context("Failed to sign SIWE message")
        .map(|signature| format!("0x{}", signature))
}
