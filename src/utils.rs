use crate::error::*;
use crate::Proxy;
use libxml::{
    parser::Parser,
    tree::{document::Document, node::Node},
    xpath::Context,
};
use reqwest::{header, ClientBuilder};
use std::time::Duration;

pub fn get_html<S: AsRef<str>>(url: S) -> String {
    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(10))
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

pub fn verify_proxy(_proxy: &Proxy) -> bool {
    unimplemented!()
}
