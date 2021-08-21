use crate::gallery::{Gallery, Image, ImageGroup};

use anyhow::{anyhow, Context, Error, Result};
use chrono::naive::NaiveDate;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fmt, fs};

pub fn gallery_from_dir(path: &Path) -> Result<Gallery> {
    let mut image_groups = Vec::<ImageGroup>::new();
    for d in read_dir(path)?.iter().filter(|d| d.is_dir) {
        let contents = read_dir(&d.path)?;
        if let Some(group) = ImageGroup::from_entries(&d.file_name, &contents)? {
            image_groups.push(group);
        }
    }
    image_groups.sort_by(|lhs, rhs| rhs.date.cmp(&lhs.date));
    Ok(Gallery { image_groups })
}

impl Image {
    fn from(d: &DirEntry) -> Result<Image> {
        Ok(Image {
            name: String::from(
                d.file_name
                    .file_stem()
                    .ok_or_else(|| {
                        anyhow!(
                            "Could not determine file stem: {}",
                            d.file_name.to_string_lossy()
                        )
                    })?
                    .to_str()
                    .ok_or_else(|| {
                        anyhow!(
                            "Could not convert file name to UTF-8: {}",
                            d.file_name.to_string_lossy()
                        )
                    })?,
            ),
            path: d.path.clone(),
            file_name: d.file_name.clone(),
        })
    }
}

impl ImageGroup {
    fn from_entries(path: &Path, v: &[DirEntry]) -> Result<Option<ImageGroup>> {
        let id = String::from(path.to_string_lossy());
        let (title, date) = {
            let re = Regex::new(r"^(\d{4})-(\d{2})-(\d{2}).").unwrap();
            let c = {
                match re.captures(&id) {
                    Some(c) => c,
                    None => return Ok(None),
                }
            };
            (
                String::from(re.replace(&id, "")),
                NaiveDate::from_ymd(
                    FromStr::from_str(c.get(1).unwrap().as_str())?,
                    FromStr::from_str(c.get(2).unwrap().as_str())?,
                    FromStr::from_str(c.get(3).unwrap().as_str())?,
                ),
            )
        };
        let mut images = Vec::new();
        let mut markdown_file = None;
        for d in v {
            if d.is_image() {
                images.push(Image::from(d)?);
            }
            if d.is_index() {
                markdown_file = Some(d.path.clone());
            }
        }
        images.sort();
        Ok(Some(ImageGroup {
            id,
            path: path.to_owned(),
            title,
            date,
            images,
            markdown_file,
        }))
    }
}

#[derive(Debug)]
struct DirEntry {
    path: PathBuf,
    file_name: PathBuf, // relative to the base dir
    is_dir: bool,
}

impl DirEntry {
    fn is_image(&self) -> bool {
        self.path
            .extension()
            .map_or(false, |e| e == "webp" || e == "jpeg")
    }
    fn is_index(&self) -> bool {
        self.path.file_name().map_or(false, |f| f == "index.md")
    }
}

impl fmt::Display for DirEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path.to_string_lossy())
    }
}

