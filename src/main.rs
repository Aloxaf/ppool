#![feature(vec_remove_item, proc_macro_hygiene, decl_macro)]
#![feature(never_type)]

#[macro_use]
extern crate structopt;

mod app;
mod checker_thread;
mod config;
mod options;
mod proxy_pool;
mod server;
mod spider;
mod spider_thread;

fn main() {
    env_logger::init();

    if let Err(e) = app::run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
