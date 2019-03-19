use app_dirs::*;
use clap::{load_yaml, App};
use failure::{format_err, Error};
use lazy_static::lazy_static;
use log::{debug, info};
use ppool::{proxy_pool::*, *};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, sleep};
use std::time::Duration;

// app_dirs 相关配置
const APP_INFO: AppInfo = AppInfo {
    name: "ppool",
    author: "Aloxaf",
};

// app_dir 会自动创建所需目录(如果不存在的话
lazy_static! {
    static ref DATA_PATH: PathBuf = {
        let mut data_path = app_dir(AppDataType::UserData, &APP_INFO, "proxy_list")
            .expect("无法创建 UserData 目录");
        data_path.push("proxies.json");
        debug!("data_path: {:?}", &data_path);
        data_path
    };
}

fn init_proxy_pool() -> AProxyPool {
    // 读取(可能的)上次的数据
    info!("正在读取缓存");

    // 存在 proxies.json 的话, 读取 & 反序列化之
    let proxy_pool = match File::open(DATA_PATH.clone()) {
        Ok(file) => serde_json::from_reader(file).unwrap(),
        Err(err) => {
            // 打开失败时, 可能是不存在, 也可能是其他问题, 此处输出错误信息便于调试
            debug!("{:?}", err);
            ProxyPool::new()
        }
    };

    Arc::new(Mutex::new(proxy_pool))
}

fn init_config(config_file: Option<&String>) -> Result<Config, Error> {
    if let Some(config_file) = config_file {
        let mut file = std::fs::File::open(config_file)
            .map_err(|e| format_err!("无法读取配置文件: {:#?}", e))?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        Ok(toml::from_str(&s).map_err(|e| format_err!("配置文件解析失败: {:#?}", e))?)
    } else {
        // 默认配置的解析一般来说是不会失败的...
        Ok(toml::from_str(DEFAULT_CONFIG).expect("默认配置解析失败"))
    }
}

fn run() -> Result<(), Error> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if matches.occurrences_of("print_config") != 0 {
        println!("{}", DEFAULT_CONFIG);
        return Ok(());
    }

    let config_file = matches.value_of("config").map(ToOwned::to_owned);

    let proxy_pool = init_proxy_pool();
    let reload = Arc::new(RwLock::new(false));
    let password = Arc::new(RwLock::new(None));

    // 启动 server
    let server = {
        let proxy_pool = proxy_pool.clone();
        let reload = reload.clone();
        let password = password.clone();
        // TODO: 此处使用元组可读性太低了点...
        thread::spawn(|| ppool::server::launch_rocket((proxy_pool, reload, password)))
    };

    thread::spawn(move || loop {
        // 读取配置, 这个地方先解构一下
        // 因为 checker_config 是要被 Arc wrap 以后传来传去的
        // 而 spider_config 由于用到的相关变量都是 Copy 的, 所以直接传就行了
        info!("正在读取配置");
        let Config {
            password: new_password,
            checker: checker_config,
            spider: spider_config,
        }: Config = init_config(config_file.as_ref()).expect("解析配置文件错误");

        *password.write().unwrap() = Some(new_password);

        // 用 Arc wrap 一下 checker_config
        // TODO: 此处的 Arc 感觉可以避免, CheckerConfig 内部可以试试不继续嵌套 struct 了
        let checker_config = Arc::new(checker_config);

        *reload.write().unwrap() = false;

        // 爬虫线程
        let spider_thread = {
            let proxy_pool = proxy_pool.clone();
            let reload = reload.clone();
            thread::spawn(move || loop {
                // TODO: 此处单线程爬取, 可否直接传 &mut, 从而避免 clone ?
                spider_thread(proxy_pool.clone(), &spider_config);

                info!("等待{}秒再次爬取...", spider_config.interval);
                for _ in 0..spider_config.interval {
                    if *reload.read().unwrap() {
                        info!("检测到重载请求, 爬虫线程已结束");
                        break;
                    }
                    sleep(Duration::from_secs(1));
                }
            })
        };

        // 代理验证线程
        let checker_thread = {
            let proxy_pool = proxy_pool.clone();
            let reload = reload.clone();
            thread::spawn(move || {
                // 5s后开始验证, 免得验证时代理池是空的
                sleep(Duration::from_secs(5));
                loop {
                    checker_thread(proxy_pool.clone(), checker_config.clone());
                    // TODO: 这个"备份"也单独开一个线程?
                    info!("写入到磁盘");
                    let data = serde_json::to_string_pretty(&proxy_pool).expect("无法序列化");
                    let mut file = File::create(DATA_PATH.clone()).expect("无法创建文件");
                    file.write_all(data.as_bytes()).expect("无法写入");

                    info!("等待{}秒再次验证...", checker_config.interval);
                    for _ in 0..checker_config.interval {
                        if *reload.read().unwrap() {
                            info!("检测到重载请求, 验证线程已结束");
                            break;
                        }
                        sleep(Duration::from_secs(1));
                    }
                }
            })
        };

        spider_thread.join().expect("爬虫线程崩溃");
        checker_thread.join().expect("验证线程崩溃");
    });

    server.join().expect("服务器线程崩溃");

    Ok(())
}

fn main() {
    env_logger::init();

    if let Err(e) = run() {
        eprintln!("{:#?}", e);
        std::process::exit(1);
    }
}
