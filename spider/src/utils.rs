use crate::error::*;
use crate::Proxy;
use libxml::{
    parser::Parser,
    tree::{document::Document, node::Node},
    xpath::Context,
};
use reqwest::{header, Client};
use std::time::Duration;

/// 获取网页
#[cfg(not(feature = "local"))]
pub fn get_html<S: AsRef<str>>(url: S) -> String {
    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .unwrap();
    let mut res = client.get(url.as_ref())
        .header(header::CONNECTION, "keep-alive")
        .header(header::CACHE_CONTROL, "max-age=0")
        .header(header::UPGRADE_INSECURE_REQUESTS, "1")
        .header(header::USER_AGENT, "Mozilla/5.0 (Windows NT 6.2; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/27.0.1453.94 Safari/537.36")
        .header(header::ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
        .header(header::ACCEPT_ENCODING, "gzip, deflate, sdch")
        .header(header::ACCEPT_LANGUAGE, "zh-CN,zh;q=0.8")
        .send()
        .unwrap();
    res.text().unwrap()
}

/// 从本地加载 html
/// 文件名为 ```url.split('.')[1]```
#[cfg(feature = "local")]
pub fn get_html<S: AsRef<str>>(url: S) -> String {
    use log::debug;
    use std::env;
    use std::fs::File;
    use std::io::Read;

    let mut self_path = env::current_dir().unwrap();
    self_path.pop();
    self_path.extend(&["spider", "tests", "html"]);

    let name = url.as_ref().split('.').skip(1).next().unwrap();
    self_path.push(format!("{}.html", name));
    debug!("read local file {:?}", self_path);
    let mut file = File::open(self_path).unwrap();
    let mut ret = String::new();
    file.read_to_string(&mut ret).unwrap();
    ret
}

/// 从 html 生成 document 和 eval_xpath 函数
pub fn get_xpath(html: &str) -> MyResult<(Document, impl Fn(&str, &Node) -> MyResult<Vec<Node>>)> {
    let parser = Parser::default_html();
    let document = parser.parse_string(&html)?;
    let context = Context::new(&document).map_err(|_| MyError::ContextInit)?;

    let eval_xpath = move |xpath: &str, node: &Node| -> MyResult<Vec<Node>> {
        let v = context
            .node_evaluate(xpath, node)
            .map_err(|_| MyError::XPathEval)?
            .get_nodes_as_vec();
        Ok(v)
    };
    Ok((document, eval_xpath))
}

/// 检测代理可用性
/// TODO: http & https 区分
pub fn verify_proxy(proxy: &Proxy) -> bool {
    let proxy = reqwest::Proxy::http(&format!("http://{}:{}", proxy.ip(), proxy.port()))
        .expect("fail to init proxy");
    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .proxy(proxy)
        .build()
        .expect("failed to build client");
    let res = match client.get("http://httpbin.org/ip").send() {
        Ok(r) => r,
        Err(_) => return false,
    };
    res.status().is_success()
}
