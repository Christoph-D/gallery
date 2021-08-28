//! Writes the HTML pages that make up the gallery.
//!
//! Currently, this is
//! * an overview page showing all the images,
//! * one page per image group for image groups with markdown files.
use super::{create_parent_directories, Config, Item, RunMode};

use crate::error::PathErrorContext;
use crate::model::{Gallery, Image, ImageGroup, ThumbnailType};

use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde::Serialize;
use std::{fs, path::PathBuf};

pub(super) struct Templates<'a>(Handlebars<'a>);

pub(super) fn make_templates<'a>() -> Result<Templates<'a>> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars.register_template_string(
        "overview",
        include_str!("../../templates/overview.handlebars"),
    )?;
    handlebars.register_template_string(
        "image_group",
        include_str!("../../templates/image_group.handlebars"),
    )?;
    Ok(Templates(handlebars))
}

/// Renders the overview page into an [`Item`].
pub(super) fn render_overview_html(
    gallery: &Gallery,
    config: &Config,
    templates: &Templates,
) -> Result<Box<dyn Item + Send>> {
    let data = GalleryData {
        title: config.page_title.clone(),
        footer: config.page_footer.clone(),
        image_groups: gallery
            .image_groups
            .iter()
            .map(|group| ImageGroupData::from_image_group(config, group, &ThumbnailType::Small))
            .collect::<Result<Vec<_>>>()?,
    };
    Ok(Box::new(HTMLFile {
        content: templates
            .0
            .render("overview", &data)
            .with_context(|| "Failed to render overview HTML page")?,
        output_path: config.output_path.join("index.html"),
    }))
}

/// Renders an image group page into an [`Item`]. This may be [`None`] if no HTML is needed.
pub(super) fn render_image_group_html(
    image_group: &ImageGroup,
    config: &Config,
    templates: &Templates,
) -> Result<Option<Box<dyn Item + Send>>> {
    if image_group.markdown_file.is_none() {
        return Ok(None);
    }
    let data = ImageGroupData::from_image_group(config, image_group, &ThumbnailType::Large)?;
    Ok(Some(Box::new(HTMLFile {
        content: templates.0.render("image_group", &data).with_context(|| {
            format!(
                "Failed to render HTML page for image group \"{}\"",
                image_group.title
            )
        })?,
        output_path: config
            .output_path
            .join(image_group.url()?)
            .join("index.html"),
    })))
}

/// An HTML file ready to be written to disk.
struct HTMLFile {
    content: String,
    output_path: PathBuf,
}

impl Item for HTMLFile {
    /// Writes the HTML file to disk.
    fn write(&self, config: &Config) -> Result<()> {
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
        thumbnail_type: &ThumbnailType,
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
            // to_string_lossy is safe because URLs are guaranteed to be ASCII.
            url: image_group.url()?.to_string_lossy().into_owned(),
        })
    }
}

impl ImageData {
    fn from_image(
        image: &Image,
        image_group: &ImageGroup,
        thumbnail_type: &ThumbnailType,
    ) -> Result<ImageData> {
        Ok(ImageData {
            // to_string_lossy is safe because URLs are guaranteed to be ASCII.
            file_name: image.url_file_name()?.to_string_lossy().into_owned(),
            name: image.name.clone(),
            thumbnail: image
                .thumbnail_url(image_group, thumbnail_type)?
                .to_string_lossy()
                .into_owned(),
            anchor: slug::slugify(&image.name),
        })
    }
}
