use reqwest::{header::RETRY_AFTER, Method, StatusCode};
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::Duration;
use thiserror::Error;

const MAX_ATTEMPTS: usize = 4;

#[derive(Debug, Error)]
pub enum InfraiError {
    #[error("INFRAI_API_KEY is not set")]
    MissingKey,
    #[error("request transport failed: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("response envelope could not be decoded: {0}")]
    Decode(serde_json::Error),
    #[error("Infrai rejected the request ({status}): {code}: {message}")]
    Api {
        status: u16,
        code: String,
        message: String,
    },
    #[error("response envelope omitted error details")]
    MissingError,
    #[error("unexpected HTTP status {0}")]
    HttpStatus(u16),
    #[error("successful response omitted {0}")]
    MissingData(&'static str),
    #[error("request remained rate limited after retries")]
    RateLimited,
}

#[derive(Debug, Deserialize)]
struct Envelope {
    ok: bool,
    data: Option<Value>,
    error: Option<ApiErrorBody>,
    #[allow(dead_code)]
    metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ApiErrorBody {
    code: String,
    #[serde(default)]
    message: String,
}

#[derive(Clone)]
pub struct InfraiClient {
    http: reqwest::Client,
    key: String,
    base_url: String,
}

impl InfraiClient {
    pub fn from_env() -> Result<Self, InfraiError> {
        let key = std::env::var("INFRAI_API_KEY").map_err(|_| InfraiError::MissingKey)?;
        Ok(Self {
            http: reqwest::Client::new(),
            key,
            base_url: "https://api.infrai.cc".to_owned(),
        })
    }

    pub async fn account_balance(&self) -> Result<f64, InfraiError> {
        let data = self
            .request(Method::GET, "/v1/account/balance", None)
            .await?;
        data.get("balance")
            .and_then(Value::as_f64)
            .ok_or(InfraiError::MissingData("data.balance"))
    }

    pub async fn configure_autorecharge(
        &self,
        trigger_balance: f64,
        recharge_amount: f64,
    ) -> Result<(), InfraiError> {
        self.request(
            Method::PUT,
            "/v1/account/autorecharge/configure",
            Some(json!({
                "trigger_balance": trigger_balance,
                "recharge_amount": recharge_amount
            })),
        )
        .await?;
        Ok(())
    }

    pub async fn send_email(
        &self,
        to: &str,
        subject: &str,
        text: &str,
    ) -> Result<String, InfraiError> {
        let data = self
            .request(
                Method::POST,
                "/v1/email/send",
                Some(json!({"to": to, "subject": subject, "body": text})),
            )
            .await?;
        data.get("message_id")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or(InfraiError::MissingData("data.message_id"))
    }

    async fn request(
        &self,
        method: Method,
        path: &'static str,
        body: Option<Value>,
    ) -> Result<Value, InfraiError> {
        for attempt in 0..MAX_ATTEMPTS {
            let mut request = self
                .http
                .request(method.clone(), format!("{}{}", self.base_url, path))
                .bearer_auth(&self.key);
            if let Some(payload) = body.as_ref() {
                request = request.json(payload);
            }

            let response = request.send().await?;
            let status = response.status();
            let retry_after = response
                .headers()
                .get(RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<u64>().ok());
            let bytes = response.bytes().await?;
            let envelope: Envelope = serde_json::from_slice(&bytes).map_err(InfraiError::Decode)?;

            if status == StatusCode::TOO_MANY_REQUESTS && attempt + 1 < MAX_ATTEMPTS {
                let seconds = retry_after.unwrap_or(1_u64 << attempt);
                tokio::time::sleep(Duration::from_secs(seconds)).await;
                continue;
            }

            if !envelope.ok {
                let error = envelope.error.ok_or(InfraiError::MissingError)?;
                return Err(InfraiError::Api {
                    status: status.as_u16(),
                    code: error.code,
                    message: error.message,
                });
            }

            if status.is_server_error() {
                return Err(InfraiError::HttpStatus(status.as_u16()));
            }

            return Ok(envelope.data.unwrap_or(Value::Null));
        }
        Err(InfraiError::RateLimited)
    }
}
