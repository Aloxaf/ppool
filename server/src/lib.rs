pub mod checker;
pub mod spider;

use ppool_spider::Proxy;
use rand::{seq::SliceRandom, thread_rng};
use std::collections::HashMap;
use std::mem;
use std::sync::{Arc, Mutex};

pub type AProxyPool = Arc<Mutex<ProxyPool>>;

/// 代理池
/// O(1) 的插入时间复杂度
/// O(1) 的随机取时间复杂度
#[derive(Debug, Default)]
pub struct ProxyPool {
    /// 用于随机取
    list: Vec<Proxy>,
    /// 用于去重 & 记录验证失败次数
    map: HashMap<String, i32>,
}

impl ProxyPool {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert(&mut self, proxy: Proxy) {
        let exist = self.map.get(proxy.ip()).is_some();
        if !exist {
            self.map.insert(proxy.ip().to_string(), 0);
            self.list.push(proxy);
        }
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_random(&mut self) -> &Proxy {
        let mut rng = thread_rng();
        self.list.choose(&mut rng).unwrap()
    }

    pub fn get_list(&self) -> &Vec<Proxy> {
        &self.list
    }

    pub fn get_map(&self) -> &HashMap<String, i32> {
        &self.map
    }

    pub fn set_list(&mut self, list: Vec<Proxy>) {
        mem::replace(&mut self.list, list);
    }

    pub fn set_map(&mut self, map: HashMap<String, i32>) {
        mem::replace(&mut self.map, map);
    }
}

impl Extend<Proxy> for ProxyPool {
    fn extend<T: IntoIterator<Item = Proxy>>(&mut self, iter: T) {
        for proxy in iter {
            self.insert(proxy);
        }
    }
}
