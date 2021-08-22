//! Data structures to represent image galleries throughput the program.
use chrono::naive::NaiveDate;
use std::fmt;
use std::path::PathBuf;

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
pub struct Gallery {
    /// The list of image groups in the gallery.
    /// Sorted by date (most recent first).
    pub image_groups: Vec<ImageGroup>,
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
