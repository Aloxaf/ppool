use ppool_spider::{AnonymityLevel, Proxy, SslType};
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem;
use std::sync::{Arc, Mutex};

pub type AProxyPool = Arc<Mutex<ProxyPool>>;
// TODO: 这个地方不想用 String, 额外 clone 了一次
pub type ProxyInfo = HashMap<String, Info>;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Info {
    /// 成功验证次数
    pub success: u32,
    /// 失败验证次数
    pub failed: u32,
    /// 连续失败次数
    pub seq: u8,
}

// 这个地方简直疯掉了, 干脆全部暴露出来让调用者自己处理
/// 代理池
/// O(1) 的插入时间复杂度
/// O(1) 的随机取时间复杂度
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProxyPool {
    /// 不稳定代理
    pub unstable: Vec<Proxy>,
    /// 稳定代理
    pub stable: Vec<Proxy>,
    /// 用于去重 & 记录验证失败次数
    pub info: ProxyInfo,
}

impl ProxyPool {
    pub fn new() -> Self {
        Default::default()
    }

    /// 插入新代理到不稳定列表中
    pub fn insert_unstable(&mut self, proxy: Proxy) {
        let exist = self.info.get(&proxy.get_key()).is_some();
        if !exist {
            self.info.insert(proxy.get_key(), Default::default());
            self.unstable.push(proxy);
        }
    }

    /// 移动代理到稳定列表中
    pub fn move_to_stable(&mut self, proxy: &Proxy) {
        let proxy = self.unstable.remove_item(&proxy).unwrap();
        self.stable.push(proxy);
    }

    /// 移动代理到不稳定列表中
    pub fn move_to_unstable(&mut self, proxy: &Proxy) {
        let proxy = self.stable.remove_item(&proxy).unwrap();
        self.unstable.push(proxy);
    }

    /// 从不稳定列表中删除一个代理
    pub fn remove_unstable(&mut self, proxy: &Proxy) {
        // 反正都用 rocket 了, unstable feature 用起来!
        self.unstable.remove_item(proxy).unwrap();
        self.info.remove(&proxy.get_key()).unwrap();
    }

    /// 从稳定列表中删除一个代理
    pub fn remove_stable(&mut self, proxy: &Proxy) {
        self.stable.remove_item(proxy).unwrap();
        self.info.remove(&proxy.get_key()).unwrap();
    }

    /// 从稳定列表中随机取出一个代理
    pub fn get_random(&mut self) -> &Proxy {
        let mut rng = thread_rng();
        self.stable.choose(&mut rng).unwrap()
    }

    // TODO: 效率太低了
    /// 根据条件筛选代理
    pub fn select(
        &self,
        ssl_type: Option<String>,
        anonymity: Option<String>,
        stability: Option<f32>,
    ) -> Vec<&Proxy> {
        let mut iter = self.stable.iter().map(|x| x).collect::<Vec<_>>();
        if let Some(ssl_type) = ssl_type {
            let ssl_type = SslType::from(ssl_type);
            iter = iter
                .into_iter()
                .filter(|proxy| proxy.ssl_type() == ssl_type)
                .collect();
        }
        if let Some(anonymity) = anonymity {
            let anonymity = AnonymityLevel::from(anonymity);
            iter = iter
                .into_iter()
                .filter(|proxy| proxy.anonymity() == anonymity)
                .collect();
        }
        if let Some(stability) = stability {
            iter = iter
                .into_iter()
                .filter(|proxy| {
                    let item = &self.info[&proxy.get_key()];
                    let failed = item.failed as f32;
                    let success = item.success as f32;
                    success / (success + failed) >= stability
                })
                .collect()
        }
        iter
    }

    pub fn select_random(
        &self,
        ssl_type: Option<String>,
        anonymity: Option<String>,
        stability: Option<f32>,
    ) -> Option<&Proxy> {
        let mut rng = thread_rng();
        self.select(ssl_type, anonymity, stability)
            .choose(&mut rng)
            .map(|proxy| *proxy)
    }

    /// 获取未验证代理的引用
    pub fn get_unstable(&self) -> &Vec<Proxy> {
        &self.unstable
    }

    /// 获取已验证代理的引用
    pub fn get_stable(&self) -> &Vec<Proxy> {
        &self.stable
    }

    /// 获取代理其他信息的引用
    pub fn get_info(&self) -> &ProxyInfo {
        &self.info
    }

    /// 设置代理信息
    pub fn set_info(&mut self, info: ProxyInfo) {
        mem::replace(&mut self.info, info);
    }

    pub fn extend_unstable<T: IntoIterator<Item = Proxy>>(&mut self, iter: T) {
        for proxy in iter {
            self.insert_unstable(proxy);
        }
    }
}
