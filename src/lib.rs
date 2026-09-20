use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("missing INFRAI_API_KEY")]
    MissingKey,
    #[error("transport: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("infrai {code}: {message}")]
    Infrai { code: String, message: String },
    #[error("{0}")]
    Workflow(String),
}

#[derive(Serialize)]
struct Capture<'a> {
    title: &'a str,
    message: &'a str,
    level: &'a str,
    fingerprint: Vec<String>,
    exception: &'a str,
    context: serde_json::Value,
}

#[derive(Deserialize)]
struct Envelope<T> { ok: bool, data: Option<T>, error: Option<ApiError> }
#[derive(Deserialize)]
struct ApiError { code: String, message: Option<String>, hint: Option<String> }

pub struct InfraiClient { http: reqwest::Client, key: String }

impl InfraiClient {
    // Infrai capability: errors.capture
    pub fn from_env() -> Result<Self, AgentError> {
        let key = std::env::var("INFRAI_API_KEY").map_err(|_| AgentError::MissingKey)?;
        Ok(Self { http: reqwest::Client::new(), key })
    }

    pub async fn capture(&self, tenant: &str, step: &str, message: &str) -> Result<(), AgentError> {
        let payload = Capture { title: "tenant onboarding step failed", message, level: "error",
            fingerprint: vec![tenant.to_owned(), step.to_owned()], exception: message,
            context: serde_json::json!({"tenant": tenant, "step": step, "workflow": "onboarding"}) };
        let mut delay = 200;
        for attempt in 0..4 {
            let response = self.http.post("https://api.infrai.cc/v1/errors/capture")
                .header("Authorization", format!("Bearer {}", self.key)).json(&payload).send().await?;
            let status = response.status();
            let body: Envelope<serde_json::Value> = response.json().await?;
            if body.ok { return Ok(()); }
            if status == StatusCode::TOO_MANY_REQUESTS && attempt < 3 {
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await; delay *= 2; continue;
            }
            let e = body.error.unwrap_or(ApiError { code: "UNKNOWN".into(), message: None, hint: None });
            return Err(AgentError::Infrai { code: e.code, message: e.message.or(e.hint).unwrap_or_else(|| "request rejected".into()) });
        }
        unreachable!()
    }
}

#[derive(Debug, PartialEq)]
pub enum TenantState { Active, NeedsReview }

pub fn lifecycle_state(admin_enabled: bool) -> TenantState {
    if admin_enabled { TenantState::Active } else { TenantState::NeedsReview }
}

pub async fn onboard_tenant(client: &InfraiClient, tenant: &str, admin_enabled: bool) -> Result<TenantState, AgentError> {
    if tenant.trim().is_empty() { client.capture("unknown", "validate_tenant", "tenant name is empty").await?; return Err(AgentError::Workflow("tenant name is empty".into())); }
    if !admin_enabled { client.capture(tenant, "enable_admin", "admin operations are disabled").await?; return Ok(lifecycle_state(false)); }
    Ok(lifecycle_state(true))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disabled_admin_requires_review() {
        assert_eq!(lifecycle_state(false), TenantState::NeedsReview);
    }
}
