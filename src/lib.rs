#![feature(vec_remove_item)]

pub mod checker_thread;
mod config;
pub mod proxy_pool;
pub mod spider;
pub mod spider_thread;

pub use crate::config::{Config, DEFAULT_CONFIG};
pub use crate::proxy_pool::*;
