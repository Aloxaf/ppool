use crate::AProxyPool;
use log::info;
use ppool_spider::utils::check_proxy;
use ppool_spider::Proxy;
use threadpool::ThreadPool;

fn inc_failed_cnt(proxies: AProxyPool, proxy: &Proxy) {
    let mut proxies = proxies.lock().expect("inc failed");
    proxies
        .info
        .get_mut(&proxy.get_key())
        .expect("no key")
        .failed += 1;
}

fn inc_success_cnt(proxies: AProxyPool, proxy: &Proxy) {
    let mut proxies = proxies.lock().expect("inc success");
    proxies
        .info
        .get_mut(&proxy.get_key())
        .expect("no key")
        .success += 1;
}

// 验证已验证代理
fn check_stable(proxies: AProxyPool) {
    // TODO: 避免 clone ?
    // 为了避免验证代理时造成阻塞, 先 clone 一遍
    let stable = {
        let proxies = proxies.lock().expect("get lock: checker thread start");
        proxies.get_stable().clone()
    };

    let pool = ThreadPool::new(20);

    for i in 0..stable.len() {
        // TODO: 避免 clone ?
        let proxy = stable[i].clone();
        let proxies = proxies.clone();
        pool.execute(move || {
            if !check_proxy(&proxy) {
                info!("验证成功: {}:{}", proxy.ip(), proxy.port());
                inc_failed_cnt(proxies.clone(), &proxy);
            } else {
                info!("验证失败: {}:{}", proxy.ip(), proxy.port());
                inc_success_cnt(proxies.clone(), &proxy);
            }

            let mut proxies = proxies.lock().expect("get lock: after stable");
            let success = proxies.info.get(&proxy.get_key()).expect("no key").success;
            let failed = proxies.info.get(&proxy.get_key()).expect("no key").failed;
            if failed > success {
                info!(
                    "失败次数:{}/{}, 从已验证列表移出",
                    failed,
                    success + failed
                );
                proxies.remove_stable(&proxy);
            }
        });
    }
    pool.join();
}

// 验证未验证代理
fn check_unstable(proxies: AProxyPool) {
    let unstable = {
        let proxies = proxies.lock().expect("get lock: checker thread start");
        proxies.get_unstable().clone()
    };

    let pool = ThreadPool::new(20);

    for proxy in unstable {
        let proxy = proxy.clone();
        let proxies = proxies.clone();
        pool.execute(move || {
            if check_proxy(&proxy) {
                info!("验证成功: {}:{}", proxy.ip(), proxy.port());
                inc_failed_cnt(proxies.clone(), &proxy);
            } else {
                info!("验证失败: {}:{}", proxy.ip(), proxy.port());
                inc_success_cnt(proxies.clone(), &proxy);
            }

            let mut proxies = proxies.lock().expect("get lock: after stable");
            let success = proxies.info.get(&proxy.get_key()).expect("no key").success;
            let failed = proxies.info.get(&proxy.get_key()).expect("no key").failed;
            if failed + success >= 4 && success > failed {
                info!(
                    "成功次数:{}/{}, 添加到已验证列表",
                    success,
                    success + failed
                );
                proxies.move_to_stable(&proxy);
            } else if failed + success >= 4 && success <= failed {
                info!(
                    "失败次数:{}/{}, 从未验证列表移出",
                    failed,
                    success + failed
                );
                proxies.remove_unstable(&proxy);
            }
        });
    }
    pool.join();
}

/// 先过一遍已验证代理, 再将未验证代理验证一遍加入已验证代理
/// 删除验证次数 >= 4 && 失败率 > 0.5 的代理
pub fn checker_thread(proxies: AProxyPool) {
    info!("代理验证开始");
    check_stable(proxies.clone());
    check_unstable(proxies);
    info!("代理验证结束");
}
