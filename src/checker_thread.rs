use super::spider::proxy::Proxy;
use super::spider::utils::check_proxy;
use crate::config::CheckerConfig;
use crate::AProxyPool;
use log::info;
use std::sync::Arc;
use threadpool::ThreadPool;

// TODO: 这个地方频繁上锁是否会影响并发性能
fn inc_failed_cnt(proxies: AProxyPool, proxy: &Proxy) {
    let mut proxies = proxies.lock().expect("inc_failed_cnt: 无法获取锁");
    let mut info = proxies
        .info
        .get_mut(&proxy.get_key())
        .expect("inc_failed_cnt: 查无此键");
    info.failed += 1;
    info.fail_times += 1;
}

fn inc_success_cnt(proxies: AProxyPool, proxy: &Proxy) {
    let mut proxies = proxies.lock().expect("inc_success_cnt: 无法获取锁");
    let mut info = proxies
        .info
        .get_mut(&proxy.get_key())
        .expect("inc_success_cnt: 查无此键");
    info.success += 1;
    info.fail_times = 0;
}

// 检查稳定代理
fn check_stable(proxies: AProxyPool, checker_config: Arc<CheckerConfig>) {
    let stability_level_down = checker_config.stability.level_down;
    let fail_times_level_down = checker_config.fail_times.level_down;
    // TODO: 避免 clone ?
    // 为了避免验证代理时造成阻塞, 先 clone 一遍
    let stable = {
        let proxies = proxies.lock().expect("check_stable: 无法获取锁");
        proxies.get_stable().clone()
    };

    let pool = ThreadPool::new(30);

    for proxy in stable {
        // TODO: 避免 clone ?
        let proxy = proxy.clone();
        let proxies = proxies.clone();
        let checker_config = checker_config.clone();
        pool.execute(move || {
            if !check_proxy(&proxy, checker_config) {
                info!("验证成功: {}:{}", proxy.ip(), proxy.port());
                inc_failed_cnt(proxies.clone(), &proxy);
            } else {
                info!("验证失败: {}:{}", proxy.ip(), proxy.port());
                inc_success_cnt(proxies.clone(), &proxy);
            }

            let mut proxies = proxies.lock().expect("无法获取锁");
            let info = proxies.info.get(&proxy.get_key()).expect("查无此键");
            let success = info.success as f32;
            let failed = info.failed as f32;
            if success / (failed + success) < stability_level_down {
                info!(
                    "稳定率:{:.2}, 降级为不稳定",
                    success / (success + failed)
                );
                proxies.move_to_unstable(&proxy);
            } else if info.fail_times >= fail_times_level_down {
                info!("连续验证失败2次, 降级为不稳定");
                proxies.move_to_unstable(&proxy);
            }
        });
    }
    pool.join();
}

// 检查不稳定代理
fn check_unstable(proxies: AProxyPool, checker_config: Arc<CheckerConfig>) {
    let stability_level_up = checker_config.stability.level_up;
    let stability_remove = checker_config.stability.remove;
    let fail_times_remove = checker_config.fail_times.remove;
    let min_check_cnt = checker_config.min_check_cnt;
    let unstable = {
        let proxies = proxies.lock().expect("check_unstable: 无法获取锁");
        proxies.get_unstable().clone()
    };

    let pool = ThreadPool::new(25);

    for proxy in unstable {
        let proxy = proxy.clone();
        let proxies = proxies.clone();
        let checker_config = checker_config.clone();
        pool.execute(move || {
            if check_proxy(&proxy, checker_config) {
                info!("验证成功: {}:{}", proxy.ip(), proxy.port());
                inc_failed_cnt(proxies.clone(), &proxy);
            } else {
                info!("验证失败: {}:{}", proxy.ip(), proxy.port());
                inc_success_cnt(proxies.clone(), &proxy);
            }

            let mut proxies = proxies.lock().expect("无法获取锁");
            let info = proxies.info.get(&proxy.get_key()).expect("查无此键");
            let success = info.success as f32;
            let failed = info.failed as f32;
            let stability = success / (failed + success);
            if failed + success >= f32::from(min_check_cnt) {
                if stability >= stability_level_up {
                    info!("稳定率:{:.2}, 标记为稳定", stability);
                    proxies.move_to_stable(&proxy);
                } else if stability < stability_remove {
                    info!("稳定率:{:.2}, 从列表中移出", stability);
                    proxies.remove_unstable(&proxy);
                }
            } else if info.fail_times >= fail_times_remove {
                info!("连续验证失败3次, 从列表中移出");
                proxies.remove_unstable(&proxy);
            }
        });
    }
    pool.join();
}

/// 代理稳定性检查线程
pub fn checker_thread(proxies: AProxyPool, checker_config: Arc<CheckerConfig>) {
    info!("代理验证开始");
    check_stable(proxies.clone(), checker_config.clone());
    check_unstable(proxies, checker_config);
    info!("代理验证结束");
}
