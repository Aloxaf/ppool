use app_dirs::*;
use clap::{load_yaml, App};
use failure::{format_err, Error};
use log::{debug, info};
use ppool::{proxy_pool::*, *};
use std::fs::File;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;

// app_dirs 相关配置
const APP_INFO: AppInfo = AppInfo {
    name: "ppool",
    author: "Aloxaf",
};

fn run() -> Result<(), Error> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if matches.occurrences_of("print_config") != 0 {
        println!("{}", DEFAULT_CONFIG);
        return Ok(());
    }

    // 读取配置, 这个地方先解构一下
    // 因为 checker_config 是要被 Arc wrap 以后传来传去的
    // 而 spider_config 由于用到的相关变量都是 Copy 的, 所以直接传就行了
    info!("正在读取配置");
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
        // 默认配置的解析一般来说是不会失败的...
        toml::from_str(DEFAULT_CONFIG).unwrap()
    };

    // 用 Arc wrap 一下 checker_config
    // TODO: 此处的 Arc 感觉可以避免, CheckerConfig 内部可以试试不继续嵌套 struct 了
    let checker_config = Arc::new(checker_config);

    // 读取(可能的)上次的数据
    info!("正在读取缓存");
    // app_dir 会自动创建所需目录(如果不存在的话
    let mut data_path = app_dir(AppDataType::UserData, &APP_INFO, "proxy_list")
        .expect("无法创建 UserData 目录");
    data_path.push("proxies.json");
    debug!("data_path: {:?}", &data_path);

    // 存在 proxies.json 的话, 读取 & 反序列化之
    let proxy_pool = match File::open(&data_path) {
        Ok(file) => serde_json::from_reader(file).unwrap(),
        Err(err) => {
            // 打开失败时, 可能是不存在, 也可能是其他问题, 此处输出错误信息便于调试
            debug!("{:?}", err);
            ProxyPool::new()
        }
    };

    let proxy_pool = Arc::new(Mutex::new(proxy_pool));

    // 先启动 server, 防止等会爬虫请求代理的时候 503
    ppool::server::launch_rocket(proxy_pool.clone());

    // 爬虫线程
    // 此处(包括下面)用大括号开启新的作用域, 主要是防止对 proxy_pool 的 shadow & move
    // 导致下面无法继续使用 proxy_pool
    // 而且感觉将两个部分分开了挺好看的 (
    {
        let proxy_pool = proxy_pool.clone();
        thread::spawn(move || loop {
            // TODO: 此处单线程爬取, 可否直接传 &mut, 从而避免 clone ?
            spider_thread(proxy_pool.clone(), &spider_config);
            info!("等待{}秒再次爬取...", spider_config.interval);
            sleep(Duration::from_secs(spider_config.interval));
        });
    }

    // 5s后开始验证, 免得验证时代理池是空的
    sleep(Duration::from_secs(5));

    // 代理验证线程
    {
        let proxy_pool = proxy_pool.clone();
        thread::spawn(move || loop {
            checker_thread(proxy_pool.clone(), checker_config.clone());
            // TODO: 这个"备份"也单独开一个线程?
            info!("写入到磁盘");
            let data = serde_json::to_string_pretty(&proxy_pool).expect("无法序列化");
            let mut file = File::create(&data_path).expect("无法创建文件");
            file.write_all(data.as_bytes()).expect("无法写入");
            info!("等待{}秒再次验证...", checker_config.interval);
            sleep(Duration::from_secs(checker_config.interval));
        });
    }

    Ok(())
}

fn main() {
    env_logger::init();

    if let Err(e) = run() {
        eprintln!("{:#?}", e);
        std::process::exit(1);
    }
}
