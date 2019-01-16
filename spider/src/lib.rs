pub mod error;
pub mod proxy_getter;
pub mod utils;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum AnonymityLevel {
    Transparent,
    Anonymous,
    Elite,
}

impl<T: Sized + AsRef<str>> From<T> for AnonymityLevel {
    fn from(s: T) -> Self {
        match s.as_ref() {
            "高匿" => AnonymityLevel::Elite,
            "匿名" => AnonymityLevel::Anonymous,
            // 默认透明
            _ => AnonymityLevel::Transparent,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Proxy {
    ip: String,
    port: u16,
    anonymity: AnonymityLevel,
    http: bool,
    https: bool,
}

impl Proxy {
    pub fn new(ip: &str, port: u16, anonymity: AnonymityLevel, http: bool, https: bool) -> Self {
        Self {
            ip: ip.to_owned(),
            port,
            anonymity,
            http,
            https,
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
    pub fn anonymity(&self) -> AnonymityLevel {
        self.anonymity
    }

    #[inline]
    pub fn http(&self) -> bool {
        self.http
    }

    #[inline]
    pub fn https(&self) -> bool {
        self.https
    }

    #[inline]
    pub fn get_key(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}
