//! Writes the images and thumbnails that make up the gallery.
use super::Item;
use super::{create_parent_directories, Config, RunMode};

use crate::error::PathErrorContext;
use crate::model::{Image, ImageGroup, ThumbnailType};

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::{fs, process};

/// Prepares an image group for writing.
pub(super) fn render_images(
    image_group: &ImageGroup,
    config: &Config,
) -> Result<Vec<Box<dyn Item + Send>>> {
    let mut res = Vec::new();
    for img in &image_group.images {
        res.extend(render_image(&img, image_group, config)?);
    }
    Ok(res)
}

/// A single image ready to be written to disk.
struct ImageFile {
    input_path: PathBuf,
    output_path: PathBuf,
}

/// A single thumbnail ready to be written to disk.
struct ThumbnailFile {
    input_path: PathBuf,
    output_path: PathBuf,
    thumbnail_type: ThumbnailType,
}

/// Prepares a single image for writing.
/// Returns a work item for the image itself and possibly some for the corresponding thumbnails.
fn render_image(
    image: &Image,
    group: &ImageGroup,
    config: &Config,
) -> Result<Vec<Box<dyn Item + Send>>> {
    let mut res: Vec<Box<dyn Item + Send>> = vec![Box::new(ImageFile {
        input_path: image.path.clone(),
        output_path: config.output_path.join(image.url(group)?),
    })];
    for t in [ThumbnailType::Small, ThumbnailType::Large] {
        if let Some(p) = thumbnail_path(group, image, config, &t)? {
            res.push(Box::new(ThumbnailFile {
                input_path: image.path.clone(),
                output_path: p.clone(),
                thumbnail_type: t,
            }))
        }
    }
    Ok(res)
}

/// Returns the full path to the thumbnail image if a thumbnail is needed.
/// Return `None` if no thumbnail is needed, for example because the thumbnail would be unused.
fn thumbnail_path(
    group: &ImageGroup,
    image: &Image,
    config: &Config,
    thumbnail_type: &ThumbnailType,
) -> Result<Option<PathBuf>> {
    match thumbnail_type {
        // No need to create a large thumbnail if the group doesn't have its own page.
        ThumbnailType::Large if group.markdown_file.is_none() => Ok(None),
        _default => Ok(Some(
            config
                .output_path
                .join(image.thumbnail_url(group, thumbnail_type)?),
        )),
    }
}

/// Returns true if the output is stale and needs to be rewritten.
fn needs_update(input_path: &Path, output_path: &Path) -> bool {
    let res = || -> Result<bool> {
        let output_modified = output_path.metadata()?.modified()?;
        let input_modified = input_path.metadata()?.modified()?;
        // Needs update if the output is older than the input.
        Ok(output_modified < input_modified)
    };
    res().unwrap_or(true)
}

impl Item for ImageFile {
    fn write(&self, config: &Config) -> Result<()> {
        if !needs_update(&self.input_path, &self.output_path) {
            return Ok(());
        }
        match &config.run_mode {
            RunMode::Normal => {
                create_parent_directories(&self.output_path)?;
                fs::copy(&self.input_path, &self.output_path).path_context(
                    &format!(
                        "Failed to copy image to \"{}\"",
                        self.output_path.to_string_lossy()
                    ),
                    &self.input_path,
                )?;
            }
            RunMode::DryRun => {
                println!("Image: \"{}\"", self.output_path.to_string_lossy());
            }
        }
        Ok(())
    }
}

impl Item for ThumbnailFile {
    fn write(&self, config: &Config) -> Result<()> {
        if !needs_update(&self.input_path, &self.output_path) {
            return Ok(());
        }
        match &config.run_mode {
            RunMode::Normal => match self.thumbnail_type {
                ThumbnailType::Small => self.write_internal("400x", "400x267+0+0"),
                ThumbnailType::Large => self.write_internal("2000x", "2000x1335+0+0"),
            },
            RunMode::DryRun => Ok(()), // Thumbnails are silent in dry-run mode.
        }
    }
}

impl ThumbnailFile {
    fn write_internal(&self, dimensions: &str, crop: &str) -> Result<()> {
        super::create_parent_directories(&self.output_path)?;
        let result = process::Command::new("convert")
            .arg(&self.input_path)
            .args(&[
                "-resize", dimensions, "-gravity", "center", "-crop", crop, "+repage", "-quality",
                "80",
            ])
            .arg(&self.output_path)
            .output()
            .path_context("Failed to run imagemagick 'convert'", &self.input_path)?;
        if !result.status.success() {
            return Err(anyhow!(
                "Failed to create thumbnail: \"{}\"\nstderr:\n{}\n\nstdout:\n{}\n",
                self.input_path.to_string_lossy(),
                String::from_utf8_lossy(&result.stderr),
                String::from_utf8_lossy(&result.stdout),
            ));
        }
        Ok(())
    }
}
