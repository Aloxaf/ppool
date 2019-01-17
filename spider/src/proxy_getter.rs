use crate::error::MyResult;
use crate::utils::*;
use crate::Proxy;
use log::info;

pub const FUNCS: [fn() -> MyResult<Vec<Proxy>>; 2] = [get_xicidaili, get_jiangxianli];

/// 西刺代理 http://www.xicidaili.com
pub fn get_xicidaili() -> MyResult<Vec<Proxy>> {
    let mut ret = vec![];

    // nn: 高匿  nt: 透明
    for _type in &["nn", "nt"] {
        for page in 1..=2 {
            let html = get_html(format!("https://www.xicidaili.com/{}/{}", _type, page))?;
            let (document, eval_xpath) = get_xpath(&html)?;
            let root = document.get_root_element().unwrap();

            let proxy_list = eval_xpath(r#".//table[@id="ip_list"]//tr[position()>1]"#, &root)?;
            for proxy in proxy_list {
                let info = eval_xpath("./td[not(*)]/text()", &proxy)?
                    .iter()
                    .filter_map(|node| {
                        // 这家代理有些列会空着, 导致提取数据错误
                        let s = document.node_to_string(node);
                        if s.trim().is_empty() {
                            None
                        } else {
                            Some(s)
                        }
                    })
                    .take(4)
                    .collect::<Vec<_>>();
                assert!(info.len() >= 4);

                info!("xicidaili: get {}:{}", info[0], info[1]);

                ret.push(Proxy::new(&info[0], &info[1], &info[2], &info[3]));
            }
        }
    }
    Ok(ret)
}

/// jiangxianli http://ip.jiangxianli.com
pub fn get_jiangxianli() -> MyResult<Vec<Proxy>> {
    let mut ret = vec![];

    for i in 1..=2 {
        let html = get_html(format!("http://ip.jiangxianli.com/?page={}", i))?;
        let (document, eval_xpath) = get_xpath(&html)?;
        let root = document.get_root_element().unwrap();

        let proxy_list = eval_xpath(".//table//tr[position()>1]", &root)?;
        for proxy in proxy_list {
            let info = eval_xpath("./td/text()", &proxy)?
                .iter()
                .skip(1)
                .take(4)
                .map(|node| document.node_to_string(&node))
                .collect::<Vec<_>>();
            assert!(info.len() >= 4);

            info!("jiangxianli: get {}:{}", info[0], info[1]);

            ret.push(Proxy::new(&info[0], &info[1], &info[2], &info[3]))
        }
    }
    Ok(ret)
}
