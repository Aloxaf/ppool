use crate::config::*;
use crate::proxy_pool::*;
use crate::spider::getter::{regex_getter, table_getter};
use log::{error, info};

/// 爬虫线程
pub fn spider_thread(proxy_pool: AProxyPool, spider_config: &SpiderConfig) {
    info!("代理爬取开始");

    // 此处原本计划是 CommonTable, CommonRegex 都是 enum Rules 的成员, 这样写起来好看一点
    // 然而在反序列化 toml 的时候出现了一点问题, 干脆就改成每类规则占用一个成员变量
    for rules in &spider_config.common_table {
        // 大解构, 这样下面就能少写点代码了
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
        let CommonRegex {
            enable,
            name,
            urls,
            ip,
            port,
            anonymity,
            ssl_type,
        } = rules;
        if !enable {
            continue;
        }
        let proxies = match regex_getter(name, urls, ip, port, anonymity, ssl_type) {
            Err(e) => {
                error!("{:#?}", e);
                vec![]
            }
            Ok(v) => v,
        };
        let mut proxy_pool = proxy_pool.lock().expect("spider_thread: 无法获取锁");
        proxy_pool.extend_unstable(proxies);
    }
    info!("代理爬取结束");
}
