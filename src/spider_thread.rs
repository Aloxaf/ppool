use super::spider::getter::table_getter;
use crate::config::*;
use crate::AProxyPool;
use log::{error, info};

/// 爬虫线程
pub fn spider_thread(proxy_pool: AProxyPool, spider_config: &SpiderConfig) {
    info!("代理爬取开始");
    for rules in &spider_config.common_table {
        let CommonTable {
            enable,
            name,
            urls,
            xpath_line,
            xpath_col,
            info_index,
        } = rules;

        if !enable {
            continue;
        }
        let proxies = match table_getter(name, urls, xpath_line, xpath_col, info_index) {
            Err(e) => {
                error!("{:#?}", e);
                vec![]
            }
            Ok(v) => v,
        };
        let mut proxy_pool = proxy_pool.lock().expect("spider_thread: 无法获取锁");
        proxy_pool.extend_unstable(proxies);
    }
    for rules in &spider_config.common_regex {
        let CommonRegex { enable, .. } = rules;
        if !enable {
            continue;
        }
        unimplemented!("通过 regex 自定义爬虫尚未实现")
    }
    info!("代理爬取结束");
}
