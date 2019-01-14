pub mod error;
pub mod proxy_getter;
pub mod utils;

#[derive(Debug, Clone)]
pub struct Proxy {
    ip: String,
    port: u16,
    anonymous: String,
    ssl: String,
}
