use failure::Error;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

pub type SpiderResult<T> = Result<T, Error>;

/// 匿名程度
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum AnonymityLevel {
    /// 透明
    Transparent,
    /// 匿名
    Anonymous,
    /// 高匿
    Elite,
}

impl<T: Sized + AsRef<str>> From<T> for AnonymityLevel {
    fn from(s: T) -> Self {
        let s = s.as_ref();
        if s.contains("高") {
            AnonymityLevel::Elite
        } else if s.contains("普") {
            AnonymityLevel::Anonymous
        } else {
            // 默认透明
            AnonymityLevel::Transparent
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum SslType {
    HTTP,
    HTTPS,
}

impl<T: Sized + AsRef<str>> From<T> for SslType {
    fn from(s: T) -> Self {
        let s = s.as_ref();
        if s.contains("HTTPS") || s.contains("https") {
            SslType::HTTPS
        } else {
            // 默认 HTTP
            SslType::HTTP
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Proxy {
    ip: Ipv4Addr,
    port: u16,
    anonymity: AnonymityLevel,
    ssl_type: SslType,
}

impl Proxy {
    pub fn new(ip: &str, port: &str, anonymity: &str, ssl_type: &str) -> Self {
        Self {
            ip: ip.parse().expect("failed to parse IP"),
            port: port.parse().expect("failed to parse port"),
            anonymity: AnonymityLevel::from(anonymity),
            ssl_type: SslType::from(ssl_type),
        }
    }

    #[inline]
    pub fn ip(&self) -> Ipv4Addr {
        self.ip
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
    pub fn ssl_type(&self) -> SslType {
        self.ssl_type
    }

    #[inline]
    pub fn get_key(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}
