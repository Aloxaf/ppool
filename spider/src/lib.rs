pub mod error;
pub mod proxy_getter;
pub mod utils;
use serde::{Deserialize, Serialize};

// TODO: anonymous 和 ssl 使用 enum
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
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

    #[inline]
    pub fn ip(&self) -> &str {
        &self.ip
    }

    #[inline]
    pub fn port(&self) -> u16 {
        self.port
    }

    #[inline]
    pub fn anonymous(&self) -> &str {
        &self.anonymous
    }

    #[inline]
    pub fn ssl(&self) -> &str {
        &self.ssl
    }

    #[inline]
    pub fn get_key(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}
