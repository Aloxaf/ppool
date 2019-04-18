use serde::Deserialize;

pub const DEFAULT_CONFIG: &'static str = include_str!("config.toml");

// 全员 pub
// 这个地方一开始全部没有 pub
// 然后我就手动加 pub, 加着加着就怀念 Emacs 的宏
// 然后我的 clion 也装了 Emacs 插件, 于是我就试了一下
// 现在我知道 C-x ) 也能关闭当前标签了
// (所以我为什么不用正则呢?
/// 配置
#[derive(Debug, Deserialize)]
pub struct Config {
    /// 管理密码
    pub password: String,
    /// 验证线程
    pub checker: CheckerConfig,
    /// 爬虫线程
    pub spider: SpiderConfig,
}

/// 验证线程配置
#[derive(Debug, Deserialize)]
pub struct CheckerConfig {
    /// 验证线程数量
    pub max_workers: usize,
    /// 间隔时间(secs)
    pub interval: u64,
    /// 进行HTTP验证的URL
    pub url_http: String,
    /// 进行HTTPS验证的URL
    pub url_https: String,
    /// 验证时允许的最大超时时间
    pub timeout: u64,
    /// 进入稳定列表所需最少验证次数
    pub min_cnt_level_up: u32,
    /// 移除所需的最少验证次数
    pub min_cnt_remove: u32,
    /// >=这个验证次数后如果还处于不稳定列表则直接移除
    pub max_cnt_remove: u32,
    /// 对连续失败次数的配置
    pub fail_times: FailTimes,
    /// 对稳定率的配置
    pub stability: Stability,
}

/// 对连续失败次数的配置
#[derive(Debug, Deserialize)]
pub struct FailTimes {
    /// 降级所需
    pub level_down: u8,
    /// 移除所需
    pub remove: u8,
}

/// 对稳定率的配置
#[derive(Debug, Deserialize)]
pub struct Stability {
    /// 升级所需
    pub level_up: f64,
    /// 降级所需
    pub level_down: f64,
    /// 移除所需
    pub remove: f64,
}

/// 爬虫配置
#[derive(Debug, Deserialize)]
pub struct SpiderConfig {
    /// 两轮间隔
    pub interval: u64,
    /// 表格类网站的规则(xpath)
    pub common_table: Vec<CommonTable>,
    /// 使用正则的规则
    pub common_regex: Vec<CommonRegex>,
}

/// 表格类网站的规则(xpath)
#[derive(Debug, Deserialize)]
pub struct CommonTable {
    /// 是否启用
    pub enable: bool,
    /// 爬虫名称
    pub name: String,
    /// 爬取的 URL 列表
    pub urls: Vec<String>,
    /// 定位行的 xpath
    pub xpath_line: String,
    /// 定位列的 xpath
    pub xpath_col: String,
    /// IP, 端口, 匿名性, 类型 在列中的序号
    pub info_index: [usize; 4],
}

/// 使用正则的规则
#[derive(Debug, Deserialize)]
pub struct CommonRegex {
    /// 是否启用
    pub enable: bool,
    /// 爬虫名称
    pub name: String,
    /// 爬取的 URL 列表
    pub urls: Vec<String>,
    /// 匹配 ip
    pub ip: String,
    /// 匹配端口
    pub port: String,
    /// 匹配匿名程度
    pub anonymity: String,
    /// 匹配HTTP/HTTPS
    pub ssl_type: String,
}
