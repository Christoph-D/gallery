//! This module writes the images and thumbnails that make up the gallery.
use crate::model::{Image, ImageGroup};

use anyhow::{anyhow, Context, Result};
use std::{fs, path::PathBuf, process};

use super::{create_parent_directories, to_web_path, Config, RunMode};

/// Different thumbnail types for different use cases.
///
/// The overview page uses small thumbnails, the image group pages use large thumbnails.
pub enum ThumbnailType {
    Small,
    Large,
}

/// An image group ready to be written to disk.
pub struct ImageGroupFiles {
    images: Vec<ImageFile>,
}

impl ImageGroupFiles {
    /// Writes the image group (all images, all thumbnails) to disk.
    ///
    /// This can be a very slow operation for large numbers of images, especially
    /// if many thumbnails need to be created. This function parallelizes its work.
    pub fn write(&self, config: &Config) -> Result<()> {
        use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
        self.images
            .par_iter()
            .map(|img| img.write(config))
            .collect::<Result<Vec<_>>>()?;
        Ok(())
    }
}

/// Prepares an image group for writing.
///
/// This is a fast read-only operation.
/// You need to call [`ImageGroupFiles::write`] to actually write the files to disk.
pub fn render_images(image_group: &ImageGroup, config: &Config) -> Result<ImageGroupFiles> {
    Ok(ImageGroupFiles {
        images: image_group
            .images
            .iter()
            .map(|img| render_image(&img, image_group, config))
            .collect::<Result<Vec<_>>>()?,
    })
}

/// Returns the path to the thumbnail image relative to the output base directory.
pub fn relative_thumbnail_path(
    group: &ImageGroup,
    image: &Image,
    thumbnail_type: &ThumbnailType,
) -> Result<PathBuf> {
    let suffix = to_web_path(&group.path)?.join(to_web_path(&image.file_name)?);
    let size = match thumbnail_type {
        ThumbnailType::Small => "small",
        ThumbnailType::Large => "large",
    };
    Ok(PathBuf::from("thumbnails").join(size).join(&suffix))
}

fn output_path(group: &ImageGroup, image: &Image, config: &Config) -> Result<Option<PathBuf>> {
    Ok(none_if_exists(
        [
            &config.output_path,
            &to_web_path(&group.path)?,
            &to_web_path(&image.file_name)?,
        ]
        .iter()
        .collect(),
    ))
}

/// Returns [`None`] if the path points to a non-existing file. Otherwise, returns the original path.
fn none_if_exists(path: PathBuf) -> Option<PathBuf> {
    if path.exists() {
        None
    } else {
        Some(path)
    }
}

/// A single image ready to be written to disk.
struct ImageFile {
    source_path: PathBuf,
    output_path: Option<PathBuf>,
    thumbnail_path_small: Option<PathBuf>,
    thumbnail_path_large: Option<PathBuf>,
}

/// Prepares a single image for writing.
fn render_image(image: &Image, group: &ImageGroup, config: &Config) -> Result<ImageFile> {
    Ok({
        ImageFile {
            source_path: image.path.clone(),
            output_path: output_path(group, image, config)?,
            thumbnail_path_small: thumbnail_path(group, image, config, &ThumbnailType::Small)?,
            thumbnail_path_large: thumbnail_path(group, image, config, &ThumbnailType::Large)?,
        }
    })
}

/// Returns the full path to the thumbnail image if a thumbnail is needed.
/// Return `None` if no thumbnail is needed for some reason, for example because
/// the thumbnail already exists or because the thumbnail would be unused.
fn thumbnail_path(
    group: &ImageGroup,
    image: &Image,
    config: &Config,
    thumbnail_type: &ThumbnailType,
) -> Result<Option<PathBuf>> {
    match thumbnail_type {
        // No need to create a large thumbnail if the group doesn't have its own page.
        ThumbnailType::Large if group.markdown_file.is_none() => Ok(None),
        _default => {
            Ok(none_if_exists(config.output_path.join(
                relative_thumbnail_path(group, image, thumbnail_type)?,
            )))
        }
    }
}

impl ImageFile {
    /// Writes the image and its thumbnails to disk.
    fn write(&self, config: &Config) -> Result<()> {
        self.write_image(config)?;
        self.write_thumbnails(config)
    }

    fn write_image(&self, config: &Config) -> Result<()> {
        let output_path = match &self.output_path {
            Some(p) => p,
            None => return Ok(()),
        };
        match &config.run_mode {
            RunMode::Normal => {
                create_parent_directories(output_path)?;
                fs::copy(&self.source_path, output_path).with_context(|| {
                    format!(
                        "Failed to copy image \"{}\" -> \"{}\"",
                        self.source_path.to_string_lossy(),
                        output_path.to_string_lossy()
                    )
                })?;
            }
            RunMode::DryRun => {
                println!("Image: \"{}\"", output_path.to_string_lossy());
            }
        }
        Ok(())
    }

    fn write_thumbnails(&self, config: &Config) -> Result<()> {
        match &config.run_mode {
            RunMode::Normal => {
                self.write_thumbnail(&self.thumbnail_path_small, "400x", "400x267+0+0")?;
                self.write_thumbnail(&self.thumbnail_path_large, "2000x", "2000x1335+0+0")
            }
            RunMode::DryRun => Ok(()), // Thumbnails are silent in dry-run mode.
        }
    }

    fn write_thumbnail(
        &self,
        thumbnail_path: &Option<PathBuf>,
        dimensions: &str,
        crop: &str,
    ) -> Result<()> {
        if thumbnail_path.is_none() {
            return Ok(());
        }
        let thumbnail_path = thumbnail_path.as_ref().unwrap();
        super::create_parent_directories(thumbnail_path)?;
        let result = process::Command::new("convert")
            .arg(&self.source_path)
            .args(&[
                "-resize", dimensions, "-gravity", "center", "-crop", crop, "+repage", "-quality",
                "80",
            ])
            .arg(thumbnail_path)
            .output()
            .with_context(|| {
                format!(
                    "Failed to run imagemagick 'convert': \"{}\"",
                    self.source_path.to_string_lossy()
                )
            })?;
        if !result.status.success() {
            return Err(anyhow!(
                "Failed to create thumbnail: \"{}\"",
                self.source_path.to_string_lossy()
            ));
        }
        Ok(())
    }
}
