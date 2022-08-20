//! Markdown parsing and output.
//!
//! Markdown syntax extended with image references.
//! To reference for example a file named "My image.webp", write:
//!
//! ```markdown
//! Some text.
//!
//! !image My image
//!
//! Some more text.
//! ```
use super::ImageData;

use crate::error::PathErrorContext;

use anyhow::{anyhow, Result};
use pulldown_cmark::{html, Event, Parser};
use std::collections::{HashMap, HashSet};
use std::{fs, path::Path};

// The output of markdown rendering.
pub(super) struct Rendered {
    // The HTML output.
    pub html: String,
    // The images in order of appearance in the markdown file.
    pub images_seen: Vec<ImageData>,
}

// Returns a formatted error message containing a list of image names.
fn markdown_image_error<T>(message: &str, images: Vec<String>, input_file: &Path) -> Result<T> {
    Err(anyhow!(
        "{}: {}",
        message,
        images
            .into_iter()
            .map(|img| format!("\"{}\"", img))
            .collect::<Vec<_>>()
            .join(", ")
    ))
    .path_context("Error in markdown file", input_file)
}

#[derive(Default)]
struct ImageStatistics {
    // Images that are referenced in the markdown file in their order of appearance.
    // May contain duplicates.
    seen: Vec<String>,
    // Unknown images in the markdown file.
    unknown: Vec<String>,
}

impl ImageStatistics {
    // Images which exist as files but are missing from the the markdown file.
    fn missing(&self, images: &[ImageData]) -> Vec<String> {
        let images_seen_set = HashSet::<String>::from_iter(self.seen.iter().cloned());
        let mut missing = Vec::new();
        for image in images {
            if !images_seen_set.contains(&image.name) {
                missing.push(image.name.to_owned());
            }
        }
        missing
    }
}

pub(super) fn to_html(input_file: &Path, images: Vec<ImageData>) -> Result<Rendered> {
    let input = fs::read_to_string(input_file)
        .path_context("Failed to open image group markdown file", input_file)?;

    let (html, stats) = {
        let mut stats = ImageStatistics::default();
        let iter = Parser::new(&input).map(|e| map_image_event(&e, &images, &mut stats));
        let mut out = String::new();
        html::push_html(&mut out, iter);
        (out, stats)
    };

    if !stats.unknown.is_empty() {
        return markdown_image_error("Unknown images in markdown file", stats.unknown, input_file);
    }

    // The markdown file must reference all images in the group.
    let images_missing = stats.missing(&images);
    if !images_missing.is_empty() {
        return markdown_image_error(
            "Images present on disk but missing from the markdown file",
            images_missing,
            input_file,
        );
    }
    Ok(Rendered {
        html,
        images_seen: reorder_images(images, &stats.seen),
    })
}

// Reorder the given list of images to match their appearance in the markdown file.
fn reorder_images(images: Vec<ImageData>, images_seen: &[String]) -> Vec<ImageData> {
    // Map image names to their index.
    // If an image appears multiple times, this takes the last index.
    let sort_keys = HashMap::<String, usize>::from_iter(
        images_seen
            .iter()
            .enumerate()
            .map(|(i, img)| (img.clone(), i)),
    );
    let mut images = images.into_iter().collect::<Vec<ImageData>>();
    images.sort_by_key(|img| sort_keys[&img.name]);
    images
}

// Maps custom Markdown image tags to HTML snippets to include the image.
fn map_image_event<'a, 'e>(
    item: &Event<'e>,
    images: &'a [ImageData],
    stats: &'a mut ImageStatistics,
) -> Event<'e> {
    let text = match item {
        Event::Text(text) => text,
        _ => return item.clone(),
    };

    const IMAGE_TAG_PREFIX: &str = "!image ";
    let image_name = match text.strip_prefix(IMAGE_TAG_PREFIX) {
        Some(name) => name,
        None => return item.clone(),
    };
    let maybe_image = images.iter().find(|img| img.name == image_name);
    match maybe_image {
        None => {
            stats.unknown.push(image_name.to_owned());
            item.clone()
        }
        Some(img) => {
            stats.seen.push(image_name.to_owned());
            Event::Html(image_markdown_snippet(img).into())
        }
    }
}

fn image_markdown_snippet(img: &ImageData) -> String {
    format!(
        r#"<div class="card shadow-sm mb-3" id="{anchor}"><a href="{file_name}"><img class="card-img-top" src="../{thumbnail}"></a></div>"#,
        anchor = img.anchor,
        file_name = img.file_name,
        thumbnail = img.thumbnail,
    )
}
