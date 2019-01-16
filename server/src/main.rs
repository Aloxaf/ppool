#![feature(proc_macro_hygiene, decl_macro)]

use log::info;
use ppool_server::{checker::checker_thread, spider::spider_thread, AProxyPool, ProxyPool};
use rocket::{get, routes, State};
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;

#[get("/")]
fn index(state: State<AProxyPool>) -> String {
    let proxies = state.lock().unwrap();
    let verified = proxies.get_verified().len();
    let unverified = proxies.get_unverified().len();
    format!(
        r#"{{"total": {}, "verified": {}, "unverified": {}}}"#,
        verified + unverified,
        verified,
        unverified
    )
}

#[get("/get")]
fn get_single(state: State<AProxyPool>) -> String {
    let mut proxies = state.lock().unwrap();
    if proxies.get_verified().len() == 0 {
        "[]".to_string()
    } else {
        let proxy = proxies.get_random();
        serde_json::to_string(proxy).unwrap()
    }
}

fn main() {
    env_logger::init();

    let proxies = Arc::new(Mutex::new(ProxyPool::new()));

    let tmp = proxies.clone();
    thread::spawn(move || loop {
        spider_thread(tmp.clone());
        checker_thread(tmp.clone());
        info!("sleeping for 10 mins...");
        sleep(Duration::from_secs(60 * 10));
    });

    rocket::ignite()
        .mount("/", routes![index, get_single])
        .manage(proxies)
        .launch();
}
