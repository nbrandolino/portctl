use crate::config::Config;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::Value;

fn api_error(status: reqwest::StatusCode, url: &str, body: String) -> String {
    let detail = if !body.is_empty() {
        serde_json::from_str::<Value>(&body)
            .ok()
            .and_then(|j| j["message"].as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| body.trim().to_string())
    } else {
        String::new()
    };
    if detail.is_empty() {
        format!("HTTP {}: {}", status, url)
    } else {
        format!("HTTP {}: {} — {}", status, url, detail)
    }
}

pub struct PortainerClient {
    base_url: String,
    client: Client,
}

impl PortainerClient {
    pub fn new() -> Self {
        let cfg = Config::load();

        let base_url = cfg
            .portainer_url
            .unwrap_or_else(|| {
                eprintln!("Error: Portainer URL is not set. Run `portctl config set-url <URL>`.");
                std::process::exit(1);
            });

        let api_token = cfg
            .api_token
            .unwrap_or_else(|| {
                eprintln!("Error: API token is not set. Run `portctl config set-token <TOKEN>`.");
                std::process::exit(1);
            });

        let mut headers = HeaderMap::new();
        headers.insert("X-API-Key", HeaderValue::from_str(&api_token).expect("Invalid API token"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self { base_url, client }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/api/{}", self.base_url.trim_end_matches('/'), path.trim_start_matches('/'))
    }

    pub fn get(&self, path: &str) -> Result<Value, String> {
        let url = self.url(path);
        let response = self.client.get(&url).send().map_err(|e| format!("Request failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(api_error(status, &url, body));
        }

        response.json::<Value>().map_err(|e| format!("Failed to parse response: {e}"))
    }

    pub fn post(&self, path: &str, body: Value) -> Result<Value, String> {
        let url = self.url(path);
        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| format!("Request failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(api_error(status, &url, body));
        }

        response.json::<Value>().map_err(|e| format!("Failed to parse response: {e}"))
    }

    pub fn post_empty(&self, path: &str) -> Result<(), String> {
        let url = self.url(path);
        let response = self.client
            .post(&url)
            .send()
            .map_err(|e| format!("Request failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(api_error(status, &url, body));
        }

        Ok(())
    }

    pub fn put(&self, path: &str, body: Value) -> Result<Value, String> {
        let url = self.url(path);
        let response = self.client
            .put(&url)
            .json(&body)
            .send()
            .map_err(|e| format!("Request failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(api_error(status, &url, body));
        }

        response.json::<Value>().map_err(|e| format!("Failed to parse response: {e}"))
    }

    pub fn post_response(&self, path: &str, body: Value) -> Result<reqwest::blocking::Response, String> {
        let url = self.url(path);
        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| format!("Request failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(api_error(status, &url, body));
        }

        Ok(response)
    }

    pub fn get_response(&self, path: &str) -> Result<reqwest::blocking::Response, String> {
        let url = self.url(path);
        let response = self.client.get(&url).send().map_err(|e| format!("Request failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(api_error(status, &url, body));
        }

        Ok(response)
    }

    pub fn get_bytes(&self, path: &str) -> Result<Vec<u8>, String> {
        let url = self.url(path);
        let response = self.client.get(&url).send().map_err(|e| format!("Request failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(api_error(status, &url, body));
        }

        response.bytes().map(|b| b.to_vec()).map_err(|e| format!("Failed to read response: {e}"))
    }

    pub fn put_raw(&self, path: &str, data: Vec<u8>) -> Result<(), String> {
        let url = self.url(path);
        let response = self.client
            .put(&url)
            .header(CONTENT_TYPE, "application/x-tar")
            .body(data)
            .send()
            .map_err(|e| format!("Request failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(api_error(status, &url, body));
        }

        Ok(())
    }

    pub fn delete(&self, path: &str) -> Result<(), String> {
        let url = self.url(path);
        let response = self.client
            .delete(&url)
            .send()
            .map_err(|e| format!("Request failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(api_error(status, &url, body));
        }

        Ok(())
    }
}
