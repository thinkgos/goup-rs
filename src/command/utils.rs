use clap::Args;

use crate::consts;
#[derive(Args, Debug, PartialEq)]
pub struct InstallOptions {
    /// skip sha256 verification.
    #[arg(long)]
    pub skip_verify: bool,
    /// enable check archive size.
    #[arg(long)]
    pub enable_check_archive_size: bool,
    /// registry index that is used to update Go version index.
    #[arg(long, default_value_t = consts::GO_REGISTRY_INDEX.to_owned(), env = consts::GOUP_GO_REGISTRY_INDEX)]
    pub registry_index: String,
    /// registry that is used to download Go archive file.
    #[arg(long, default_value_t = consts::GO_REGISTRY.to_owned(), env = consts::GOUP_GO_REGISTRY)]
    pub registry: String,
}
