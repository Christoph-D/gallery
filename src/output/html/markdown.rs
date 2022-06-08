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

pub(super) fn to_html(input_file: &Path, images: Vec<ImageData>) -> Result<Rendered> {
    let input = fs::read_to_string(input_file)
        .path_context("Failed to open image group markdown file", input_file)?;

    let (html, images_seen, images_unknown) = {
        let mut images_seen = Vec::new();
        let mut images_unknown = Vec::new();
        let iter = ImageGroupMarkdownIterator {
            iter: Parser::new(&input),
            images: &images,
            images_seen: &mut images_seen,
            images_unknown: &mut images_unknown,
        };
        let mut out = String::new();
        html::push_html(&mut out, iter);
        (out, images_seen, images_unknown)
    };

    if !images_unknown.is_empty() {
        return markdown_image_error(
            "Unknown images in markdown file",
            images_unknown,
            input_file,
        );
    }

    // The markdown file must reference all images in the group.
    let images_missing = {
        let images_seen_set = HashSet::<String>::from_iter(images_seen.iter().cloned());
        let mut missing = Vec::new();
        for image in &images {
            if !images_seen_set.contains(&image.name) {
                missing.push(image.name.to_owned());
            }
        }
        missing
    };
    if !images_missing.is_empty() {
        return markdown_image_error(
            "Images present on disk but missing from the markdown file",
            images_missing,
            input_file,
        );
    }
    Ok(Rendered {
        html,
        images_seen: reorder_images(images, &images_seen)?,
    })
}

// Reorder the given list of images to match their appearance in the markdown file.
fn reorder_images(images: Vec<ImageData>, images_seen: &[String]) -> Result<Vec<ImageData>> {
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
    Ok(images)
}

struct ImageGroupMarkdownIterator<'a, I> {
    iter: I,
    // Images from files on disk.
    images: &'a [ImageData],
    // Images that are referenced in the markdown file in their order of appearance.
    // May contain duplicates.
    images_seen: &'a mut Vec<String>,
    // Images that are in images but missing in the markdown file.
    images_unknown: &'a mut Vec<String>,
}

impl<'a, 'e, I: Iterator<Item = Event<'e>>> Iterator for ImageGroupMarkdownIterator<'a, I> {
    type Item = Event<'e>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(event) => self.map_image_event(&event).or(Some(event)),
            None => None,
        }
    }
}

impl<'a, I> ImageGroupMarkdownIterator<'a, I> {
    fn map_image_event<'e>(&mut self, item: &Event<'e>) -> Option<Event<'e>> {
        let text = match item {
            Event::Text(text) => text,
            _ => return None,
        };

        const IMAGE_TAG_PREFIX: &str = "!image ";
        if !text.starts_with(IMAGE_TAG_PREFIX) {
            return None;
        }
        let image_name = text.strip_prefix(IMAGE_TAG_PREFIX).unwrap();
        let maybe_image = self.images.iter().find(|img| img.name == image_name);
        match maybe_image {
            None => {
                self.images_unknown.push(image_name.to_owned());
                None
            }
            Some(img) => {
                self.images_seen.push(image_name.to_owned());
                Some(Event::Html(image_markdown_snippet(img).into()))
            }
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
