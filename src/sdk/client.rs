use anyhow::{anyhow, Context, Result};
use reqwest::{header, Client, StatusCode};
use serde::de::DeserializeOwned;

use super::models::{
    RequestSignupOtpRequest, RequestSignupOtpResponse, SignupRequest, SignupResponse,
    SourceOfFundsQuestion, VerifySiweSignatureRequest, VerifySiweSignatureResponse,
};

#[derive(Debug, Clone)]
pub struct GnosisPayClient {
    base_url: String,
    http: Client,
}

impl GnosisPayClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        let mut default_headers = header::HeaderMap::new();
        default_headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/125 Safari/537.36",
            ),
        );
        default_headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json, text/plain, */*"),
        );
        default_headers.insert(
            header::ORIGIN,
            header::HeaderValue::from_static("https://docs.gnosispay.com"),
        );
        default_headers.insert(
            header::REFERER,
            header::HeaderValue::from_static("https://docs.gnosispay.com/"),
        );

        let http = Client::builder()
            .default_headers(default_headers)
            .cookie_store(true)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            http,
        }
    }

    pub async fn generate_nonce(&self) -> Result<String> {
        let url = self.url("/api/v1/auth/nonce");
        let response = self
            .http
            .get(url)
            .send()
            .await
            .context("Failed to send nonce request")?;
        let status = response.status();
        let body = response
            .text()
            .await
            .context("Failed to read nonce response")?;

        if !status.is_success() {
            return Err(anyhow!(
                "Gnosis Pay nonce request failed with status {status}: {body}"
            ));
        }

        Ok(body.trim().trim_matches('"').to_string())
    }

    pub async fn verify_siwe_signature(
        &self,
        request: &VerifySiweSignatureRequest,
    ) -> Result<VerifySiweSignatureResponse> {
        self.post_json("/api/v1/auth/challenge", None, request)
            .await
    }

    pub async fn request_signup_otp(&self, email: &str) -> Result<RequestSignupOtpResponse> {
        let request = RequestSignupOtpRequest {
            email: email.to_string(),
        };
        self.post_json("/api/v1/auth/signup/otp", None, &request)
            .await
    }

    pub async fn signup(
        &self,
        bearer_token: &str,
        request: &SignupRequest,
    ) -> Result<SignupResponse> {
        self.post_json("/api/v1/auth/signup", Some(bearer_token), request)
            .await
    }

    pub async fn retrieve_source_of_funds_questions(
        &self,
        bearer_token: &str,
        locale: Option<&str>,
    ) -> Result<Vec<SourceOfFundsQuestion>> {
        let mut request = self
            .http
            .get(self.url("/api/v1/source-of-funds"))
            .bearer_auth(bearer_token);

        if let Some(locale) = locale.filter(|value| !value.trim().is_empty()) {
            request = request.query(&[("locale", locale)]);
        }

        let response = request
            .send()
            .await
            .context("Failed to send Source of Funds request")?;

        Self::parse_json_response(response).await
    }

    async fn post_json<T, R>(&self, path: &str, bearer_token: Option<&str>, body: &T) -> Result<R>
    where
        T: serde::Serialize + ?Sized,
        R: DeserializeOwned,
    {
        let mut request = self.http.post(self.url(path)).json(body);

        if let Some(token) = bearer_token.filter(|value| !value.trim().is_empty()) {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .with_context(|| format!("Failed to send POST {path}"))?;
        Self::parse_json_response(response).await
    }

    async fn parse_json_response<T: DeserializeOwned>(response: reqwest::Response) -> Result<T> {
        let status = response.status();
        let body = response
            .text()
            .await
            .context("Failed to read response body")?;

        if !status.is_success() {
            return Err(anyhow!(
                "Gnosis Pay API request failed with status {}: {}",
                status,
                body
            ));
        }

        if status == StatusCode::NO_CONTENT || body.trim().is_empty() {
            return Err(anyhow!(
                "Expected a JSON response, but the response body was empty"
            ));
        }

        serde_json::from_str(&body)
            .with_context(|| format!("Failed to parse JSON response: {body}"))
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}