// Reads a directory non-recursively.
fn read_dir(base_dir: &Path) -> Result<Vec<DirEntry>> {
    let mut res = Vec::new();
    let p = base_dir.to_string_lossy();
    for path in
        fs::read_dir(base_dir).with_context(|| format!("Failed to open directory: {}", p))?
    {
        let d = path.with_context(|| format!("Failed to read the contents of directory: {}", p))?;
        let path = d.path();
        res.push(DirEntry {
            file_name: PathBuf::from(path.strip_prefix(base_dir).map_err(Error::msg)?),
            is_dir: d
                .metadata()
                .with_context(|| format!("Could not read metadata: {}", path.to_string_lossy()))?
                .is_dir(),
            path,
        })
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::{DirEntry, Image, ImageGroup};
    use chrono::naive::NaiveDate;
    use std::path::{Path, PathBuf};

    fn dir(dirname: &str, file_names: &[(&str, bool)]) -> Vec<DirEntry> {
        file_names
            .iter()
            .map(|(p, is_dir)| DirEntry {
                path: [dirname, p].iter().collect(),
                file_name: PathBuf::from(p),
                is_dir: *is_dir,
            })
            .collect()
    }
    struct SimpleImageGroup<'a> {
        name: &'a str,
        title: &'a str,
        date: NaiveDate,
        // (name, path)
        images: &'a [(&'a str, &'a str)],
        markdown_file: Option<&'a str>,
    }
    impl<'a> From<SimpleImageGroup<'a>> for ImageGroup {
        fn from(s: SimpleImageGroup) -> ImageGroup {
            ImageGroup {
                id: String::from(s.name),
                path: PathBuf::from(s.name),
                title: String::from(s.title),
                date: s.date,
                images: s
                    .images
                    .iter()
                    .map(|(n, p)| Image {
                        name: String::from(*n),
                        path: PathBuf::from(p),
                        file_name: PathBuf::from(p).file_name().unwrap().into(),
                    })
                    .collect(),
                markdown_file: s.markdown_file.map(PathBuf::from),
            }
        }
    }

    #[test]
    fn test_empty_dir() {
        assert_eq!(
            ImageGroup::from_entries(Path::new("2021-01-01 Fuji, Japan"), &[]).unwrap(),
            Some(ImageGroup::from(SimpleImageGroup {
                title: "Fuji, Japan",
                name: "2021-01-01 Fuji, Japan",
                date: NaiveDate::from_ymd(2021, 01, 01),
                images: &[],
                markdown_file: None,
            }))
        );
    }
    #[test]
    fn test_simple_dir() {
        assert_eq!(
            ImageGroup::from_entries(
                Path::new("2021-01-01 Fuji, Japan"),
                &dir(
                    "2021-01-01 Fuji, Japan",
                    &[("Valley.webp", false), ("Summit.webp", false),]
                )
            )
            .unwrap(),
            Some(ImageGroup::from(SimpleImageGroup {
                name: "2021-01-01 Fuji, Japan",
                title: "Fuji, Japan",
                date: NaiveDate::from_ymd(2021, 01, 01),
                images: &[
                    ("Summit", "2021-01-01 Fuji, Japan/Summit.webp"),
                    ("Valley", "2021-01-01 Fuji, Japan/Valley.webp"),
                ],
                markdown_file: None,
            }))
        );
    }
    #[test]
    fn test_index() {
        assert_eq!(
            ImageGroup::from_entries(
                Path::new("2021-01-01 Fuji, Japan"),
                &dir("some/path/2021-01-01 Fuji, Japan", &[("index.md", false)])
            )
            .unwrap(),
            Some(ImageGroup::from(SimpleImageGroup {
                name: "2021-01-01 Fuji, Japan",
                title: "Fuji, Japan",
                date: NaiveDate::from_ymd(2021, 01, 01),
                images: &[],
                markdown_file: Some("some/path/2021-01-01 Fuji, Japan/index.md")
            }))
        );
    }
    #[test]
    fn test_ignored_entries() {
        assert_eq!(
            ImageGroup::from_entries(
                Path::new("2021-12-01 Fuji, Japan"),
                &dir(
                    "some/path/2021-12-01 Fuji, Japan",
                    &[
                        ("Valley", true), // directory
                        ("Summit.webp", false),
                        ("something.unknown", false),
                    ]
                )
            )
            .unwrap(),
            Some(ImageGroup::from(SimpleImageGroup {
                name: "2021-12-01 Fuji, Japan",
                title: "Fuji, Japan",
                date: NaiveDate::from_ymd(2021, 12, 01),
                images: &[("Summit", "some/path/2021-12-01 Fuji, Japan/Summit.webp")],
                markdown_file: None,
            }))
        );
    }
    #[test]
    fn test_missing_date_in_dirname() {
        assert_eq!(
            ImageGroup::from_entries(
                Path::new("2021-01 Fuji, Japan"),
                &dir("some/path/2021-01 Fuji, Japan", &[("Summit.webp", false)])
            )
            .unwrap(),
            None
        );
    }
}
