use crate::AProxyPool;
use log::{error, info};
use ppool_spider::proxy_getter::FUNCS;

/// 爬虫线程
pub fn spider_thread(proxies: AProxyPool) {
    info!("spider thread start!");
    for func in &FUNCS {
        let ret = func().unwrap_or_else(|err| {
            error!("{:?}", err);
            vec![]
        });
        let mut proxies = proxies.lock().unwrap();
        proxies.extend_unverified(ret);
    }
    info!("spider thread end!");
}
