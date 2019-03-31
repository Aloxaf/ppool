use super::proxy::*;
use super::utils::*;
use itertools::izip;
use log::{error, info};
use regex::Regex;

/// 处理比较规整的代理网站
pub fn table_getter<T: AsRef<str>>(
    // 网站名称, 用于日志
    name: &str,
    // URL 列表
    url_list: &[T],
    // 提取行
    xpath_1: &str,
    // 提取列
    xpath_2: &str,
    // ip, 端口, 匿名性, 类型 所在的位置
    info_pos: &[usize; 4],
) -> SpiderResult<Vec<Proxy>> {
    let mut ret = vec![];

    for url in url_list {
        // 先请求 html, 并处理获得 eval_xpath 和 根节点
        let html = get_html(url.as_ref())?;
        let (document, eval_xpath) = get_xpath(&html)?;
        let root = document.get_root_element().unwrap();

        // 提取列表的每一行
        let proxy_list = eval_xpath(xpath_1, &root)?;
        for proxy in proxy_list {
            // 提取列表的每一列
            let info = eval_xpath(xpath_2, &proxy)?
                .iter()
                .map(|node| {
                    // 取出内容, 并排除空白列
                    let s = document.node_to_string(node);
                    // 此处应该有更完整的 unescape
                    let s = s.replace("&#13;", "\r");
                    s.trim().to_owned()
                })
                .collect::<Vec<_>>();

            // 如果最终得到的列长度不够, 则放弃这一行
            if info.len() < *info_pos.iter().max().unwrap() {
                continue;
            }

            let (ip, port, anonymity, ssl_type) = (
                &info[info_pos[0]],
                &info[info_pos[1]],
                &info[info_pos[2]],
                &info[info_pos[3]],
            );

            if let Ok(proxy) = Proxy::new(ip, port, anonymity, ssl_type) {
                info!("{}: [{}, {}, {}, {}]", name, ip, port, anonymity, ssl_type);
                ret.push(proxy);
            } else {
                error!("BAD IP from {}: [{}, {}]", name, ip, port);
            }
        }
    }
    Ok(ret)
}

/// 用 正则表达式爬取
pub fn regex_getter<T: AsRef<str>>(
    // 网站名称, 用于日志
    name: &str,
    // URL 列表
    url_list: &[T],
    // 提取 IP
    re_ip: &str,
    // 提取端口
    re_port: &str,
    // 提取匿名程度
    re_anonymity: &str,
    // 提取 SSL 类型
    re_ssl_type: &str,
) -> SpiderResult<Vec<Proxy>> {
    let re_ip = Regex::new(re_ip)?;
    let re_port = Regex::new(re_port)?;
    let re_anonymity = Regex::new(re_anonymity)?;
    let re_ssl_type = Regex::new(re_ssl_type)?;

    let mut ret = vec![];
    for url in url_list {
        let html = get_html(url.as_ref())?;

        for (ip, port, anonymity, ssl_type) in izip!(
            re_ip.captures_iter(&html),
            re_port.captures_iter(&html),
            re_anonymity.captures_iter(&html),
            re_ssl_type.captures_iter(&html)
        ) {
            let ip = &ip[0];
            let port = &port[0];
            let anonymity = &anonymity[0];
            let ssl_type = &ssl_type[0];

            if let Ok(proxy) = Proxy::new(ip, port, anonymity, ssl_type) {
                info!("{}: [{}, {}, {}, {}]", name, ip, port, anonymity, ssl_type);
                ret.push(proxy);
            } else {
                error!("BAD IP from {}: [{}, {}]", name, ip, port);
            }
        }
    }

    Ok(ret)
}
