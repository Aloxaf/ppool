pub mod error;
pub mod proxy_getter;
pub mod utils;
use serde::Serialize;

// TODO: anonymous 和 ssl 使用 enum
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct Proxy {
    ip: String,
    port: u16,
    anonymous: String,
    ssl: String,
}

impl Proxy {
    pub fn new(ip: &str, port: u16, anonymous: &str, ssl: &str) -> Self {
        Self {
            ip: ip.to_owned(),
            port,
            anonymous: anonymous.to_owned(),
            ssl: ssl.to_owned(),
        }
    }

    pub fn ip(&self) -> &str {
        &self.ip
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn anonymous(&self) -> &str {
        &self.anonymous
    }

    pub fn ssl(&self) -> &str {
        &self.ssl
    }
}
