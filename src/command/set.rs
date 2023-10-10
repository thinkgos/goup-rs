use clap::Args;
// use ratatui::prelude::CrosstermBackend;
// use ratatui::terminal::Terminal;

use super::{switch_go_version, Run};

#[derive(Args, Debug)]
#[command(disable_version_flag = true)]
pub struct Set {
    /// target go version
    version: Option<String>,
}

impl Run for Set {
    fn run(&self) -> Result<(), anyhow::Error> {
        if let Some(version) = &self.version {
            switch_go_version(version)
        } else {
            // TODO: implement me
            // let mut term = Terminal::new(CrosstermBackend::new(io::stdout()))?;
            // _ = term
            Ok(())
        }
    }
}
