use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::config::Config;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error ({status}): {body}")]
    Api { status: u16, body: String },
    #[error("Not authenticated. Run `umami-cli auth login` first.")]
    NotAuthenticated,
    #[error("{0}")]
    Other(String),
}

pub type ApiResult<T> = Result<T, ApiError>;

pub struct UmamiClient {
    http: reqwest::Client,
    base_url: String,
    token: Option<String>,
}

impl UmamiClient {
    pub fn new(base_url: &str, token: Option<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
        }
    }

    pub fn from_config(config: &Config) -> ApiResult<Self> {
        let base_url = config
            .server_url
            .as_deref()
            .ok_or_else(|| ApiError::Other("No server URL configured. Run `umami-cli auth login` first.".into()))?
            .trim_end_matches('/')
            .to_string();

        Ok(Self {
            http: reqwest::Client::new(),
            base_url,
            token: config.token.clone(),
        })
    }

    fn headers(&self) -> ApiResult<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(ref token) = self.token {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {token}"))
                    .map_err(|e| ApiError::Other(e.to_string()))?,
            );
        }
        Ok(headers)
    }

    fn require_auth(&self) -> ApiResult<()> {
        if self.token.is_none() {
            return Err(ApiError::NotAuthenticated);
        }
        Ok(())
    }

    pub async fn login(&mut self, username: &str, password: &str) -> ApiResult<Value> {
        let body = serde_json::json!({ "username": username, "password": password });
        let resp = self
            .http
            .post(format!("{}/api/auth/login", self.base_url))
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status().as_u16();
        if status >= 400 {
            let text = resp.text().await.unwrap_or_default();
            return Err(ApiError::Api { status, body: text });
        }

        let data: Value = resp.json().await?;
        if let Some(token) = data.get("token").and_then(|t| t.as_str()) {
            self.token = Some(token.to_string());
        }
        Ok(data)
    }

    pub async fn verify(&self) -> ApiResult<Value> {
        self.require_auth()?;
        self.post("/api/auth/verify", &serde_json::json!({})).await
    }

    // Generic HTTP methods

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> ApiResult<T> {
        self.require_auth()?;
        let resp = self
            .http
            .get(format!("{}{}", self.base_url, path))
            .headers(self.headers()?)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    pub async fn get_with_query<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> ApiResult<T> {
        self.require_auth()?;
        let resp = self
            .http
            .get(format!("{}{}", self.base_url, path))
            .headers(self.headers()?)
            .query(query)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    pub async fn post<T: DeserializeOwned>(&self, path: &str, body: &Value) -> ApiResult<T> {
        self.require_auth()?;
        let resp = self
            .http
            .post(format!("{}{}", self.base_url, path))
            .headers(self.headers()?)
            .json(body)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> ApiResult<T> {
        self.require_auth()?;
        let resp = self
            .http
            .delete(format!("{}{}", self.base_url, path))
            .headers(self.headers()?)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    async fn handle_response<T: DeserializeOwned>(resp: reqwest::Response) -> ApiResult<T> {
        let status = resp.status().as_u16();
        if status >= 400 {
            let text = resp.text().await.unwrap_or_default();
            return Err(ApiError::Api { status, body: text });
        }
        Ok(resp.json().await?)
    }
}
