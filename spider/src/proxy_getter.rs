use crate::error::MyResult;
use crate::utils::*;
use crate::Proxy;

/// 西刺代理 http://www.xicidaili.com
pub fn get_xicidaili() -> MyResult<Vec<Proxy>> {
    let mut ret = vec![];

    // nn: 高匿  nt: 透明
    for _type in &["nn", "nt"] {
        for page in 1..=2 {
            let html = get_html(format!("https://www.xicidaili.com/{}/{}", _type, page));
            let (document, eval_xpath) = get_xpath(&html)?;
            let root = document.get_root_element().unwrap();

            let proxy_list = eval_xpath(r#".//table[@id="ip_list"]//tr[position()>1]"#, &root)?;
            for proxy in proxy_list {
                let info = eval_xpath("./td[not(*)]/text()", &proxy)?
                    .iter()
                    .take(4)
                    .map(|node| document.node_to_string(node))
                    .collect::<Vec<_>>();
                assert!(info.len() >= 4);

                // mem::replace 会比 clone 高效吗
                ret.push(Proxy {
                    ip: info[0].clone(),
                    ssl: info[3].clone(),
                    port: info[1].parse::<u16>().expect("failed to parse port"),
                    anonymous: info[2].clone(),
                });
            }
        }
    }
    Ok(ret)
}

/// guobanjia http://ip.jiangxianli.com
pub fn get_jiangxianli() -> MyResult<Vec<Proxy>> {
    let mut ret = vec![];

    for i in 1..=2 {
        let html = get_html(format!("http://ip.jiangxianli.com/?page={}", i));
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

            ret.push(Proxy {
                ip: info[1].clone(),
                ssl: info[3].clone(),
                port: info[2].parse::<u16>().expect("failed to parse port"),
                anonymous: info[4].clone(),
            })
        }
    }
    Ok(ret)
}
