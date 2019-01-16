#![feature(proc_macro_hygiene, decl_macro)]

use app_dirs::*;
use log::{debug, info};
use ppool_server::{checker::checker_thread, spider::spider_thread, AProxyPool, ProxyPool};
use rocket::{get, routes, State};
use std::fs::File;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;

const APP_INFO: AppInfo = AppInfo {
    name: "ppool",
    author: "Aloxaf",
};

#[get("/")]
fn index(state: State<AProxyPool>) -> String {
    let proxies = state.lock().unwrap();
    let verified = proxies.get_verified().len();
    let unverified = proxies.get_unverified().len();
    format!(
        "{{\n  \"total\": {},\n  \"verified\": {},\n  \"unverified\": {}\n}}",
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
        serde_json::to_string_pretty(proxy).unwrap()
    }
}

#[get("/get_all")]
fn get_all(state: State<AProxyPool>) -> String {
    let proxies = state.lock().unwrap();
    let proxy = proxies.get_verified();
    serde_json::to_string_pretty(proxy).unwrap()
}

fn main() {
    env_logger::init();

    let mut data_path =
        app_dir(AppDataType::UserData, &APP_INFO, "proxy_list").expect("cannot create appdir");
    data_path.push("proxies.json");

    debug!("data_path: {:?}", &data_path);

    let proxies = {
        if let Ok(file) = File::open(&data_path) {
            Arc::new(Mutex::new(serde_json::from_reader(file).unwrap()))
        } else {
            Arc::new(Mutex::new(ProxyPool::new()))
        }
    };

    {
        let data_path = data_path.clone();
        let proxies = proxies.clone();
        thread::spawn(move || loop {
            spider_thread(proxies.clone());
            checker_thread(proxies.clone());
            info!("writing to disk");
            let data = serde_json::to_string_pretty(&proxies).unwrap();
            let mut file = File::create(&data_path).unwrap();
            file.write(data.as_bytes()).unwrap();
            info!("sleeping for 10 mins...");
            sleep(Duration::from_secs(60 * 10));
        });
    }

    rocket::ignite()
        .mount("/", routes![index, get_single, get_all])
        .manage(proxies)
        .launch();
}
