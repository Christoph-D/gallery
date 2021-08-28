//! Writes the gallery to the output directory.
//!
//! Together with its submodules, this module writes everything including images, thumbnails, and HTML files.
mod html;
mod images;

use crate::error::PathErrorContext;
use crate::model::Gallery;

use anyhow::Result;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::fs;
use std::path::{Path, PathBuf};

/// Normal or dryrun (read-only) mode.
pub(crate) enum RunMode {
    Normal,
    DryRun,
}

/// Configuration options for the output module.
pub(crate) struct Config {
    /// The target directory where to write the gallery.
    pub output_path: PathBuf,
    /// Normal or dryrun (read-only) mode.
    pub run_mode: RunMode,
    /// The top-level title of the generated gallery.
    pub page_title: String,
    /// An optional footer to show (for example) a copyright notice.
    pub page_footer: Option<String>,
}

/// A work item containing something to be written to disk.
trait Item {
    /// Writes the item to disk.
    fn write(&self, config: &Config) -> Result<()>;
}

/// Writes the gallery to disk.
pub(crate) fn write_files(gallery: &Gallery, config: &Config) -> Result<()> {
    let templates = html::make_templates()?;

    // Create work items.
    let items = {
        let mut items = vec![html::render_overview_html(gallery, config, &templates)?];
        for i in &gallery.image_groups {
            items.extend(html::render_image_group_html(&i, config, &templates)?);
            items.extend(images::render_images(&i, config)?);
        }
        items
    };

    // Write items in parallel to maximize throughput.
    items
        .into_par_iter()
        .map(|item| item.write(config))
        .collect::<Result<Vec<_>>>()?;

    write_static(config)
}

/// Writes static assets such as CSS and Javascript files to disk.
fn write_static(config: &Config) -> Result<()> {
    let css_path = config.output_path.join("css").join("bootstrap.min.css");
    let custom_css_path = config.output_path.join("css").join("style.css");
    let js_path = config
        .output_path
        .join("js")
        .join("bootstrap.bundle.min.js");
    for (path, content) in [
        (&css_path, include_str!("../templates/bootstrap.min.css")),
        (&custom_css_path, include_str!("../templates/style.css")),
        (
            &js_path,
            include_str!("../templates/bootstrap.bundle.min.js"),
        ),
    ] {
        match config.run_mode {
            RunMode::Normal => {
                create_parent_directories(path)?;
                fs::write(path, content).path_context("Failed to write asset", &path)?;
            }
            RunMode::DryRun => {
                println!("Static: \"{}\"", path.to_string_lossy());
            }
        }
    }
    Ok(())
}

/// Takes a path to a file and creates all parent directories.
///
/// Differences to [`fs::create_dir_all`]:
/// * This function skips the last element of the path, assuming it's a file name.
/// * This function returns more descriptive errors of the right type.
fn create_parent_directories(path: &Path) -> Result<()> {
    let dir = path
        .parent()
        .path_context("Could not determine parent directory", &path)?;
    fs::create_dir_all(dir).path_context("Failed to create directory", &dir)
}
