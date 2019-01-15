#![cfg(test)]
use ppool_spider::proxy_getter::*;

// TODO: env_logger::init()

#[test]
fn test_xici() {
    let proxies = get_xicidaili().unwrap();
    assert!(proxies.len() > 0);
    assert_ne!(proxies[0].ip(), "");
    assert_ne!(proxies[0].anonymous(), "");
    assert_ne!(proxies[0].ssl(), "");
    println!("{}", proxies.len());
    println!("{:#?}", proxies[0]);
}

#[test]
fn test_jiangxianli() {
    let proxies = get_jiangxianli().unwrap();
    assert!(proxies.len() > 0);
    assert_ne!(proxies[0].ip(), "");
    assert_ne!(proxies[0].anonymous(), "");
    assert_ne!(proxies[0].ssl(), "");
    println!("{}", proxies.len());
    println!("{:#?}", proxies[0]);
}
