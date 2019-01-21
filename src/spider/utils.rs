use super::proxy::*;
use super::user_agent;
use crate::config::*;
use failure::format_err;
use libxml::{
    parser::Parser,
    tree::{document::Document, node::Node},
    xpath::Context,
};
use log::*;
use reqwest::{header, Client};
use std::sync::Arc;
use std::time::Duration;

/// 来一份代理
fn get_proxy(ssl_type: &str) -> SpiderResult<reqwest::Proxy> {
    let mut res = reqwest::get(&format!(
        "http://localhost:8000/get?ssl_type={}&anonymity=高匿",
        ssl_type
    ))?;
    let proxy: Proxy = serde_json::from_str(&res.text()?)?;
    info!("获取代理: {}:{}", proxy.ip(), proxy.port());
    let proxy = reqwest::Proxy::all(&format!("http://{}:{}", proxy.ip(), proxy.port())).unwrap();
    Ok(proxy)
}

/// 获取网页
pub fn get_html<S: AsRef<str>>(url: S) -> SpiderResult<String> {
    for i in 0..5 {
        let mut client = Client::builder().timeout(Duration::from_secs(20));
        // 第一次不使用代理
        if i > 0 {
            // 根据 URL 选择代理类型
            let ssl_type = if url.as_ref().contains("https") {
                "HTTPS"
            } else {
                "HTTP"
            };
            match get_proxy(ssl_type) {
                Ok(proxy) => client = client.proxy(proxy),
                Err(err) => error!("获取代理失败: {:?}", err),
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
            Err(err) => error!("get_html: {:?}", err),
        }
    }
    Err(format_err!("访问 {} 失败", url.as_ref()))
}

/// 从 html 生成 document 和 eval_xpath 函数
pub fn get_xpath(
    html: &str,
) -> SpiderResult<(Document, impl Fn(&str, &Node) -> SpiderResult<Vec<Node>>)> {
    // 先解析 HTML
    let parser = Parser::default_html();
    let document = parser
        .parse_string(&html)
        .map_err(|_| format_err!("无法解析 HTML"))?;
    let context = Context::new(&document).map_err(|_| format_err!("Context 初始化失败"))?;

    // 函数用法: eval_xpath(XPATH, 目标节点)
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
#[inline]
pub fn check_proxy(proxy: &Proxy, config: Arc<CheckerConfig>) -> bool {
    let ssl_type = proxy.ssl_type();
    let proxy = reqwest::Proxy::all(&format!("http://{}:{}", proxy.ip(), proxy.port())).unwrap();
    let client = Client::builder()
        .timeout(Duration::from_secs(config.timeout))
        .proxy(proxy)
        .build()
        .expect("无法构建 Client");
    // httpbin 在国外, 应该不能代表国内访问速度
    let res = match ssl_type {
        SslType::HTTPS => client.head(&config.url_https).send(),
        SslType::HTTP => client.head(&config.url_http).send(),
    };
    match res {
        Ok(r) => r.status().is_success(),
        Err(_e) => false,
    }
}
