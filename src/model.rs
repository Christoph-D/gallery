//! Data structures to represent image galleries throughput the program.
use crate::error::{path_error, PathErrorContext};

use anyhow::Result;
use chrono::naive::NaiveDate;
use std::fmt;
use std::path::{Path, PathBuf};

/// An input image.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Image {
    /// The user-visible name of the image.
    pub name: String,
    /// The full path to the source image.
    pub path: PathBuf,
    /// The file name of the source image.
    pub file_name: PathBuf,
}

/// A list of input images.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ImageGroup {
    /// The path to the image group directory relative to the base directory.
    pub path: PathBuf,
    /// The user-visible title of the image group.
    pub title: String,
    /// The date of the image group.
    pub date: NaiveDate,
    /// The contained images.
    /// Sorted alphabetically.
    pub images: Vec<Image>,
    /// An optional markdown file to explain the image group.
    /// Not yet fully implemented.
    pub markdown_file: Option<PathBuf>,
}

/// A gallery of images.
#[derive(Debug)]
pub(crate) struct Gallery {
    /// The list of image groups in the gallery.
    /// Sorted by date (most recent first).
    pub image_groups: Vec<ImageGroup>,
}

/// Different thumbnail types for different use cases.
///
/// The overview page uses small thumbnails, the image group pages use large thumbnails.
pub(crate) enum ThumbnailType {
    Small,
    Large,
}

impl Image {
    pub(crate) fn new(file_name: PathBuf, path: PathBuf) -> Result<Image> {
        Ok(Image {
            name: file_name
                .file_stem()
                .path_context("Failed to determine file stem", &file_name)?
                .to_str()
                .path_context("Failed to decode file name as UTF-8", &file_name)?
                .to_owned(),
            path,
            file_name,
        })
    }

    /// The URL to this image, relative to the base directory.
    /// The return value is guaranteed to consist only of ASCII characters.
    pub(crate) fn url(&self, image_group: &ImageGroup) -> Result<PathBuf> {
        Ok(image_group.url()?.join(self.url_file_name()?))
    }

    /// The URL to this image, relative to the location of the image.
    /// That is, the returned URL contains no slashes.
    /// The return value is guaranteed to consist only of ASCII characters.
    pub(crate) fn url_file_name(&self) -> Result<PathBuf> {
        to_web_path(&self.file_name)
    }

    /// The URL to the thumbnail image relative to the output base directory.
    /// The return value is guaranteed to consist only of ASCII characters.
    pub(crate) fn thumbnail_url(
        &self,
        group: &ImageGroup,
        thumbnail_type: &ThumbnailType,
    ) -> Result<PathBuf> {
        let mut suffix = to_web_path(&group.path)?.join(to_web_path(&self.file_name)?);
        // Always use webp for thumbnails to get a reasonable quality.
        suffix.set_extension("webp");
        let size = match thumbnail_type {
            ThumbnailType::Small => "small",
            ThumbnailType::Large => "large",
        };
        Ok(PathBuf::from("thumbnails").join(size).join(&suffix))
    }
}

impl ImageGroup {
    /// The URL to this image group, relative to the base directory.
    /// The return value is guaranteed to consist only of ASCII characters.
    pub(crate) fn url(&self) -> Result<PathBuf> {
        to_web_path(&self.path)
    }
}

impl fmt::Display for ImageGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\"{} ({})\" -> [{}] [{:?}]",
            self.title,
            self.date,
            self.images
                .iter()
                .map(|i| i.name.as_ref())
                .collect::<Vec<_>>()
                .join(", "),
            self.markdown_file
                .as_ref()
                .map_or(String::new(), |p| p.to_string_lossy().to_string()),
        )
    }
}

impl fmt::Display for Gallery {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for g in &self.image_groups {
            writeln!(f, "{}", g)?
        }
        Ok(())
    }
}

/// Converts a single-element path into something suitable for a URL.
fn to_web_path(path: &Path) -> Result<PathBuf> {
    if path.components().count() != 1 {
        return Err(path_error(
            "Cannot convert multi-component paths into URLs",
            &path,
        ));
    }
    let p = path
        .to_str()
        .path_context("Failed to convert path to UTF-8", &path)?;
    // Keep the file extension intact if one is present.
    match p.rsplit_once('.') {
        Some((path, ext)) => Ok(PathBuf::from(slug::slugify(path) + "." + ext)),
        None => Ok(PathBuf::from(slug::slugify(p))),
    }
}

#[cfg(test)]
mod tests {
    use super::{to_web_path, Image, ImageGroup, ThumbnailType};
    use chrono::naive::NaiveDate;
    use std::path::{Path, PathBuf};

    // Tests for to_web_path.

    #[test]
    fn to_web_path_empty_is_error() {
        assert!(to_web_path(Path::new("")).is_err());
    }

    #[test]
    fn to_web_path_simple() {
        assert_eq!(
            to_web_path(Path::new("2021-12-01 Fuji, Japan")).unwrap(),
            PathBuf::from("2021-12-01-fuji-japan")
        );
    }

    #[test]
    fn to_web_path_umlaut_is_removed() {
        assert_eq!(
            to_web_path(Path::new("2021-12-01 Zürich")).unwrap(),
            PathBuf::from("2021-12-01-zurich")
        );
    }

    #[test]
    fn to_web_path_file_extension_remains_intact() {
        assert_eq!(
            to_web_path(Path::new("Fuji, Japan.webp")).unwrap(),
            PathBuf::from("fuji-japan.webp")
        );
    }

    #[test]
    fn to_web_path_multi_component_is_error() {
        assert!(to_web_path(Path::new("2021-12-01 Fuji, Japan/Summit.webp")).is_err());
    }

    // Tests for thumbnails.

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
            image.thumbnail_url(&group, &ThumbnailType::Small).unwrap(),
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
            image.thumbnail_url(&group, &ThumbnailType::Small).unwrap(),
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
            image.thumbnail_url(&group, &ThumbnailType::Large).unwrap(),
            PathBuf::from("thumbnails/large/2021-01-01-some-group/some-file.webp")
        );
    }
}
