#![feature(proc_macro_hygiene, decl_macro)]

use app_dirs::*;
use clap::{load_yaml, App};
use failure::{format_err, Error};
use log::{debug, info};
use ppool::checker_thread::checker_thread;
use ppool::proxy_pool::*;
use ppool::spider_thread::spider_thread;
use ppool::{Config, DEFAULT_CONFIG};
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
  "get?<ssl_type:str>&<anonymity:str>&<stability:f32>": "随机获取一个代理, 无特殊需求请勿增加参数, 速度较慢",
  "get_all?<ssl_type:str>&<anonymity:str>&<stability:f32>": "获取所有可用代理",
  "get_status": "获取代理池信息",
}"#
}

#[get("/get_status")]
fn get_status(state: State<AProxyPool>) -> String {
    let proxies = state.lock().unwrap();
    let stable_cnt = proxies.get_stable().len();
    let unstable_cnt = proxies.get_unstable().len();
    format!(
        r#"{{
  "total": {},
  "stable": {},
  "unstable": {},
}}"#,
        stable_cnt + unstable_cnt,
        stable_cnt,
        unstable_cnt
    )
}

// TODO: 提前搞个类型转换
#[get("/get?<ssl_type>&<anonymity>&<stability>")]
fn get_single(
    state: State<AProxyPool>,
    ssl_type: Option<String>,
    anonymity: Option<String>,
    stability: Option<f32>,
) -> String {
    let mut proxies = state.lock().unwrap();
    if proxies.get_stable().is_empty() {
        "[]".to_string()
    } else if ssl_type.is_none() && anonymity.is_none() && stability.is_none() {
        let proxy = proxies.get_random();
        serde_json::to_string_pretty(proxy).unwrap()
    } else {
        let proxy = proxies.select_random(ssl_type, anonymity, stability);
        if let Some(proxy) = proxy {
            serde_json::to_string_pretty(proxy).unwrap()
        } else {
            "null".to_owned()
        }
    }
}

#[get("/get_all?<ssl_type>&<anonymity>&<stability>")]
fn get_all(
    state: State<AProxyPool>,
    ssl_type: Option<String>,
    anonymity: Option<String>,
    stability: Option<f32>,
) -> String {
    let proxies = state.lock().unwrap();
    if ssl_type.is_none() && anonymity.is_none() && stability.is_none() {
        let proxy = proxies.get_stable();
        serde_json::to_string_pretty(proxy).unwrap()
    } else {
        let proxy = proxies.select(ssl_type, anonymity, stability);
        serde_json::to_string_pretty(&proxy).unwrap()
    }
}

// TODO: del api
// 其实并不想增加这个 API, 感觉没啥用...还增加复杂度

// TODO: reload api
// 修改配置文件后不用 kill 进程

fn run() -> Result<(), Error> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if matches.occurrences_of("print_config") != 0 {
        println!("{}", DEFAULT_CONFIG);
        return Ok(());
    }

    // 读取配置
    debug!("正在读取配置");
    let Config {
        checker: checker_config,
        spider: spider_config,
    }: Config = if let Some(config_file) = matches.value_of("config") {
        let mut file = std::fs::File::open(config_file)
            .map_err(|e| format_err!("无法读取配置文件: {:#?}", e))?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        toml::from_str(&s).map_err(|e| format_err!("配置文件解析失败: {:#?}", e))?
    } else {
        toml::from_str(DEFAULT_CONFIG).unwrap()
    };

    // 读取(可能的)上次的数据
    debug!("正在读取缓存");
    let mut data_path = app_dir(AppDataType::UserData, &APP_INFO, "proxy_list")
        .expect("无法创建 UserData 目录");
    data_path.push("proxies.json");
    debug!("data_path: {:?}", &data_path);

    let save_data = if let Ok(file) = File::open(&data_path) {
        serde_json::from_reader(file).unwrap()
    } else {
        ProxyPool::new()
    };

    let proxies = Arc::new(Mutex::new(save_data));

    {
        let proxies = proxies.clone();
        thread::spawn(move || loop {
            spider_thread(proxies.clone(), &spider_config);
            info!("等待20分钟再次爬取...");
            sleep(Duration::from_secs(spider_config.interval));
        });
    }

    {
        let data_path = data_path.clone();
        let proxies = proxies.clone();
        thread::spawn(move || loop {
            info!("等待1分钟开始验证...");
            sleep(Duration::from_secs(checker_config.interval));
            checker_thread(proxies.clone(), &checker_config);
            // TODO: 这个"备份"也单独开一个线程?
            info!("写入到磁盘");
            let data = serde_json::to_string_pretty(&proxies).expect("无法序列化");
            let mut file = File::create(&data_path).expect("无法创建文件");
            file.write_all(data.as_bytes()).expect("无法写入");
        });
    }

    // 火箭, 发射! (误
    rocket::ignite()
        .mount("/", routes![index, get_status, get_single, get_all])
        .manage(proxies)
        .launch();
    Ok(())
}

fn main() {
    env_logger::init();

    if let Err(e) = run() {
        eprintln!("{:#?}", e);
        std::process::exit(1);
    }
}
