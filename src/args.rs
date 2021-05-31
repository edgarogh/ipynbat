use clap::{crate_authors, crate_version, AppSettings, Clap};
use std::path::PathBuf;

#[derive(clap::Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Args {
    /// Path to the Jupyter notebook to display
    pub file: Option<PathBuf>,

    /// Run the notebook and display the computed output instead of the cached output
    /// The notebook is never modified
    #[clap(short, long)]
    pub run: bool,

    /// Override the detected kernel with another one, given its name
    /// Use `--list-kernels` to find all kernels in your system
    #[clap(short, long)]
    pub kernel: Option<String>,

    /// Print a list of installed kernels and exit
    #[clap(long)]
    pub list_kernels: bool,
}

impl Args {
    pub fn parse() -> Self {
        Clap::parse()
    }
}
