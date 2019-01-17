use crate::error::MyResult;
use crate::utils::*;
use crate::Proxy;
use log::info;

pub const FUNCS: [fn() -> MyResult<Vec<Proxy>>; 5] =
    [get_xicidaili, get_jiangxianli, get_data5u, iphai, ip336];

// TODO: 这个地方应该可以抽象成配置文件了, 函数是多余的

/// 处理比较规整的代理网站
fn table_getter<T: AsRef<str>>(
    // URL 列表
    url_list: &[T],
    // 网站名称, 用于日志
    name: &str,
    // 提取行
    xpath_1: &str,
    // 提取列
    xpath_2: &str,
    // ip, 端口, 匿名性, 类型 所在的位置
    info_pos: [usize; 4],
) -> MyResult<Vec<Proxy>> {
    let mut ret = vec![];
    for url in url_list {
        let html = get_html(url.as_ref())?;
        let (document, eval_xpath) = get_xpath(&html)?;
        let root = document.get_root_element().unwrap();

        let proxy_list = eval_xpath(xpath_1, &root)?;
        for proxy in proxy_list {
            let info = eval_xpath(xpath_2, &proxy)?
                .iter()
                .filter_map(|node| {
                    let s = document.node_to_string(node);
                    let s = s.replace("&#13;", "");
                    let s = s.trim();
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.to_owned())
                    }
                })
                .collect::<Vec<_>>();
            if info.len() < *info_pos.iter().max().unwrap() {
                continue;
            }
            let (ip, port, anonymity, ssl_type) = (
                &info[info_pos[0]],
                &info[info_pos[1]],
                &info[info_pos[2]],
                &info[info_pos[3]],
            );
            info!("{}: [{}, {}, {}, {}]", name, ip, port, anonymity, ssl_type);
            ret.push(Proxy::new(ip, port, anonymity, ssl_type));
        }
    }
    Ok(ret)
}

/// 西刺代理 http://www.xicidaili.com
pub fn get_xicidaili() -> MyResult<Vec<Proxy>> {
    let url_list = ["nn", "nt"]
        .iter()
        .map(|s| (1..=2).map(move |n| format!("https://www.xicidaili.com/{}/{}", s, n)))
        .flatten()
        .collect::<Vec<_>>();
    let ret = table_getter(
        &url_list,
        "西刺代理",
        r#".//table[@id="ip_list"]//tr[position()>1]"#,
        "./td[not(*)]/text()",
        [0, 1, 2, 3],
    )?;
    Ok(ret)
}

/// ProxyIpLib 项目地址: https://github.com/jiangxianli/ProxyIpLib
/// http://ip.jiangxianli.com
pub fn get_jiangxianli() -> MyResult<Vec<Proxy>> {
    let url_list = (1..=2)
        .map(|n| format!("http://ip.jiangxianli.com/?page={}", n))
        .collect::<Vec<_>>();
    let ret = table_getter(
        &url_list,
        "ProxyIpLib",
        ".//table//tr[position()>1]",
        "./td/text()",
        [1, 2, 3, 4],
    )?;
    Ok(ret)
}

/// 无忧代理
pub fn get_data5u() -> MyResult<Vec<Proxy>> {
    let url_list = [
        "http://www.data5u.com/free/gngn/index.shtml",
        "http://www.data5u.com/free/gnpt/index.shtml",
    ];
    let ret = table_getter(
        &url_list,
        "无忧代理",
        r#"//ul[@class="l2"]"#,
        ".//li/text()",
        [0, 1, 2, 3],
    )?;
    Ok(ret)
}

/// ip 海
pub fn iphai() -> MyResult<Vec<Proxy>> {
    let url_list = [
        "http://www.iphai.com/free/ng",
        "http://www.iphai.com/free/wg",
    ];
    let ret = table_getter(
        &url_list,
        "IP海",
        ".//table//tr[position()>1]",
        "./td/text()",
        [0, 1, 2, 3],
    )?;
    Ok(ret)
}

// TODO: 这玩意儿GBK编码, 识别不到匿名类型
///云代理
pub fn ip336() -> MyResult<Vec<Proxy>> {
    use crate::AnonymityLevel;
    let url_list = [
        "http://www.ip3366.net/free/?stype=1",
        // "http://www.ip3366.net/free/?stype=2",
    ];
    let mut ret = table_getter(
        &url_list,
        "云代理",
".//table//tr[position()>1]",
        "./td/text()",
        [0, 1, 2, 3],
    )?;
    for proxy in &mut ret {
        proxy.anonymity = AnonymityLevel::Elite;
    }
    Ok(ret)
}
