use crate::checker_thread::checker_thread;
use crate::config::*;
use crate::options::Opt;
use crate::proxy_pool::*;
use crate::server::MyState;
use crate::spider::getter::*;
use crate::spider_thread::spider_thread;

use app_dirs::*;
use failure::{format_err, Error};
use lazy_static::lazy_static;
use log::{debug, info};
use structopt::StructOpt;

use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
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
        debug!("data_path: {}", data_path.display());
        data_path
    };
}

fn init_proxy_pool() -> Result<AProxyPool, Error> {
    // 读取(可能的)上次的数据
    info!("正在读取缓存");

    // 存在 proxies.json 的话, 读取 & 反序列化之
    if Path::new(&*DATA_PATH).exists() {
        let proxy_pool = serde_json::from_reader(File::open(&*DATA_PATH)?)?;
        Ok(Arc::new(proxy_pool))
    } else {
        Ok(Arc::new(ProxyPool::new()))
    }
}

fn init_config(config_file: Option<&String>) -> Result<Config, Error> {
    if let Some(config_file) = config_file {
        let mut file = std::fs::File::open(config_file)
            .map_err(|e| format_err!("无法读取配置文件: {}", e))?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        Ok(toml::from_str(&s).map_err(|e| format_err!("配置文件解析失败: {}", e))?)
    } else {
        // 默认配置的解析一般来说是不会失败的...
        Ok(toml::from_str(DEFAULT_CONFIG).expect("默认配置解析失败"))
    }
}

fn test_proxy(config_file: Option<String>, rule_name: &str) {
    let config = init_config(config_file.as_ref()).expect("解析配置文件错误");

    for rules in &config.spider.common_table {
        let CommonTable {
            name,
            urls,
            xpath_line,
            xpath_col,
            info_index,
            ..
        } = rules;

        if name != rule_name {
            continue;
        }

        match table_getter(name, urls, xpath_line, xpath_col, info_index) {
            Err(e) => eprintln!("{}", e),
            Ok(v) => {
                for proxy in &v {
                    println!("{:?}", proxy);
                }
            }
        };
    }

    for rules in &config.spider.common_regex {
        let CommonRegex {
            name,
            urls,
            ip,
            port,
            anonymity,
            ssl_type,
            ..
        } = rules;

        if name != rule_name {
            continue;
        }

        match regex_getter(name, urls, ip, port, anonymity, ssl_type) {
            Err(e) => eprintln!("{}", e),
            Ok(v) => {
                for proxy in &v {
                    println!("{:?}", proxy);
                }
            }
        };
    }
}

pub fn run() -> Result<(), Error> {
    let args: Opt = Opt::from_args();

    let config_file = args.config.clone();

    if args.print_config {
        println!("{}", DEFAULT_CONFIG);
        return Ok(());
    } else if args.test.is_some() {
        test_proxy(config_file, args.test.as_ref().unwrap());
        return Ok(());
    }

    let proxy_pool = init_proxy_pool()?;
    let reload = Arc::new(RwLock::new(false));
    let password = Arc::new(RwLock::new(None));

    // 启动 server
    let server = {
        let proxy_pool = proxy_pool.clone();
        let reload = reload.clone();
        let password = password.clone();
        thread::spawn(|| crate::server::launch_rocket(MyState::new(proxy_pool, reload, password)))
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
        // FIXME: rustc 说 checker_thread 未被使用, 让我加个 _ 前缀
        let _checker_thread = {
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
                    let mut file = File::create(&*DATA_PATH).expect("无法创建文件");
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
        _checker_thread.join().expect("验证线程崩溃");
    });

    server.join().expect("服务器线程崩溃");

    Ok(())
}
