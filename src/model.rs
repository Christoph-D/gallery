//! Data structures to represent image galleries throughput the program.
use crate::error::PathErrorContext;

use anyhow::Result;
use std::fmt;
use std::path::{Path, PathBuf};
use time::Date;

/// An input image.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Image {
    /// The user-visible name of the image.
    pub name: String,
    /// The full path to the source image.
    pub path: PathBuf,
    /// The file name of the source image.
    pub file_name: PathBuf,
}

/// A list of input images.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImageGroup {
    /// The path to the image group directory relative to the base directory.
    pub path: PathBuf,
    /// The user-visible title of the image group.
    pub title: String,
    /// The date of the image group.
    pub date: Date,
    /// The contained images.
    /// Sorted alphabetically.
    pub images: Vec<Image>,
    /// An optional markdown file to explain the image group.
    /// Not yet fully implemented.
    pub markdown_file: Option<PathBuf>,
}

/// A gallery of images.
#[derive(Debug)]
pub struct Gallery {
    /// The list of image groups in the gallery.
    /// Sorted by date (most recent first).
    pub image_groups: Vec<ImageGroup>,
}

/// Different thumbnail types for different use cases.
///
/// The overview page uses small thumbnails, the image group pages use large thumbnails.
pub enum ThumbnailType {
    Small,
    Large,
}

impl Image {
    pub fn new(file_name: PathBuf, path: PathBuf) -> Result<Image> {
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
}

impl ImageGroup {
    /// The URL to this image group, relative to the base directory.
    /// The return value is guaranteed to consist only of ASCII characters.
    pub fn url(&self) -> Result<PathBuf> {
        let mut p = to_web_path(&self.path)?;
        p.set_extension("html");
        Ok(PathBuf::from("html").join(p))
    }
    /// The URL to an image in this image group, relative to the base directory.
    /// The return value is guaranteed to consist only of ASCII characters.
    pub fn image_url(&self, img: &Image) -> Result<PathBuf> {
        Ok(PathBuf::from("img").join(self.image_filename(img)?))
    }
    /// The URL to an image in this image group, relative to the base directory.
    /// The return value is guaranteed to consist only of ASCII characters.
    pub fn thumbnail_url(&self, img: &Image, thumbnail_type: &ThumbnailType) -> Result<PathBuf> {
        let mut suffix = self.image_filename(img)?;
        // Always use webp for thumbnails to get a reasonable quality.
        suffix.set_extension("webp");
        let size = match thumbnail_type {
            ThumbnailType::Small => "small",
            ThumbnailType::Large => "large",
        };
        Ok(PathBuf::from("thumbnails").join(size).join(&suffix))
    }
    /// The web-safe filename of an image in this image group.
    /// The return value is guaranteed to consist only of ASCII characters.
    pub fn image_filename(&self, img: &Image) -> Result<PathBuf> {
        to_web_path(&self.path.join(&img.file_name))
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

/// Converts a path into something suitable for a URL. The resulting path consists of a single component.
fn to_web_path(path: &Path) -> Result<PathBuf> {
    let p = path
        .to_str()
        .path_context("Failed to convert path to UTF-8", path)?;
    // Keep the file extension intact if one is present.
    let Some((path, ext)) = p.rsplit_once('.') else {
        return Ok(PathBuf::from(slug::slugify(p)));
    };
    Ok(PathBuf::from(slug::slugify(path) + "." + ext))
}

#[cfg(test)]
mod tests {
    use super::{Image, ImageGroup, ThumbnailType, to_web_path};
    use std::path::{Path, PathBuf};
    use time::{Date, Month};

    // Tests for to_web_path.

    #[test]
    fn to_web_path_empty_is_empty() {
        assert_eq!(to_web_path(Path::new("")).unwrap(), PathBuf::from(""));
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
        assert_eq!(
            to_web_path(Path::new("2021-12-01 Fuji, Japan/Summit.webp")).unwrap(),
            PathBuf::from("2021-12-01-fuji-japan-summit.webp")
        );
    }

    // Tests for thumbnails.

    fn new_image_group(group_path: &str, image_path: &str) -> ImageGroup {
        let image_path = PathBuf::from(image_path);
        ImageGroup {
            path: PathBuf::from(group_path),
            title: "ignored".to_owned(),
            date: Date::from_calendar_date(2021, Month::January, 01).unwrap(),
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
            group.thumbnail_url(&image, &ThumbnailType::Small).unwrap(),
            PathBuf::from("thumbnails/small/2021-01-01-some-group-some-file.webp")
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
            group.thumbnail_url(&image, &ThumbnailType::Small).unwrap(),
            // The thumbnail should be webp even for jpeg source files.
            PathBuf::from("thumbnails/small/2021-01-01-some-group-some-file.webp")
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
            group.thumbnail_url(&image, &ThumbnailType::Large).unwrap(),
            PathBuf::from("thumbnails/large/2021-01-01-some-group-some-file.webp")
        );
    }
}
