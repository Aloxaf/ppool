use crate::AProxyPool;
use log::info;
use ppool_spider::utils::verify_proxy;

// TODO: 多线程
/// 验证代理的有效性
/// 删除失败 3 次的代理
pub fn checker_thread(proxies: AProxyPool) {
    // 为了避免验证代理时造成阻塞, 先将内容移出
    let (list, mut map) = {
        let tmp = proxies.lock().unwrap();
        (tmp.get_list().clone(), tmp.get_map().clone())
    };

    let mut failed = vec![];
    for i in 0..list.len() {
        let proxy = &list[i];
        if !verify_proxy(proxy) {
            info!("verify proxy: {}:{}, false", proxy.ip(), proxy.port());
            *map.get_mut(proxy.ip()).unwrap() += 1;
            if *map.get(proxy.ip()).unwrap() == 3 {
                failed.push(i);
            }
        } else {
            info!("verify proxy: {}:{}, true", proxy.ip(), proxy.port());
        }
    }
    for i in &failed {
        map.remove(list[*i].ip());
    }
    let list = list
        .iter()
        .enumerate()
        .filter_map(|(idx, proxy)| {
            if failed.contains(&idx) {
                Some(proxy.to_owned())
            } else {
                None
            }
        })
        .collect();
    let mut proxies = proxies.lock().unwrap();
    proxies.set_list(list);
    proxies.set_map(map);
}
