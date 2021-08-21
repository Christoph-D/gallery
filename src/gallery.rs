use chrono::naive::NaiveDate;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Image {
    pub name: String,
    // Full path to the source image.
    pub path: PathBuf,
    // The file name of the source image.
    pub file_name: PathBuf,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImageGroup {
    pub id: String,
    // Path relative to the base directory.
    pub path: PathBuf,
    pub title: String,
    pub date: NaiveDate,
    pub images: Vec<Image>,
    pub markdown_file: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Gallery {
    // Sorted by date (most recent first).
    pub image_groups: Vec<ImageGroup>,
}

impl fmt::Display for ImageGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<{}> \"{} ({})\" -> [{}] [{:?}]",
            self.id,
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
