use crate::gallery::{Image, ImageGroup};

use anyhow::{anyhow, Context, Result};
use std::{fs, path::PathBuf, process};

use super::{create_parent_directories, to_web_path, Config, RunMode};

pub enum ThumbnailType {
    Small,
    Large,
}

pub fn write_images(image_group: &ImageGroup, config: &Config) -> Result<()> {
    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
    image_group
        .images
        .par_iter()
        .map(|img| render_image(&img, image_group, config).map(|rendered| rendered.write(config)))
        .collect::<Result<Vec<_>>>()?;
    Ok(())
}

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

fn thumbnail_path(
    group: &ImageGroup,
    image: &Image,
    config: &Config,
    thumbnail_type: &ThumbnailType,
) -> Result<PathBuf> {
    Ok(config
        .output_path
        .join(relative_thumbnail_path(group, image, thumbnail_type)?))
}

fn none_if_exists(path: PathBuf) -> Option<PathBuf> {
    if path.exists() {
        None
    } else {
        Some(path)
    }
}

struct ImageFile {
    source_path: PathBuf,
    output_path: Option<PathBuf>,
    thumbnail_path_small: Option<PathBuf>,
    thumbnail_path_large: Option<PathBuf>,
}

fn check_thumbnail_path(
    group: &ImageGroup,
    image: &Image,
    config: &Config,
    thumbnail_type: &ThumbnailType,
) -> Result<Option<PathBuf>> {
    match thumbnail_type {
        // No need to create a large thumbnail if the group doesn't have its own page.
        ThumbnailType::Large if group.markdown_file.is_none() => Ok(None),
        _default => Ok(none_if_exists(thumbnail_path(
            group,
            image,
            config,
            thumbnail_type,
        )?)),
    }
}

fn render_image(image: &Image, group: &ImageGroup, config: &Config) -> Result<ImageFile> {
    Ok({
        ImageFile {
            source_path: image.path.clone(),
            output_path: output_path(group, image, config)?,
            thumbnail_path_small: check_thumbnail_path(
                group,
                image,
                config,
                &ThumbnailType::Small,
            )?,
            thumbnail_path_large: check_thumbnail_path(
                group,
                image,
                config,
                &ThumbnailType::Large,
            )?,
        }
    })
}

impl ImageFile {
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

    fn write_thumbnails(&self, config: &Config) -> Result<()> {
        match &config.run_mode {
            RunMode::Normal => {
                self.write_thumbnail(&self.thumbnail_path_small, "400x", "400x267+0+0")?;
                self.write_thumbnail(&self.thumbnail_path_large, "2000x", "2000x1335+0+0")
            }
            RunMode::DryRun => Ok(()), // Thumbnails are silent in dry-run mode.
        }
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

    fn write(&self, config: &Config) -> Result<()> {
        self.write_image(config)?;
        self.write_thumbnails(config)
    }
}
