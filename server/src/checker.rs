use crate::AProxyPool;
use log::info;
use ppool_spider::utils::verify_proxy;
use ppool_spider::Proxy;
use threadpool::ThreadPool;

// TODO: 多线程

fn inc_failed_cnt(proxies: AProxyPool, proxy: &Proxy) {
    let mut proxies = proxies.lock().unwrap();
    proxies.info.get_mut(proxy.ip()).unwrap().failed += 1;
}

fn inc_success_cnt(proxies: AProxyPool, proxy: &Proxy) {
    let mut proxies = proxies.lock().unwrap();
    proxies.info.get_mut(proxy.ip()).unwrap().success += 1;
}

/// 先过一遍已验证代理, 再将未验证代理验证一遍加入已验证代理
/// 删除验证次数 >= 4 && 失败率 > 0.5 的代理
pub fn checker_thread(proxies: AProxyPool) {
    info!("checker thread start!");
    // TODO: 避免 clone ?
    // 为了避免验证代理时造成阻塞, 先 clone 一遍
    let (verified, unverified) = {
        let proxies = proxies.lock().unwrap();
        (
            proxies.get_verified().clone(),
            proxies.get_unverified().clone(),
        )
    };

    // 验证已验证代理
    for i in 0..verified.len() {
        let proxy = &verified[i];
        if !verify_proxy(proxy) {
            info!("[failed] verify proxy: {}:{}", proxy.ip(), proxy.port());
            inc_failed_cnt(proxies.clone(), proxy);
        } else {
            info!("[success] verify proxy: {}:{}", proxy.ip(), proxy.port());
            inc_success_cnt(proxies.clone(), proxy);
        }

        let mut proxies = proxies.lock().unwrap();
        let success = proxies.info.get(proxy.ip()).unwrap().success;
        let failed = proxies.info.get(proxy.ip()).unwrap().failed;
        if failed * 2 > success {
            info!(
                "[{}/{}] delete proxy: {}:{}",
                success,
                failed,
                proxy.ip(),
                proxy.port()
            );
            proxies.remove_verified(proxy);
        }
    }

    // 验证未验证代理
    for proxy in unverified {
        if verify_proxy(&proxy) {
            info!("[success] verify proxy: {}:{}", proxy.ip(), proxy.port());
            let mut proxies = proxies.lock().unwrap();
            proxies.insert_verified(proxy.clone());
        }
        let mut proxies = proxies.lock().unwrap();
        proxies.remove_unverified(&proxy);
    }

    info!("checker thread end!");
}
