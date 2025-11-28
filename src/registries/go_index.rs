use std::fs::File;

use anyhow::anyhow;
use semver::{Version, VersionReq};
use serde::Deserialize;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::dir::Dir;
use crate::toolchain;

#[derive(Debug)]
pub enum Resolution {
    Resolved(String), // 已确定版本
    Unresolved,       // 未确定, 需要进一步确定
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GoIndex {
    pub versions: Vec<String>, // 已发布go版本列表
    pub latest: String,        // 最新稳定版本
    pub secondary: String,     // 次新稳定版本
    pub sha256: String,        // 版本列表的sha256
}

impl GoIndex {
    pub fn read() -> Option<GoIndex> {
        let goup_home = Dir::goup_home().ok()?;
        let index_go = goup_home.index_go();
        if index_go.exists() {
            let file = File::open(index_go).ok()?;
            Some(serde_json::from_reader(file).ok()?)
        } else {
            None
        }
    }
    pub fn write_if_change(index: &GoIndex) -> Result<(), anyhow::Error> {
        let index_go = Dir::goup_home()?.index_go();
        if index_go.exists()
            && let Ok(file) = File::open(&index_go)
            && let Ok(old) = serde_json::from_reader::<_, GoIndex>(file)
            && old.sha256 == index.sha256
        {
            return Ok(());
        }
        let file = File::create(&index_go)?;
        serde_json::to_writer(file, index)?;
        Ok(())
    }
    // 匹配本地版本
    pub fn match_version(&self, ver_req: &VersionReq) -> Option<String> {
        self.versions
            .iter()
            .rev()
            .find_map(|v| {
                toolchain::semantic(v)
                    .ok()
                    .filter(|semver| ver_req.matches(semver))
                    .map(|_| v)
            })
            .map(ToOwned::to_owned)
    }
    // 尝试匹配归档版本
    pub fn try_match_archived_version(
        &self,
        ver_req: &VersionReq,
    ) -> Result<Resolution, anyhow::Error> {
        if self.versions.is_empty() || self.latest.is_empty() || self.secondary.is_empty() {
            return Ok(Resolution::Unresolved);
        }
        if ver_req.comparators.len() != 1 {
            // 先匹配本地版本
            let ver = self.match_version(ver_req);
            let search_type = if let Some(ver) = ver
                && (ver != self.latest || ver != self.secondary)
            {
                Resolution::Resolved(ver)
            } else {
                Resolution::Unresolved
            };
            return Ok(search_type);
        }

        let latest = toolchain::semantic(&self.latest)?;
        let secondary = toolchain::semantic(&self.secondary)?;
        let is_match_archived = toolchain::is_match_archived(&latest, &secondary, ver_req);
        if is_match_archived {
            self.match_version(ver_req)
                .map(Resolution::Resolved)
                .ok_or_else(|| anyhow!("no matching version found!"))
        } else {
            Ok(Resolution::Unresolved)
        }
    }
}

impl From<Vec<String>> for GoIndex {
    fn from(versions: Vec<String>) -> Self {
        let mut version: Vec<(Version, String)> = versions
            .into_iter()
            .filter_map(|v| toolchain::semantic(&v).ok().map(|ver| (ver, v)))
            .collect();
        version.sort_by(|a, b| a.0.cmp(&b.0));

        let mut latest: Option<(u64, u64, String)> = None;
        let mut secondary: Option<String> = None;
        // 反向迭代, 找到最新的稳定版本和次新的稳定版本
        for (ver, raw) in version.iter().rev() {
            if !ver.pre.is_empty() {
                continue; // 跳过 prerelease (rc/beta/alpha)
            }
            let Some(ref cur) = latest else {
                latest = Some((ver.major, ver.minor, raw.clone()));
                continue;
            };
            if ver.major <= cur.0 && ver.minor < cur.1 {
                secondary = Some(raw.clone());
                break;
            }
        }
        let latest = latest.map(|v| v.2).unwrap_or_default();
        let secondary = secondary.unwrap_or_else(|| latest.clone());

        let mut context = Sha256::new();
        let versions = version
            .into_iter()
            .map(|v| {
                context.update(&v.1);
                v.1
            })
            .collect::<Vec<String>>();
        let sha256 = format!("{:x}", context.finalize());
        Self {
            versions,
            latest,
            secondary,
            sha256,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GoIndex;

    #[test]
    fn test_impl_from_trait() {
        {
            let v1 = vec![
                "1.24.0", "1.25.2", "1.24.1", "1.25rc1", "1.25.1", "1.23.2", "1.25.3", "1.24rc1",
                "1.23rc1", "1.23.0", "1.24.2", "1.23.1", "1.25.0",
            ];
            let want = vec![
                "1.23rc1", "1.23.0", "1.23.1", "1.23.2", "1.24rc1", "1.24.0", "1.24.1", "1.24.2",
                "1.25rc1", "1.25.0", "1.25.1", "1.25.2", "1.25.3",
            ];

            let v2 = v1.iter().map(|s| s.to_string()).collect::<Vec<String>>();
            let cgv: GoIndex = v2.into();
            assert_eq!(cgv.versions, want);
            assert_eq!(cgv.latest, "1.25.3");
            assert_eq!(cgv.secondary, "1.24.2");
        }
        {
            let v1 = vec![
                "1.24.0",
                "1.24rc1",
                "1.24.2",
                "1.25rc2",
                "1.25beta2",
                "1.23.1",
                "1.25rc1",
                "1.24.1",
                "1.23rc1",
                "1.23.0",
                "1.25beta1",
            ];
            let want = vec![
                "1.23rc1",
                "1.23.0",
                "1.23.1",
                "1.24rc1",
                "1.24.0",
                "1.24.1",
                "1.24.2",
                "1.25beta1",
                "1.25beta2",
                "1.25rc1",
                "1.25rc2",
            ];
            let v2 = v1.iter().map(|s| s.to_string()).collect::<Vec<String>>();
            let cgv: GoIndex = v2.into();
            assert_eq!(cgv.versions, want);
            assert_eq!(cgv.latest, "1.24.2");
            assert_eq!(cgv.secondary, "1.23.1");
        }
    }
}
