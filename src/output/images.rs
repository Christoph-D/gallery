//! Writes the images and thumbnails that make up the gallery.
use crate::error::PathErrorContext;
use crate::model::{Image, ImageGroup};

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::{fs, process};

use super::Item;
use super::{create_parent_directories, to_web_path, Config, RunMode};

/// Different thumbnail types for different use cases.
///
/// The overview page uses small thumbnails, the image group pages use large thumbnails.
pub(super) enum ThumbnailType {
    Small,
    Large,
}

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

/// Returns the path to the thumbnail image relative to the output base directory.
pub(super) fn relative_thumbnail_path(
    group: &ImageGroup,
    image: &Image,
    thumbnail_type: &ThumbnailType,
) -> Result<PathBuf> {
    let mut suffix = to_web_path(&group.path)?.join(to_web_path(&image.file_name)?);
    // Always use webp for thumbnails to get a reasonable quality.
    suffix.set_extension("webp");
    let size = match thumbnail_type {
        ThumbnailType::Small => "small",
        ThumbnailType::Large => "large",
    };
    Ok(PathBuf::from("thumbnails").join(size).join(&suffix))
}

fn output_path(group: &ImageGroup, image: &Image, config: &Config) -> Result<PathBuf> {
    Ok([
        &config.output_path,
        &to_web_path(&group.path)?,
        &to_web_path(&image.file_name)?,
    ]
    .iter()
    .collect())
}

/// A single image ready to be written to disk.
struct ImageFile {
    input_path: PathBuf,
    output_path: PathBuf,
}

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
        output_path: output_path(group, image, config)?,
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
        _default => Ok(Some(config.output_path.join(relative_thumbnail_path(
            group,
            image,
            thumbnail_type,
        )?))),
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

#[cfg(test)]
mod tests {
    use super::{relative_thumbnail_path, Image, ImageGroup, ThumbnailType};
    use chrono::naive::NaiveDate;
    use std::path::PathBuf;

    fn new_image_group(group_path: &str, image_path: &str) -> ImageGroup {
        let image_path = PathBuf::from(image_path);
        ImageGroup {
            path: PathBuf::from(group_path),
            title: "ignored".to_owned(),
            date: NaiveDate::from_ymd(2021, 01, 01),
            images: vec![Image {
                name: "ignored".to_owned(),
                path: image_path.clone(),
                file_name: PathBuf::from(image_path.file_name().unwrap()),
            }],
            markdown_file: None,
        }
    }

    #[test]
    fn thumbnail_path_simple() {
        let group = new_image_group(
            "2021-01-01 Some group",
            "/some/path/2021-01-01 Some group/Some file.webp",
        );
        let image = group.images.get(0).unwrap();
        assert_eq!(
            relative_thumbnail_path(&group, image, &ThumbnailType::Small).unwrap(),
            PathBuf::from("thumbnails/small/2021-01-01-some-group/some-file.webp")
        );
    }

    #[test]
    fn thumbnail_path_jpeg() {
        let group = new_image_group(
            "2021-01-01 Some group",
            "/some/path/input/2021-01-01 Some group/Some file.jpeg",
        );
        let image = group.images.get(0).unwrap();
        assert_eq!(
            relative_thumbnail_path(&group, image, &ThumbnailType::Small).unwrap(),
            // The thumbnail should be webp even for jpeg source files.
            PathBuf::from("thumbnails/small/2021-01-01-some-group/some-file.webp")
        );
    }

    #[test]
    fn thumbnail_path_large() {
        let group = new_image_group(
            "2021-01-01 Some group",
            "/some/path/2021-01-01 Some group/Some file.webp",
        );
        let image = group.images.get(0).unwrap();
        assert_eq!(
            relative_thumbnail_path(&group, image, &ThumbnailType::Large).unwrap(),
            PathBuf::from("thumbnails/large/2021-01-01-some-group/some-file.webp")
        );
    }
}
