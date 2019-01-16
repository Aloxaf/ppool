#![feature(vec_remove_item)]

pub mod checker;
pub mod spider;

use ppool_spider::Proxy;
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
    success: u32,
    /// 失败验证次数
    failed: u32,
}

// 这个地方简直疯掉了, 干脆全部暴露出来让调用者自己处理
/// 代理池
/// O(1) 的插入时间复杂度
/// O(1) 的随机取时间复杂度ip.to_string()
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProxyPool {
    /// 未验证的代理
    pub unverified: Vec<Proxy>,
    /// 验证过的代理
    pub verified: Vec<Proxy>,
    /// 用于去重 & 记录验证失败次数
    pub info: ProxyInfo,
}

impl ProxyPool {
    pub fn new() -> Self {
        Default::default()
    }

    /// 插入新代理到未验证列表中
    pub fn insert_unverified(&mut self, proxy: Proxy) {
        let exist = self.info.get(&proxy.get_key()).is_some();
        if !exist {
            self.unverified.push(proxy);
        }
    }

    /// 插入新代理到已验证列表中
    pub fn insert_verified(&mut self, proxy: Proxy) {
        self.info.insert(proxy.get_key(), Default::default());
        self.verified.push(proxy);
    }

    /// 删除一个未验证代理
    pub fn remove_unverified(&mut self, proxy: &Proxy) {
        // 反正都用 rocket 了, unstable feature 用起来!
        self.unverified.remove_item(proxy).unwrap();
    }

    /// 删除一个已验证代理
    pub fn remove_verified(&mut self, proxy: &Proxy) {
        self.verified.remove_item(proxy).unwrap();
        self.info.remove(&proxy.get_key()).unwrap();
    }

    /// 随机取出一个已验证代理
    pub fn get_random(&mut self) -> &Proxy {
        let mut rng = thread_rng();
        self.verified.choose(&mut rng).unwrap()
    }

    /// 获取未验证代理的引用
    pub fn get_unverified(&self) -> &Vec<Proxy> {
        &self.unverified
    }

    /// 获取已验证代理的引用
    pub fn get_verified(&self) -> &Vec<Proxy> {
        &self.verified
    }

    /// 获取代理其他信息的引用
    pub fn get_info(&self) -> &ProxyInfo {
        &self.info
    }

    /// 设置未验证代理
    pub fn set_unverified(&mut self, unverified: Vec<Proxy>) {
        mem::replace(&mut self.unverified, unverified);
    }

    /// 设置已验证代理
    pub fn set_verified(&mut self, verified: Vec<Proxy>) {
        mem::replace(&mut self.verified, verified);
    }

    /// 设置代理信息
    pub fn set_info(&mut self, info: ProxyInfo) {
        mem::replace(&mut self.info, info);
    }

    fn extend_unverified<T: IntoIterator<Item = Proxy>>(&mut self, iter: T) {
        for proxy in iter {
            self.insert_unverified(proxy);
        }
    }
}
