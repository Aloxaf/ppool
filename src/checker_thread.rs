use crate::config::CheckerConfig;
use crate::spider::proxy::Proxy;
use crate::spider::utils::check_proxy;
use crate::AProxyPool;
use log::info;
use std::sync::Arc;
use threadpool::ThreadPool;

// TODO: 这个地方频繁上锁是否会影响并发性能
#[inline]
fn inc_failed_cnt(proxy_pool: &AProxyPool, proxy: &Proxy) {
    let mut proxy_pool = proxy_pool.write().expect("inc_failed_cnt: 无法获取锁");
    let mut info = proxy_pool
        .info
        .get_mut(&proxy.get_key())
        .expect("inc_failed_cnt: 查无此键");
    info.failed += 1;
    info.fail_times += 1;
}

#[inline]
fn inc_success_cnt(proxy_pool: &AProxyPool, proxy: &Proxy) {
    let mut proxy_pool = proxy_pool.write().expect("inc_success_cnt: 无法获取锁");
    let mut info = proxy_pool
        .info
        .get_mut(&proxy.get_key())
        .expect("inc_success_cnt: 查无此键");
    info.success += 1;
    info.fail_times = 0;
}

// 检查稳定代理
fn check_stable(proxy_pool: AProxyPool, checker_config: Arc<CheckerConfig>) {
    // TODO: 避免 clone ?
    // 为了避免验证代理时造成阻塞, 先 clone 一遍
    let stable = {
        let proxy_pool = proxy_pool.read().expect("check_stable: 无法获取锁");
        proxy_pool.get_stable().clone()
    };

    let pool = ThreadPool::new(checker_config.max_workers);

    // 反正是 clone 的, consume 掉也无所谓
    for proxy in stable {
        let proxy_pool = proxy_pool.clone();
        // TODO: 避免 clone ?
        let checker_config = checker_config.clone();

        pool.execute(move || {
            if check_proxy(&proxy, &checker_config) {
                info!("验证成功: {}:{}", proxy.ip(), proxy.port());
                inc_success_cnt(&proxy_pool, &proxy);
            } else {
                info!("验证失败: {}:{}", proxy.ip(), proxy.port());
                inc_failed_cnt(&proxy_pool, &proxy);
            }

            let mut proxy_pool = proxy_pool.write().expect("无法获取锁");
            let info = proxy_pool.info.get(&proxy.get_key()).expect("查无此键");
            let stability = info.stability();

            // 稳定率过低
            if stability < checker_config.stability.level_down {
                info!("稳定率:{:.2}, 降级为不稳定", stability);
                proxy_pool.move_to_unstable(&proxy);
            // 连续失败次数过多
            } else if info.fail_times >= checker_config.fail_times.level_down {
                info!(
                    "连续验证失败{}次, 降级为不稳定",
                    checker_config.fail_times.level_down
                );
                proxy_pool.move_to_unstable(&proxy);
            }
        });
    }

    // 等待所有线程结束
    pool.join();
}

// TODO: 这两个函数大体框架一致, 是否能简化一下?
// 检查不稳定代理
fn check_unstable(proxy_pool: AProxyPool, checker_config: Arc<CheckerConfig>) {
    let unstable = {
        let proxy_pool = proxy_pool.read().expect("check_unstable: 无法获取锁");
        proxy_pool.get_unstable().clone()
    };

    let pool = ThreadPool::new(checker_config.max_workers);

    for proxy in unstable {
        let proxy_pool = proxy_pool.clone();
        let checker_config = checker_config.clone();

        pool.execute(move || {
            if check_proxy(&proxy, &checker_config) {
                info!("验证成功: {}:{}", proxy.ip(), proxy.port());
                inc_success_cnt(&proxy_pool, &proxy);
            } else {
                info!("验证失败: {}:{}", proxy.ip(), proxy.port());
                inc_failed_cnt(&proxy_pool, &proxy);
            }

            let mut proxy_pool = proxy_pool.write().expect("无法获取锁");
            let info = proxy_pool.info.get(&proxy.get_key()).expect("查无此键");
            let stability = info.stability();

            // 检测次数 & 稳定率达标
            if info.check_cnt() >= checker_config.min_cnt_level_up
                && stability >= checker_config.stability.level_up
            {
                info!("稳定率:{:.2}, 标记为稳定", stability);
                proxy_pool.move_to_stable(&proxy);
            // 稳定率过低
            } else if info.check_cnt() >= checker_config.min_cnt_remove
                && stability < checker_config.stability.remove
            {
                info!("稳定率:{:.2}, 从列表中移除", stability);
                proxy_pool.remove_unstable(&proxy);
            // 连续失败次数过多
            } else if info.fail_times >= checker_config.fail_times.remove {
                info!(
                    "连续验证失败{}次, 从列表中移除",
                    checker_config.fail_times.remove
                );
                proxy_pool.remove_unstable(&proxy);
            // 烂代理扶不上墙
            } else if info.check_cnt() >= checker_config.max_cnt_remove {
                info!(
                    "{}次验证后仍不稳定, 从列表中移除",
                    checker_config.max_cnt_remove
                );
                proxy_pool.remove_unstable(&proxy);
            }
        });
    }
    pool.join();
}

/// 代理稳定性检查线程
pub fn checker_thread(proxies: AProxyPool, checker_config: Arc<CheckerConfig>) {
    info!("代理验证开始");
    check_stable(proxies.clone(), checker_config.clone());
    // 节省一次 clone
    check_unstable(proxies, checker_config);
    info!("代理验证结束");
}
