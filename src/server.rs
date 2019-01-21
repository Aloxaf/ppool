use crate::proxy_pool::*;
use rocket::{get, routes, State};

#[get("/")]
fn index(_state: State<AProxyPool>) -> &'static str {
    r#"{
  "get?<ssl_type:str>&<anonymity:str>&<stability:f32>": "随机获取一个代理, 无特殊需求请勿增加参数, 速度较慢",
  "get_all?<ssl_type:str>&<anonymity:str>&<stability:f32>": "获取所有可用代理",
  "get_status": "获取代理池信息",
}"#
}

#[get("/get_status")]
fn get_status(state: State<AProxyPool>) -> String {
    let proxy_pool = state.lock().unwrap();
    let stable_cnt = proxy_pool.get_stable().len();
    let unstable_cnt = proxy_pool.get_unstable().len();
    format!(
        r#"{{
  "total": {},
  "stable": {},
  "unstable": {},
}}"#,
        stable_cnt + unstable_cnt,
        stable_cnt,
        unstable_cnt
    )
}

// TODO: 提前搞个类型转换
#[get("/get?<ssl_type>&<anonymity>&<stability>")]
fn get_single(
    state: State<AProxyPool>,
    ssl_type: Option<String>,
    anonymity: Option<String>,
    stability: Option<f32>,
) -> String {
    let mut proxy_pool = state.lock().unwrap();

    // 啥参数都没有, 直接调用 get_random, O(1) 时间复杂度
    let proxy = if ssl_type.is_none() && anonymity.is_none() && stability.is_none() {
        proxy_pool.get_random()
    // 有参数的话, 使用 O(n) 复杂度的 select_random
    } else {
        proxy_pool.select_random(ssl_type, anonymity, stability)
    };
    // None 会被序列化为 null, Some 会被忽略, 非常棒棒
    serde_json::to_string(&proxy).unwrap()
}

#[get("/get_all?<ssl_type>&<anonymity>&<stability>")]
fn get_all(
    state: State<AProxyPool>,
    ssl_type: Option<String>,
    anonymity: Option<String>,
    stability: Option<f32>,
) -> String {
    let proxy_pool = state.lock().unwrap();
    // get_stable 返回 &Vec<T>, select 返回 Vec<&T>, 所以这个地方无法简化成 get_single 的逻辑
    if ssl_type.is_none() && anonymity.is_none() && stability.is_none() {
        let proxy = proxy_pool.get_stable();
        serde_json::to_string_pretty(proxy).unwrap()
    } else {
        let proxy = proxy_pool.select(ssl_type, anonymity, stability);
        serde_json::to_string_pretty(&proxy).unwrap()
    }
}

// TODO: del api
// 其实并不想增加这个 API, 感觉没啥用...还增加复杂度

// TODO: reload api
// 修改配置文件后不用 kill 进程

/// 火箭发射!
pub fn launch_rocket(proxy_pool: AProxyPool) {
    rocket::ignite()
        .mount("/", routes![index, get_status, get_single, get_all])
        .manage(proxy_pool)
        .launch();
}
