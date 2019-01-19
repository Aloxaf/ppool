use super::proxy::*;
use super::utils::*;
use log::info;

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
