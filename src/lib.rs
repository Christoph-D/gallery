//! A static site generator for photo galleries.

mod input;
mod model;
mod output;

use anyhow::{Context, Result};
use clap::{App, Arg};
use std::path::PathBuf;

/// Generates a photo gallery based on the provided commandline arguments.
///
/// To use the arguments provided by the system, pass in [`std::env::args_os()`].
pub fn run_on_args<I, T>(args: I) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let matches = App::new("Gallery")
        .arg(
            Arg::with_name("dry_run")
                .long("dry_run")
                .help("If set, then don't write any files."),
        )
        .arg(
            Arg::with_name("input")
                .long("input")
                .takes_value(true)
                .required(true)
                .help("The source directory."),
        )
        .arg(
            Arg::with_name("output")
                .long("output")
                .takes_value(true)
                .required(true)
                .help("The output directory."),
        )
        .arg(
            Arg::with_name("page_title")
                .long("page_title")
                .takes_value(true)
                .required(true)
                .help("The top-level page title."),
        )
        .arg(
            Arg::with_name("footer")
                .long("footer")
                .takes_value(true)
                .help("An HTML snippet for the page footer."),
        )
        .get_matches_from(args);

    let input_path = PathBuf::from(matches.value_of("input").unwrap());
    let gallery = input::gallery_from_dir(&input_path).with_context(|| "Failed to read gallery")?;

    output::write_files(
        &gallery,
        &output::Config {
            output_path: PathBuf::from(matches.value_of("output").unwrap()),
            run_mode: if matches.is_present("dry_run") {
                output::RunMode::DryRun
            } else {
                output::RunMode::Normal
            },
            page_title: matches.value_of("page_title").unwrap().to_string(),
            page_footer: matches.value_of("footer").map(|s| s.to_string()),
        },
    )
    .with_context(|| "Failed to write gallery")
}
