//! Writes the gallery to the output directory.
//!
//! Together with its submodules, this module writes everything including images, thumbnails, and HTML files.
mod html;
mod images;

use crate::error::{path_error, PathErrorContext};
use crate::model::Gallery;

use anyhow::Result;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::fs;
use std::path::{Path, PathBuf};

/// Normal or dryrun (read-only) mode.
pub enum RunMode {
    Normal,
    DryRun,
}

/// Configuration options for the output module.
pub struct Config {
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
pub fn write_files(gallery: &Gallery, config: &Config) -> Result<()> {
    let templates = html::make_templates()?;

    // Create work items.
    let mut items = vec![html::render_overview_html(gallery, config, &templates)?];
    for i in &gallery.image_groups {
        items.extend(html::render_image_group_html(&i, config, &templates)?);
        items.extend(images::render_images(&i, config)?);
    }

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

/// Converts a single-element path into something suitable for a URL.
fn to_web_path(path: &Path) -> Result<PathBuf> {
    if path.components().count() != 1 {
        return Err(path_error(
            "Cannot convert multi-component paths into URLs",
            &path,
        ));
    }
    let p = path
        .to_str()
        .path_context("Failed to convert path to UTF-8", &path)?;
    // Keep the file extension intact if one is present.
    match p.rsplit_once('.') {
        Some((path, ext)) => Ok(PathBuf::from(slug::slugify(path) + "." + ext)),
        None => Ok(PathBuf::from(slug::slugify(p))),
    }
}

#[cfg(test)]
mod tests {
    use super::to_web_path;
    use std::path::{Path, PathBuf};

    #[test]
    fn to_web_path_empty_is_error() {
        assert!(to_web_path(Path::new("")).is_err());
    }

    #[test]
    fn to_web_path_simple() {
        assert_eq!(
            to_web_path(Path::new("2021-12-01 Fuji, Japan")).unwrap(),
            PathBuf::from("2021-12-01-fuji-japan")
        );
    }

    #[test]
    fn to_web_path_umlaut_is_removed() {
        assert_eq!(
            to_web_path(Path::new("2021-12-01 Zürich")).unwrap(),
            PathBuf::from("2021-12-01-zurich")
        );
    }

    #[test]
    fn to_web_path_file_extension_remains_intact() {
        assert_eq!(
            to_web_path(Path::new("Fuji, Japan.webp")).unwrap(),
            PathBuf::from("fuji-japan.webp")
        );
    }

    #[test]
    fn to_web_path_multi_component_is_error() {
        assert!(to_web_path(Path::new("2021-12-01 Fuji, Japan/Summit.webp")).is_err());
    }
}
