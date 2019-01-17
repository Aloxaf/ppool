use crate::AProxyPool;
use log::info;
use ppool_spider::utils::check_proxy;
use ppool_spider::Proxy;
use threadpool::ThreadPool;

fn inc_failed_cnt(proxies: AProxyPool, proxy: &Proxy) {
    let mut proxies = proxies.lock().expect("inc failed");
    let mut info = proxies.info.get_mut(&proxy.get_key()).expect("no key");
    info.failed += 1;
    info.seq += 1;
}

fn inc_success_cnt(proxies: AProxyPool, proxy: &Proxy) {
    let mut proxies = proxies.lock().expect("inc success");
    let mut info = proxies.info.get_mut(&proxy.get_key()).expect("no key");
    info.success += 1;
    info.seq = 0;
}

// 检查稳定代理
fn check_stable(proxies: AProxyPool) {
    // TODO: 避免 clone ?
    // 为了避免验证代理时造成阻塞, 先 clone 一遍
    let stable = {
        let proxies = proxies.lock().expect("get lock: checker thread start");
        proxies.get_stable().clone()
    };

    let pool = ThreadPool::new(25);

    for proxy in stable {
        // TODO: 避免 clone ?
        let proxy = proxy.clone();
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
            let info = proxies.info.get(&proxy.get_key()).expect("no key");
            let success = info.success as f32;
            let failed = info.failed as f32;
            if success / (failed + success) < 0.7 {
                info!(
                    "稳定率:{:.2}, 降级为不稳定",
                    success / (success + failed)
                );
                proxies.move_to_unstable(&proxy);
            } else if info.seq >= 4 {
                info!("连续验证失败4次, 降级为不稳定");
                proxies.move_to_unstable(&proxy);
            }
        });
    }
    pool.join();
}

// 检查不稳定代理
fn check_unstable(proxies: AProxyPool) {
    let unstable = {
        let proxies = proxies.lock().expect("get lock: checker thread start");
        proxies.get_unstable().clone()
    };

    let pool = ThreadPool::new(25);

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
            let info = proxies.info.get(&proxy.get_key()).expect("no key");
            let success = info.success as f32;
            let failed = info.failed as f32;
            let stability = success / (failed + success);
            if failed + success >= 5.0 && stability >= 0.7 {
                info!("稳定率:{:.2}, 标记为稳定", stability);
                proxies.move_to_stable(&proxy);
            } else if failed + success >= 5.0 && stability < 0.6 {
                info!("稳定率:{:.2}, 从列表中移出", stability);
                proxies.remove_unstable(&proxy);
            } else if info.seq >= 6 {
                info!("连续验证失败4次, 从列表中移出");
                proxies.remove_unstable(&proxy);
            }
        });
    }
    pool.join();
}

/// 代理稳定性检查线程
pub fn checker_thread(proxies: AProxyPool) {
    info!("代理验证开始");
    check_stable(proxies.clone());
    check_unstable(proxies);
    info!("代理验证结束");
}
