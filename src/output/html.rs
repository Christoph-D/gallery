use crate::gallery::{Gallery, Image, ImageGroup};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use std::{fs, path::PathBuf};

use super::{create_parent_directories, images, to_web_path, Config, RunMode};

pub struct HTMLFile {
    content: String,
    output_path: PathBuf,
}

impl HTMLFile {
    pub fn write(&self, config: &Config) -> Result<()> {
        match &config.run_mode {
            RunMode::Normal => {
                create_parent_directories(&self.output_path)?;
                fs::write(&self.output_path, &self.content).with_context(|| {
                    format!(
                        "Failed to write HTML file: \"{}\"",
                        self.output_path.to_string_lossy()
                    )
                })
            }
            RunMode::DryRun => {
                println!("HTML:  \"{}\"", self.output_path.to_string_lossy());
                Ok(())
            }
        }
    }
}

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

#[derive(Serialize)]
struct GalleryData {
    title: String,
    footer: Option<String>,
    image_groups: Vec<ImageGroupData>,
}

#[derive(Serialize)]
struct ImageGroupData {
    title: String,
    footer: Option<String>,
    date: String,
    markdown_content: Option<String>,
    images: Vec<ImageData>,
    url: String,
}

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
        Ok(ImageGroupData {
            title: image_group.title.clone(),
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
                .ok_or_else(|| {
                    anyhow!(
                        "Failed to remove file extension: \"{}\"",
                        image.file_name.to_string_lossy()
                    )
                })?
                .to_string_lossy()
                .to_string(),
            thumbnail: images::relative_thumbnail_path(image_group, image, thumbnail_type)?
                .to_string_lossy()
                .to_string(),
            anchor: slug::slugify(&image.name),
        })
    }
}