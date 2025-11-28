use anyhow::anyhow;
use reqwest::blocking::Client;
use scraper::{Html, Selector};

use crate::registries::{go_index::GoIndex, registry_index::RegistryIndex};

#[derive(Debug)]
pub struct NgxFancyIndex {
    host: String,
}

impl RegistryIndex for NgxFancyIndex {
    fn get_upstream_latest_go_version(&self) -> Result<String, anyhow::Error> {
        self.inner_list_upstream_go_versions().map(|i| i.latest)
    }
    fn list_upstream_go_versions(&self) -> Result<Vec<String>, anyhow::Error> {
        self.inner_list_upstream_go_versions().map(|i| i.versions)
    }
}

impl NgxFancyIndex {
    pub fn new(host: &str) -> NgxFancyIndex {
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
        let selector =
            Selector::parse("table tbody tr td a").map_err(|e| anyhow!("selector {}", e))?;
        let items: Vec<String> = document
            .select(&selector)
            .filter_map(|element| {
                let filename = element.text().collect::<String>();
                let filename = filename.trim();
                if !filename.starts_with("go")
                    || filename.ends_with('/')
                    || !filename.ends_with(".src.tar.gz")
                {
                    None
                } else {
                    // go1.22.3.src.tar.gz -> go1.22.3
                    let ver = filename
                        .trim_start_matches("go")
                        .trim_end_matches(".src.tar.gz");
                    Some(ver.to_owned())
                }
            })
            .collect();
        Ok(items.into())
    }
}
