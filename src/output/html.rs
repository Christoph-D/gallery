//! Writes the HTML pages that make up the gallery.
//!
//! Currently, this is
//! * an overview page showing all the images,
//! * one page per image group for image groups with markdown files.
mod markdown;

use super::{create_parent_directories, Config, GalleryOrder, Item, RunMode};

use crate::error::{path_error, PathErrorContext};
use crate::model::{Gallery, Image, ImageGroup, ThumbnailType};

use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

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
        image_groups: image_groups_sorted(gallery, config)?,
    };
    Ok(Box::new(HTMLFile {
        content: templates
            .0
            .render("overview", &data)
            .with_context(|| "Failed to render overview HTML page")?,
        output_path: config.output_path.join("index.html"),
    }))
}

fn image_groups_sorted(gallery: &Gallery, config: &Config) -> Result<Vec<ImageGroupData>> {
    let mut image_groups = gallery
        .image_groups
        .iter()
        .map(|group| ImageGroupData::from_image_group(config, group, &ThumbnailType::Small))
        .collect::<Result<Vec<_>>>()?;
    image_groups.sort_by(|lhs, rhs| {
        if lhs.date != rhs.date {
            match config.order {
                GalleryOrder::OldestFirst => lhs.date.cmp(&rhs.date),
                GalleryOrder::MostRecentFirst => rhs.date.cmp(&lhs.date),
            }
        } else {
            lhs.title.cmp(&rhs.title)
        }
    });
    Ok(image_groups)
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
        output_path: config.output_path.join(image_group.url()?),
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
    url: String,
    name: String,
    thumbnail: String,
    anchor: String,
}

impl ImageGroupData {
    // Adds markdown content and reorders the images to match the markdown content.
    fn add_markdown(self, markdown_file: &Option<PathBuf>) -> Result<Self> {
        let Some(markdown_file) = markdown_file else {
            return Ok(self);
        };
        let res = markdown::to_html(markdown_file, self.images)?;
        Ok(Self {
            markdown_content: Some(res.html.clone()),
            images: res.images_seen,
            ..self
        })
    }

    fn from_image_group(
        config: &Config,
        image_group: &ImageGroup,
        thumbnail_type: &ThumbnailType,
    ) -> Result<ImageGroupData> {
        // Suppress the title if it's redundant.
        let title =
            if image_group.images.len() == 1 && image_group.images[0].name == image_group.title {
                None
            } else {
                Some(image_group.title.clone())
            };
        let images = image_group
            .images
            .iter()
            .map(|image| ImageData::from_image(image, image_group, thumbnail_type))
            .collect::<Result<Vec<_>>>()?;
        let data = ImageGroupData {
            title,
            footer: config.page_footer.clone(),
            date: image_group.date.to_string(),
            markdown_content: None,
            images,
            url: url_to_string(&image_group.url()?)?,
        };
        data.add_markdown(&image_group.markdown_file)
    }
}

impl ImageData {
    fn from_image(
        image: &Image,
        image_group: &ImageGroup,
        thumbnail_type: &ThumbnailType,
    ) -> Result<ImageData> {
        Ok(ImageData {
            url: url_to_string(&image_group.image_url(image)?)?,
            name: image.name.clone(),
            thumbnail: url_to_string(&image_group.thumbnail_url(image, thumbnail_type)?)?,
            anchor: slug::slugify(&image.name),
        })
    }
}

/// Converts a URL from path form into a string.
/// The path components will be joined by slashes.
fn url_to_string(url: &Path) -> Result<String> {
    Ok(url
        .iter()
        .map(|c| c.to_str())
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| path_error("Failed to decode UTF-8", url))?
        .join("/"))
}

#[cfg(test)]
mod tests {
    use super::url_to_string;
    use std::path::PathBuf;

    #[test]
    fn test_url() {
        assert_eq!(url_to_string(&PathBuf::from("foo")).unwrap(), "foo");
    }

    #[test]
    fn test_composite_url() {
        assert_eq!(
            url_to_string(&PathBuf::from("foo").join("bar")).unwrap(),
            "foo/bar"
        );
    }
}
