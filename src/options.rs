#[derive(StructOpt, Debug)]
#[structopt(name = "ppool")]
pub struct Opt {
    /// 使用自定义配置文件
    #[structopt(short = "c", long, value_name = "FILE")]
    pub config: Option<String>,

    /// 输出默认配置到标准输出
    #[structopt(short = "C", long)]
    pub print_config: bool,

    /// 测试获取指定代理
    #[structopt(short = "t", long, value_name = "NAME")]
    pub test: Option<String>,
}
