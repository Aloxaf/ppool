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
fn index(_state: State<AProxyPool>) -> &'static str {
    r#"{
  "get?<http:bool>&<https:bool>&<anonymity:str>&<stability:f32>": "随机获取一个代理, 无特殊需求请勿增加参数, 速度较慢",
  "get_all?<http:bool>&<https:bool>&<anonymity:str>&<stability:f32>": "获取所有可用代理",
  "get_status": "获取代理池信息",
}"#
}

#[get("/get_status")]
fn get_status(state: State<AProxyPool>) -> String {
    let proxies = state.lock().unwrap();
    let verified = proxies.get_verified().len();
    let unverified = proxies.get_unverified().len();
    format!(
        r#"{{
  "total": {},
  "verified": {},
  "unverified": {},
}}"#,
        verified + unverified,
        verified,
        unverified
    )
}

#[get("/get?<http>&<https>&<anonymity>&<stability>")]
fn get_single(
    state: State<AProxyPool>,
    http: Option<bool>,
    https: Option<bool>,
    anonymity: Option<String>,
    stability: Option<f32>,
) -> String {
    let mut proxies = state.lock().unwrap();
    if proxies.get_verified().len() == 0 {
        "[]".to_string()
    } else if http.is_none() && https.is_none() && anonymity.is_none() && stability.is_none() {
        let proxy = proxies.get_random();
        serde_json::to_string_pretty(proxy).unwrap()
    } else {
        let proxy = proxies.select_random(http, https, anonymity, stability);
        serde_json::to_string_pretty(proxy).unwrap()
    }
}

#[get("/get_all?<http>&<https>&<anonymity>&<stability>")]
fn get_all(
    state: State<AProxyPool>,
    http: Option<bool>,
    https: Option<bool>,
    anonymity: Option<String>,
    stability: Option<f32>,
) -> String {
    let proxies = state.lock().unwrap();
    if http.is_none() && https.is_none() && anonymity.is_none() && stability.is_none() {
        let proxy = proxies.get_verified();
        serde_json::to_string_pretty(proxy).unwrap()
    } else {
        let proxy = proxies.select(http, https, anonymity, stability);
        serde_json::to_string_pretty(&proxy).unwrap()
    }
}

fn main() {
    env_logger::init();

    let mut data_path =
        app_dir(AppDataType::UserData, &APP_INFO, "proxy_list").expect("cannot create appdir");
    data_path.push("proxies.json");

    debug!("data_path: {:?}", &data_path);

    let proxies = {
        if let Ok(file) = File::open(&data_path) {
            Arc::new(Mutex::new(
                serde_json::from_reader(file).expect("无法读取配置文件"),
            ))
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
        .mount("/", routes![index, get_status, get_single, get_all])
        .manage(proxies)
        .launch();
}
