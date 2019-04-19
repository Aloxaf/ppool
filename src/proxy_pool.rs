use crate::spider::proxy::*;
use owning_ref::RwLockReadGuardRef;
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddrV4;
use std::sync::{Arc, RwLock};

pub type AProxyPool = Arc<ProxyPool>;
pub type ProxyInfo = RwLock<HashMap<SocketAddrV4, _ProxyInfo>>;
pub type ProxyList = RwLock<ProxyListInner>;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProxyListInner {
    /// 不稳定代理
    unstable: Vec<Proxy>,
    /// 稳定代理
    stable: Vec<Proxy>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct _ProxyInfo {
    /// 成功验证次数
    pub success: u32,
    /// 失败验证次数
    pub failed: u32,
    /// 连续失败次数, 这一项主要是防止一个高稳定性代理下线以后迟迟不能被剔除
    pub fail_times: u8,
}

impl _ProxyInfo {
    #[inline]
    pub fn stability(&self) -> f64 {
        f64::from(self.success) / f64::from(self.check_cnt())
    }

    #[inline]
    pub fn check_cnt(&self) -> u32 {
        self.success + self.failed
    }
}

/// 代理池
/// O(1) 的插入时间复杂度
/// O(1) 的随机取时间复杂度
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProxyPool {
    list: ProxyList,
    info: ProxyInfo,
}

// TODO: 一堆 unwrap() ?
// &mut self/&self 时由于可能线程不安全, 无法把 Error 传来传去
// 那 Arc<Self> 呢
impl ProxyPool {
    pub fn new() -> Self {
        Default::default()
    }

    /// 移动代理到稳定列表中
    pub fn move_to_stable(self: Arc<Self>, proxy: &Proxy) {
        let mut proxy_list = self.list.write().unwrap();
        let proxy = proxy_list.unstable.remove_item(&proxy).unwrap();
        proxy_list.stable.push(proxy);
    }

    /// 移动代理到不稳定列表中
    pub fn move_to_unstable(self: Arc<Self>, proxy: &Proxy) {
        let mut proxy_list = self.list.write().unwrap();
        let proxy = proxy_list.stable.remove_item(&proxy).unwrap();
        proxy_list.unstable.push(proxy);
    }

    /// 从不稳定列表中删除一个代理
    pub fn remove_unstable(self: Arc<Self>, proxy: &Proxy) {
        // 反正都用 rocket 了, unstable feature 用起来!
        let mut proxy_list = self.list.write().unwrap();
        proxy_list.unstable.remove_item(proxy).unwrap();
        self.info.write().unwrap().remove(&proxy.get_key()).unwrap();
    }

    /// 从稳定列表中删除一个代理
    pub fn _remove_stable(self: Arc<Self>, proxy: &Proxy) {
        let mut proxy_list = self.list.write().unwrap();
        proxy_list.stable.remove_item(proxy).unwrap();
        self.info.write().unwrap().remove(&proxy.get_key()).unwrap();
    }

    /// 从稳定列表中随机取出一个代理
    pub fn get_random(self: Arc<Self>) -> Option<Proxy> {
        let mut rng = thread_rng();
        let list = self.list.read().unwrap();
        list.stable.choose(&mut rng).cloned()
    }

    /// 根据条件筛选代理
    pub fn select(
        self: Arc<Self>,
        ssl_type: Option<String>,
        anonymity: Option<String>,
        stability: Option<f32>,
    ) -> Vec<Proxy> {
        let proxy_list = self.list.read().unwrap();
        let proxy_info = self.info.read().unwrap();
        // 此处将所有 Iterator 泛化为 Iterator<Item = &Proxy>, 以便使用同一个变量存储中间结果
        // 省去 collect 开销
        let mut iter = Box::new(proxy_list.stable.iter()) as Box<Iterator<Item = &Proxy>>;
        if let Some(ssl_type) = ssl_type {
            let ssl_type = ssl_type.parse().unwrap();
            iter = Box::new(iter.filter(move |proxy| proxy.ssl_type() == ssl_type))
                as Box<Iterator<Item = &Proxy>>;
        }
        if let Some(anonymity) = anonymity {
            let anonymity = anonymity.parse().unwrap();
            iter = Box::new(iter.filter(move |proxy| proxy.anonymity() == anonymity))
                as Box<Iterator<Item = &Proxy>>;
        }
        if let Some(stability) = stability {
            iter = Box::new(iter.filter(move |proxy| {
                let item = &proxy_info[&proxy.get_key()];
                let failed = item.failed as f32;
                let success = item.success as f32;
                success / (success + failed) >= stability
            })) as Box<Iterator<Item = &Proxy>>;
        }
        iter.cloned().collect()
        // FIXME: 究极 clone
    }

    pub fn select_random(
        self: Arc<Self>,
        ssl_type: Option<String>,
        anonymity: Option<String>,
        stability: Option<f32>,
    ) -> Option<Proxy> {
        let mut rng = thread_rng();
        self.select(ssl_type, anonymity, stability)
            .choose(&mut rng)
            .cloned()
    }

    /// 获取未验证代理的引用
    pub fn get_unstable(&self) -> RwLockReadGuardRef<ProxyListInner, Vec<Proxy>> {
        RwLockReadGuardRef::new(self.list.read().unwrap()).map(|list| &list.unstable)
    }

    /// 获取已验证代理的引用
    pub fn get_stable(&self) -> RwLockReadGuardRef<ProxyListInner, Vec<Proxy>> {
        RwLockReadGuardRef::new(self.list.read().unwrap()).map(|list| &list.stable)
    }

    /// 代理验证失败计数 +1
    pub fn inc_failed_cnt(self: Arc<Self>, proxy: &Proxy) {
        let mut info = self.info.write().unwrap();
        let mut info = info.get_mut(&proxy.get_key()).unwrap();
        info.failed += 1;
        info.fail_times += 1;
    }

    /// 代理验证成功计数 +1
    pub fn inc_success_cnt(self: Arc<Self>, proxy: &Proxy) {
        let mut info = self.info.write().unwrap();
        let mut info = info.get_mut(&proxy.get_key()).unwrap();
        info.success += 1;
        info.fail_times = 0;
    }

    pub fn get_info(self: Arc<Self>, proxy: &Proxy) -> (f64, u32, u8) {
        let info = self.info.read().unwrap();
        let proxy_info = info.get(&proxy.get_key()).unwrap();
        (
            proxy_info.stability(),
            proxy_info.check_cnt(),
            proxy_info.fail_times,
        )
    }

    pub fn extend_unstable<T: IntoIterator<Item = Proxy>>(self: Arc<Self>, iter: T) {
        let mut proxy_info = self.info.write().unwrap();
        let mut proxy_list = self.list.write().unwrap();
        for proxy in iter {
            let exist = proxy_info.get(&proxy.get_key()).is_some();
            if !exist {
                proxy_info.insert(proxy.get_key(), Default::default());
                proxy_list.unstable.push(proxy);
            }
        }
    }
}
