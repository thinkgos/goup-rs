use anyhow::anyhow;
use reqwest::blocking::Client;
use scraper::{Html, Selector};

use crate::registries::{go_index::GoIndex, registry_index::RegistryIndex};

#[derive(Debug)]
pub struct NgxAutoIndex {
    host: String,
}

impl RegistryIndex for NgxAutoIndex {
    fn get_upstream_latest_go_version(&self) -> Result<String, anyhow::Error> {
        self.inner_list_upstream_go_versions().map(|i| i.latest)
    }
    fn list_upstream_go_versions(&self) -> Result<Vec<String>, anyhow::Error> {
        self.inner_list_upstream_go_versions().map(|i| i.versions)
    }
}

impl NgxAutoIndex {
    pub fn new(host: &str) -> NgxAutoIndex {
        Self {
            host: host.to_owned(),
        }
    }
    fn inner_list_upstream_go_versions(&self) -> Result<GoIndex, anyhow::Error> {
        let resp = Client::new()
            .get(&self.host)
            .header("User-Agent", env!("CARGO_PKG_VERSION"))
            .send()?;
        if !resp.status().is_success() {
            return Err(anyhow!(
                "{} unreachable, status {}",
                self.host,
                resp.status()
            ));
        }
        let text = resp.text()?;
        let document = Html::parse_document(&text);
        let selector = Selector::parse("pre a").map_err(|e| anyhow!("selector {}", e))?;
        let items: Vec<String> = document
            .select(&selector)
            .filter_map(|element| {
                let href = element.value().attr("href")?;
                let href = href.trim_start_matches("/golang/");
                if !href.starts_with("go") || href.ends_with('/') || !href.ends_with(".src.tar.gz")
                {
                    return None;
                }
                // go1.22.3.src.tar.gz -> go1.22.3
                let ver = href
                    .trim_start_matches("/golang/")
                    .trim_start_matches("go")
                    .trim_end_matches(".src.tar.gz");
                Some(ver.to_owned())
            })
            .collect();
        Ok(items.into())
    }
}
