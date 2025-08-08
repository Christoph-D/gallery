//! Writes the gallery to the output directory.
//!
//! Together with its submodules, this module writes everything including images, thumbnails, and HTML files.
mod html;
mod images;

use crate::config::{Config, GalleryOrder, RunMode};
use crate::error::PathErrorContext;
use crate::model::Gallery;

use anyhow::Result;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::fs;
use std::path::Path;

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
        let mut items = vec![];
        for i in &gallery.image_groups {
            items.extend(html::render_image_group_html(i, config, &templates)?);
            items.extend(images::render_images(i, config)?);
        }
        items
    };

    // Write items in parallel to maximize throughput.
    items
        .into_par_iter()
        .map(|item| item.write(config))
        .collect::<Result<Vec<_>>>()?;

    // The overview has to come last because it depends on the thumbnail images to generate placeholders.
    html::render_overview_html(gallery, config, &templates)?.write(config)?;

    write_static(config)
}

/// Writes static assets such as CSS and Javascript files to disk.
fn write_static(config: &Config) -> Result<()> {
    for (path, content) in [
        (
            "css/bootstrap.min.css",
            include_str!("../templates/bootstrap.min.css"),
        ),
        ("css/style.css", include_str!("../templates/style.css")),
        (
            "css/basicLightbox.min.css",
            include_str!("../templates/basicLightbox.min.css"),
        ),
        (
            "js/bootstrap.bundle.min.js",
            include_str!("../templates/bootstrap.bundle.min.js"),
        ),
        (
            "js/basicLightbox.min.js",
            include_str!("../templates/basicLightbox.min.js"),
        ),
        (
            "js/wheel-zoom.min.js",
            include_str!("../templates/wheel-zoom.min.js"),
        ),
        ("js/lazyload.js", include_str!("../templates/lazyload.js")),
    ] {
        let path = &config.output_path.join(path);
        match config.run_mode {
            RunMode::Normal => {
                create_parent_directories(path)?;
                fs::write(path, content).path_context("Failed to write asset", path)?;
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
        .path_context("Could not determine parent directory", path)?;
    fs::create_dir_all(dir).path_context("Failed to create directory", dir)
}
