use anyhow::Result;
use goup_rs::registries::registry_index::{RegistryIndex, ngx_fancy_index::NgxFancyIndex};

fn main() -> Result<()> {
    // let host = "https://mirrors.ustc.edu.cn/golang/"; // 暂时不能用

    // let items = NgxAutoIndex::new(host).list_upstream_go_versions()?;

    // let host = "https://mirrors.nju.edu.cn/golang";
    // let host = "https://mirrors.hust.edu.cn/golang";
    let host = "https://mirrors.aliyun.com/golang";

    let items: Vec<String> = NgxFancyIndex::new(host).list_upstream_go_versions()?;
    for item in items {
        println!("{}", item);
    }
    let _ = items;
    Ok(())
}
