//! Writes the HTML pages that make up the gallery.
//!
//! Currently, this is
//! * an overview page showing all the images,
//! * one page per image group for image groups with markdown files.
use crate::error::PathErrorContext;
use crate::model::{Gallery, Image, ImageGroup};

use anyhow::{Context, Result};
use serde::Serialize;
use std::{fs, path::PathBuf};

use super::{create_parent_directories, images, to_web_path, Config, RunMode};

/// An HTML file ready to be written to disk.
pub struct HTMLFile {
    content: String,
    output_path: PathBuf,
}

impl HTMLFile {
    /// Writes the HTML file to disk.
    pub fn write(&self, config: &Config) -> Result<()> {
        match &config.run_mode {
            RunMode::Normal => {
                create_parent_directories(&self.output_path)?;
                fs::write(&self.output_path, &self.content)
                    .path_context("Failed to write HTML file", &self.output_path)
            }
            RunMode::DryRun => {
                println!("HTML:  \"{}\"", self.output_path.to_string_lossy());
                Ok(())
            }
        }
    }
}

/// Renders the overview page into an [`HTMLFile`].
///
/// This is a read-only operation.
/// You need to call [`HTMLFile::write`] to actually write the file to disk.
pub fn render_overview_html(
    gallery: &Gallery,
    config: &Config,
    handlebars: &handlebars::Handlebars,
) -> Result<HTMLFile> {
    let data = GalleryData {
        title: config.page_title.clone(),
        footer: config.page_footer.clone(),
        image_groups: gallery
            .image_groups
            .iter()
            .map(|group| {
                ImageGroupData::from_image_group(config, group, &images::ThumbnailType::Small)
            })
            .collect::<Result<Vec<_>>>()?,
    };
    Ok(HTMLFile {
        content: handlebars
            .render("overview", &data)
            .with_context(|| "Failed to render overview HTML page")?,
        output_path: config.output_path.join("index.html"),
    })
}

/// Renders an image group page into an [`HTMLFile`]. This may be [`None`] if no HTML is needed.
///
/// This is a read-only operation.
/// You need to call [`HTMLFile::write`] to actually write the file to disk.
pub fn render_image_group_html(
    image_group: &ImageGroup,
    config: &Config,
    handlebars: &handlebars::Handlebars,
) -> Result<Option<HTMLFile>> {
    if image_group.markdown_file.is_none() {
        return Ok(None);
    }
    let data =
        ImageGroupData::from_image_group(config, image_group, &images::ThumbnailType::Large)?;
    Ok(Some(HTMLFile {
        content: handlebars.render("image_group", &data).with_context(|| {
            format!(
                "Failed to render HTML page for image group \"{}\"",
                image_group.title
            )
        })?,
        output_path: config
            .output_path
            .join(to_web_path(&image_group.path)?)
            .join("index.html"),
    }))
}

/// Used in handlebars templates to describe a gallery.
#[derive(Serialize)]
struct GalleryData {
    title: String,
    footer: Option<String>,
    image_groups: Vec<ImageGroupData>,
}

/// Used in handlebars templates to describe an image group.
#[derive(Serialize)]
struct ImageGroupData {
    title: Option<String>,
    footer: Option<String>,
    date: String,
    markdown_content: Option<String>,
    images: Vec<ImageData>,
    url: String,
}

/// Used in handlebars templates to describe a single image.
#[derive(Serialize)]
struct ImageData {
    file_name: String,
    name: String,
    thumbnail: String,
    anchor: String,
}

impl ImageGroupData {
    fn from_image_group(
        config: &Config,
        image_group: &ImageGroup,
        thumbnail_type: &images::ThumbnailType,
    ) -> Result<ImageGroupData> {
        // Suppress the title if it's redundant.
        let title = if image_group.images.len() == 1
            && image_group.images.get(0).unwrap().name == image_group.title
        {
            None
        } else {
            Some(image_group.title.clone())
        };
        Ok(ImageGroupData {
            title,
            footer: config.page_footer.clone(),
            date: image_group.date.to_string(),
            markdown_content: None,
            images: image_group
                .images
                .iter()
                .map(|image| ImageData::from_image(image, image_group, thumbnail_type))
                .collect::<Result<Vec<_>>>()?,
            url: slug::slugify(image_group.path.to_string_lossy()),
        })
    }
}

impl ImageData {
    fn from_image(
        image: &Image,
        image_group: &ImageGroup,
        thumbnail_type: &images::ThumbnailType,
    ) -> Result<ImageData> {
        Ok(ImageData {
            file_name: to_web_path(&image.file_name)?.to_string_lossy().to_string(),
            name: image
                .file_name
                .file_stem()
                .path_context("Failed to remove file extension", &image.file_name)?
                .to_string_lossy()
                .to_string(),
            thumbnail: images::relative_thumbnail_path(image_group, image, thumbnail_type)?
                .to_string_lossy()
                .to_string(),
            anchor: slug::slugify(&image.name),
        })
    }
}
