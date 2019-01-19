#![feature(vec_remove_item)]

mod checker_thread;
pub mod config;
pub mod proxy_pool;
pub mod spider;
mod spider_thread;

pub use crate::checker_thread::checker_thread;
pub use crate::config::*;
pub use crate::proxy_pool::*;
pub use crate::spider_thread::spider_thread;
