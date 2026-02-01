use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A record of a request for logging purposes.
#[derive(Debug, Clone)]
pub struct RequestLogRecord {
    pub request_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub endpoint: String,
    pub original_url: String,
    pub manifest_headers: Option<String>,
    pub segment_headers: Option<String>,
    pub key_provided: bool,
    pub key_type: Option<String>,
    pub decrypt_enabled: bool,
    pub response_status: i32,
    pub response_time_ms: i64,
    pub content_length: Option<i64>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
}

impl RequestLogRecord {
    pub fn new(endpoint: &str, url: &str) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            endpoint: endpoint.to_string(),
            original_url: url.to_string(),
            manifest_headers: None,
            segment_headers: None,
            key_provided: false,
            key_type: None,
            decrypt_enabled: false,
            response_status: 0,
            response_time_ms: 0,
            content_length: None,
            error_type: None,
            error_message: None,
            client_ip: None,
            user_agent: None,
        }
    }

    pub fn with_headers(mut self, manifest: Option<&str>, segment: Option<&str>) -> Self {
        self.manifest_headers = manifest.map(String::from);
        self.segment_headers = segment.map(String::from);
        self
    }

    pub fn with_key_info(mut self, key_provided: bool, key_type: Option<&str>) -> Self {
        self.key_provided = key_provided;
        self.key_type = key_type.map(String::from);
        self
    }

    pub fn with_decrypt(mut self, enabled: bool) -> Self {
        self.decrypt_enabled = enabled;
        self
    }

    pub fn with_response(mut self, status: u16, time_ms: i64, length: Option<u64>) -> Self {
        self.response_status = status as i32;
        self.response_time_ms = time_ms;
        self.content_length = length.map(|l| l as i64);
        self
    }

    pub fn with_error(mut self, error_type: &str, message: &str) -> Self {
        self.error_type = Some(error_type.to_string());
        self.error_message = Some(message.to_string());
        self
    }

    pub fn with_client_info(mut self, ip: Option<&str>, user_agent: Option<&str>) -> Self {
        self.client_ip = ip.map(String::from);
        self.user_agent = user_agent.map(String::from);
        self
    }
}
