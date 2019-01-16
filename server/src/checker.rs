use crate::AProxyPool;
use log::info;
use ppool_spider::utils::verify_proxy;
use ppool_spider::Proxy;
use threadpool::ThreadPool;

fn inc_failed_cnt(proxies: AProxyPool, proxy: &Proxy) {
    let mut proxies = proxies.lock().expect("inc failed");
    proxies.info.get_mut(&proxy.get_key()).expect("no key").failed += 1;
}

fn inc_success_cnt(proxies: AProxyPool, proxy: &Proxy) {
    let mut proxies = proxies.lock().expect("inc success");
    proxies.info.get_mut(&proxy.get_key()).expect("no key").success += 1;
}

/// 先过一遍已验证代理, 再将未验证代理验证一遍加入已验证代理
/// 删除验证次数 >= 4 && 失败率 > 0.5 的代理
pub fn checker_thread(proxies: AProxyPool) {
    info!("checker thread start!");
    // TODO: 避免 clone ?
    // 为了避免验证代理时造成阻塞, 先 clone 一遍
    let (verified, unverified) = {
        let proxies = proxies.lock().expect("get lock: checker thread start");
        (
            proxies.get_verified().clone(),
            proxies.get_unverified().clone(),
        )
    };

    let pool = ThreadPool::new(20);

    // 验证已验证代理
    for i in 0..verified.len() {
        let proxy = verified[i].clone();
        let proxies = proxies.clone();
        pool.execute(move || {
            if !verify_proxy(&proxy) {
                info!("[failed] verify proxy: {}:{}", proxy.ip(), proxy.port());
                inc_failed_cnt(proxies.clone(), &proxy);
            } else {
                info!("[success] verify proxy: {}:{}", proxy.ip(), proxy.port());
                inc_success_cnt(proxies.clone(), &proxy);
            }

            let mut proxies = proxies.lock().expect("get lock: after verified");
            let success = proxies.info.get(&proxy.get_key()).expect("no key").success;
            let failed = proxies.info.get(&proxy.get_key()).expect("no key").failed;
            if failed + success >= 4 && failed * 2 > success {
                info!(
                    "[{}/{}] delete proxy: {}:{}",
                    success,
                    failed,
                    proxy.ip(),
                    proxy.port()
                );
                proxies.remove_verified(&proxy);
            }
        });
    }
    pool.join();

    // 验证未验证代理
    for proxy in unverified {
        let proxy = proxy.clone();
        let proxies = proxies.clone();
        pool.execute(move || {
            if verify_proxy(&proxy) {
                info!("[success] verify proxy: {}:{}", proxy.ip(), proxy.port());
                let mut proxies = proxies.lock().expect("get lock: insert verified");
                proxies.insert_verified(proxy.clone());
            } else {
                info!("[failed] verify proxy: {}:{}", proxy.ip(), proxy.port());
            }
            let mut proxies = proxies.lock().expect("get lock: remove unverified");
            proxies.remove_unverified(&proxy);
        });
    }
    pool.join();

    info!("checker thread end!");
}
