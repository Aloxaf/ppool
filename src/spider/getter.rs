use super::proxy::*;
use super::utils::*;
use lazy_static::lazy_static;
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
    lazy_static! {
        static ref RE_IP: Regex = Regex::new(r"[0-9.]+").unwrap();
        static ref RE_PORT: Regex = Regex::new(r"\d+").unwrap();
    }

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
                .filter_map(|node| {
                    // 取出内容, 并排除空白列
                    let s = document.node_to_string(node);
                    // 此处应该有更完整的 unescape
                    let s = s.replace("&#13;", "\r");
                    let s = s.trim();
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.to_owned())
                    }
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

            if RE_IP.is_match(ip) && RE_PORT.is_match(port) {
                info!("{}: [{}, {}, {}, {}]", name, ip, port, anonymity, ssl_type);
                ret.push(Proxy::new(ip, port, anonymity, ssl_type));
            } else {
                error!("BAD IP from {}: [{}, {}]", name, ip, port);
            }
        }
    }
    Ok(ret)
}
