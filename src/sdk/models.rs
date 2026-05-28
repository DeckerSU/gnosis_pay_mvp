use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct VerifySiweSignatureRequest {
    pub message: String,
    pub signature: String,
    #[serde(rename = "ttlInSeconds")]
    pub ttl_in_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VerifySiweSignatureResponse {
    pub token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestSignupOtpRequest {
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequestSignupOtpResponse {
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignupRequest {
    #[serde(rename = "authEmail")]
    pub auth_email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otp: Option<String>,
    #[serde(rename = "marketingCampaign", skip_serializing_if = "Option::is_none")]
    pub marketing_campaign: Option<String>,
    #[serde(rename = "partnerId", skip_serializing_if = "Option::is_none")]
    pub partner_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SignupResponse {
    pub id: String,
    pub token: String,
    #[serde(rename = "hasSignedUp")]
    pub has_signed_up: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceOfFundsQuestion {
    pub question: String,
    pub answers: Vec<String>,
}
