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
use std::collections::HashSet;
use std::{fs, path::Path};

pub(super) fn to_html(input_file: &Path, images: &[ImageData]) -> Result<String> {
    let input = fs::read_to_string(input_file)
        .path_context("Failed to open image group markdown file", input_file)?;

    let (html_output, images_seen, images_unknown) = {
        let mut images_seen = HashSet::new();
        let mut images_unknown = HashSet::new();
        let iter = ImageGroupMarkdownIterator {
            iter: Parser::new(&input),
            images,
            images_seen: &mut images_seen,
            images_unknown: &mut images_unknown,
        };
        let mut out = String::new();
        html::push_html(&mut out, iter);
        (out, images_seen, images_unknown)
    };

    if !images_unknown.is_empty() {
        return Err(anyhow!(
            "Unknown images: {}",
            images_unknown.into_iter().collect::<Vec<_>>().join(", ")
        ))
        .path_context("Error in markdown file", input_file);
    }

    // The markdown file must reference all images in the group.
    let images_missing = {
        let mut missing = Vec::new();
        for image in images {
            if !images_seen.contains(&image.name) {
                missing.push(image.name.to_owned());
            }
        }
        missing
    };
    if !images_missing.is_empty() {
        Err(anyhow!("Missing images: {}", images_missing.join(", ")))
            .path_context("Error in markdown file", input_file)
    } else {
        Ok(html_output)
    }
}

struct ImageGroupMarkdownIterator<'a, I> {
    iter: I,
    images: &'a [ImageData],
    images_seen: &'a mut HashSet<String>,
    images_unknown: &'a mut HashSet<String>,
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
                self.images_unknown.insert(image_name.to_owned());
                None
            }
            Some(img) => {
                self.images_seen.insert(image_name.to_owned());
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
