/// 配置
#[derive(Debug)]
pub struct Config {
    /// 验证线程数量
    max_worker: u16,
    /// 爬虫线程间隔时间
    spider_interval: u16,
    /// 检查器线程间隔时间
    checker_interval: u16,
    /// 降级所需的连续失败次数
    level_down_fail_times: u8,
    /// 降级的稳定性阈值
    level_down_stability: f32,
    /// 移除所需的连续失败次数
    remove_fail_times: u8,
    /// 移除所需的稳定性阈值
    remove_stability: u8,
    /// 验证代理的 URL
    url_for_check: String,
    /// 验证代理的超时时间
    check_timeout: u8,
}

/// 爬虫配置
#[derive(Debug)]
pub struct SpiderConfig {
    /// 是否启用
    enable: bool,
    /// 爬虫名称
    name: String,
    /// 爬取的 URL 列表
    urls: Vec<String>,
    /// 爬取规则
    rule: SpiderRule,
}

/// 通用爬虫规则
#[derive(Debug)]
pub enum SpiderRule {
    /// 表格类网站的规则(xpath)
    CommonTable {
        /// 定位行的 xpath
        xpath_line: String,
        /// 定位列的 xpath
        xpath_col: String,
        /// IP, 端口, 匿名性, 类型 在列中的序号
        info_index: [usize; 4],
    },
    /// 使用正则的规则
    CommonRegex {
        /// 匹配 ip
        ip: String,
        /// 匹配端口
        port: String,
        /// 匹配匿名程度
        anonymity: String,
        /// 匹配HTTP/HTTPS
        ssl_type: String,
    },
}
