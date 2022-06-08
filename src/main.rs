//! A static site generator for photo galleries.
mod error;
mod input;
mod model;
mod output;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

/// Commandline arguments.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// If set, then don't write any files.
    #[clap(long = "dry_run")]
    dry_run: bool,

    /// The source directory.
    #[clap(long)]
    input: String,

    /// The output directory.
    #[clap(long)]
    output: String,

    /// The top-level page title.
    #[clap(long = "page_title")]
    page_title: String,

    /// An HTML snippet for the page footer.
    #[clap(long)]
    footer: Option<String>,
}

impl Cli {
    fn run_mode(&self) -> output::RunMode {
        if self.dry_run {
            output::RunMode::DryRun
        } else {
            output::RunMode::Normal
        }
    }

    fn output_config(&self) -> output::Config {
        output::Config {
            output_path: PathBuf::from(&self.output),
            run_mode: self.run_mode(),
            page_title: self.page_title.to_owned(),
            page_footer: self.footer.to_owned(),
        }
    }
}

/// Generates a photo gallery based on the provided commandline arguments.
///
/// To use the arguments provided by the system, pass in [`std::env::args_os()`].
fn run_on_args<I, T>(args: I) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let args = Cli::parse_from(args);
    let input_path = PathBuf::from(&args.input);
    let gallery = input::gallery_from_dir(&input_path).with_context(|| "Failed to read gallery")?;
    output::write_files(&gallery, &args.output_config()).with_context(|| "Failed to write gallery")
}

fn main() {
    if let Err(e) = run_on_args(std::env::args_os()) {
        println!("Error: {:?}", e);
    }
}
