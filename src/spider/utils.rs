use super::proxy::*;
use super::user_agent;
use failure::format_err;
use libxml::{
    parser::Parser,
    tree::{document::Document, node::Node},
    xpath::Context,
};
use log::{debug, error};
use reqwest::{header, Client};
use std::time::Duration;

/// 获取代理
fn get_proxy(ssl_type: &str) -> Option<reqwest::Proxy> {
    let mut res = reqwest::get(&format!(
        "http://localhost:8000/get?ssl_type={}&anonymity=高匿",
        ssl_type
    ))
    .unwrap();
    let proxy: Proxy = match serde_json::from_str(&res.text().unwrap()) {
        Ok(v) => v,
        Err(_) => return None,
    };
    debug!("获取代理: {}:{}", proxy.ip(), proxy.port());
    let proxy = reqwest::Proxy::all(&format!("http://{}:{}", proxy.ip(), proxy.port()))
        .expect("build proxy error");
    Some(proxy)
}

/// 获取网页
#[cfg(not(feature = "local"))]
pub fn get_html<S: AsRef<str>>(url: S) -> SpiderResult<String> {
    for i in 0..5 {
        let mut client = Client::builder().timeout(Duration::from_secs(20));
        // 第一次不使用代理
        if i > 0 {
            let ssl_type = if url.as_ref().contains("https") {
                "HTTPS"
            } else {
                "HTTP"
            };
            if let Some(proxy) = get_proxy(ssl_type) {
                client = client.proxy(proxy)
            } else {
                // 没有代理的话, 再尝试也没用了, 直接退出
                break;
            }
        }
        let client = client.build()?;
        let res = client
            .get(url.as_ref())
            .header(header::CONNECTION, "keep-alive")
            .header(header::CACHE_CONTROL, "max-age=0")
            .header(header::UPGRADE_INSECURE_REQUESTS, "1")
            .header(header::USER_AGENT, user_agent::random())
            .header(
                header::ACCEPT,
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
            )
            .header(header::ACCEPT_ENCODING, "gzip, deflate, sdch")
            .header(header::ACCEPT_LANGUAGE, "zh-CN,zh;q=0.8")
            .send();
        match res {
            Ok(mut res) => {
                if res.status().is_success() {
                    return Ok(res.text()?);
                } else {
                    error!("get_html: {}", res.status());
                }
            }
            Err(e) => error!("get_html: {:?}", e),
        }
    }
    Err(format_err!("访问 {} 失败", url.as_ref()))
}

/// 从本地加载 html
/// 文件名为 ```url.split('.')[1]```
#[cfg(feature = "local")]
pub fn get_html<S: AsRef<str>>(url: S) -> SpiderResult<String> {
    use std::env;
    use std::fs::File;
    use std::io::Read;

    let mut self_path = env::current_dir().unwrap();
    self_path.pop();
    self_path.extend(&["spider", "tests", "html"]);

    let name = url.as_ref().split('.').skip(1).next().unwrap();
    self_path.push(format!("{}.html", name));
    debug!("读取本地文件 {:?}", self_path);
    let mut file = File::open(self_path).unwrap();
    let mut ret = String::new();
    file.read_to_string(&mut ret).unwrap();
    Ok(ret)
}

/// 从 html 生成 document 和 eval_xpath 函数
pub fn get_xpath(
    html: &str,
) -> SpiderResult<(Document, impl Fn(&str, &Node) -> SpiderResult<Vec<Node>>)> {
    let parser = Parser::default_html();
    let document = parser
        .parse_string(&html)
        .map_err(|_| format_err!("无法解析 HTML"))?;
    let context = Context::new(&document).map_err(|_| format_err!("Context 初始化失败"))?;

    let eval_xpath = move |xpath: &str, node: &Node| -> SpiderResult<Vec<Node>> {
        let v = context
            .node_evaluate(xpath, node)
            .map_err(|_| format_err!("XPath 执行失败"))?
            .get_nodes_as_vec();
        Ok(v)
    };
    Ok((document, eval_xpath))
}

/// 检测代理可用性
pub fn check_proxy(proxy: &Proxy) -> bool {
    let ssl_type = proxy.ssl_type();
    let proxy = reqwest::Proxy::all(&format!("http://{}:{}", proxy.ip(), proxy.port()))
        .expect("无法初始化代理");
    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .proxy(proxy)
        .build()
        .expect("无法构建 Client");
    // httpbin 在国外, 应该不能代表国内访问速度
    let res = match ssl_type {
        SslType::HTTPS => client.head("https://www.baidu.com/").send(),
        SslType::HTTP => client.head("http://www.baidu.com/").send(),
    };
    match res {
        Ok(r) => r.status().is_success(),
        Err(_e) => false,
    }
}
