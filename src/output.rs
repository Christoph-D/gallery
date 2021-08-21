use crate::gallery::{Gallery, Image, ImageGroup};

use anyhow::{anyhow, Context, Result};
use handlebars::Handlebars;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

pub enum RunMode {
    Normal,
    DryRun,
}

pub struct Config {
    pub output_path: PathBuf,
    pub run_mode: RunMode,
    pub page_title: String,
    pub page_footer: Option<String>,
}

pub fn write_files(gallery: &Gallery, config: &Config) -> Result<()> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars.register_template_string("overview", include_str!("../templates/overview.hb"))?;
    handlebars
        .register_template_string("image_group", include_str!("../templates/image_group.hb"))?;

    render_overview_html(gallery, config, &handlebars)?.write(config)?;
    for i in &gallery.image_groups {
        render_image_group_html(&i, config, &handlebars)?.map_or(Ok(()), |f| f.write(config))?;
        images::write_images(&i, config)?;
    }
    write_static(config)
}

fn write_static(config: &Config) -> Result<()> {
    let css_path = config.output_path.join("css").join("bootstrap.min.css");
    let custom_css_path = config.output_path.join("css").join("style.css");
    let js_path = config
        .output_path
        .join("js")
        .join("bootstrap.bundle.min.js");
    for (path, content) in [
        (&css_path, include_str!("../templates/bootstrap.min.css")),
        (&custom_css_path, include_str!("../templates/style.css")),
        (
            &js_path,
            include_str!("../templates/bootstrap.bundle.min.js"),
        ),
    ] {
        match config.run_mode {
            RunMode::Normal => {
                create_parent_directories(path)?;
                fs::write(path, content).with_context(|| {
                    format!("Failed to write asset: \"{}\"", css_path.to_string_lossy())
                })?;
            }
            RunMode::DryRun => {
                println!("Static: \"{}\"", path.to_string_lossy());
            }
        }
    }
    Ok(())
}

fn create_parent_directories(path: &Path) -> Result<()> {
    let dir = path.parent().ok_or_else(|| {
        anyhow!(
            "Could not determine parent directory of \"{}\"",
            path.to_string_lossy()
        )
    })?;
    fs::create_dir_all(dir)
        .with_context(|| format!("Failed to create directory \"{}\"", dir.to_string_lossy()))
}

fn to_web_path(path: &Path) -> Result<PathBuf> {
    let p = path.to_str().ok_or_else(|| {
        anyhow!(
            "Failed to convert path to UTF-8: \"{}\"",
            path.to_string_lossy()
        )
    })?;
    // Keep the file extension intact if one is present.
    match p.rsplit_once('.') {
        Some((path, ext)) => Ok(PathBuf::from(slug::slugify(path) + "." + ext)),
        None => Ok(PathBuf::from(slug::slugify(p))),
    }
}

struct HTMLFile {
    content: String,
    output_path: PathBuf,
}

impl HTMLFile {
    fn write(&self, config: &Config) -> Result<()> {
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

fn render_overview_html(
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

fn render_image_group_html(
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

mod images {
    use crate::gallery::{Image, ImageGroup};

    use anyhow::{anyhow, Context, Result};
    use std::path::PathBuf;
    use std::{fs, process};

    use super::{create_parent_directories, to_web_path, Config, RunMode};

    pub enum ThumbnailType {
        Small,
        Large,
    }

    pub fn write_images(image_group: &ImageGroup, config: &Config) -> Result<()> {
        use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
        image_group
            .images
            .par_iter()
            .map(|img| {
                render_image(&img, image_group, config).map(|rendered| rendered.write(config))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(())
    }

    fn none_if_exists(path: PathBuf) -> Option<PathBuf> {
        if path.exists() {
            None
        } else {
            Some(path)
        }
    }

    pub fn relative_output_path(group: &ImageGroup, image: &Image) -> Result<PathBuf> {
        Ok(to_web_path(&group.path)?.join(to_web_path(&image.file_name)?))
    }

    pub fn output_path(
        group: &ImageGroup,
        image: &Image,
        config: &Config,
    ) -> Result<Option<PathBuf>> {
        Ok(none_if_exists(
            config.output_path.join(relative_output_path(group, image)?),
        ))
    }

    pub fn relative_thumbnail_path(
        group: &ImageGroup,
        image: &Image,
        thumbnail_type: &ThumbnailType,
    ) -> Result<PathBuf> {
        let suffix = to_web_path(&group.path)?.join(to_web_path(&image.file_name)?);
        let size = match thumbnail_type {
            ThumbnailType::Small => "small",
            ThumbnailType::Large => "large",
        };
        Ok(PathBuf::from("thumbnails").join(size).join(&suffix))
    }

    pub fn thumbnail_path(
        group: &ImageGroup,
        image: &Image,
        config: &Config,
        thumbnail_type: &ThumbnailType,
    ) -> Result<PathBuf> {
        Ok(config
            .output_path
            .join(relative_thumbnail_path(group, image, thumbnail_type)?))
    }

    struct ImageFile {
        source_path: PathBuf,
        output_path: Option<PathBuf>,
        thumbnail_path_small: Option<PathBuf>,
        thumbnail_path_large: Option<PathBuf>,
    }

    fn check_thumbnail_path(
        group: &ImageGroup,
        image: &Image,
        config: &Config,
        thumbnail_type: &ThumbnailType,
    ) -> Result<Option<PathBuf>> {
        // No need to create a large thumbnail if the group doesn't have its own page.
        match thumbnail_type {
            ThumbnailType::Large if group.markdown_file.is_none() => return Ok(None),
            _default => (),
        };
        Ok(none_if_exists(thumbnail_path(
            group,
            image,
            config,
            thumbnail_type,
        )?))
    }

    fn render_image(image: &Image, group: &ImageGroup, config: &Config) -> Result<ImageFile> {
        Ok({
            ImageFile {
                source_path: image.path.clone(),
                output_path: output_path(group, image, config)?,
                thumbnail_path_small: check_thumbnail_path(
                    group,
                    image,
                    config,
                    &ThumbnailType::Small,
                )?,
                thumbnail_path_large: check_thumbnail_path(
                    group,
                    image,
                    config,
                    &ThumbnailType::Large,
                )?,
            }
        })
    }

    impl ImageFile {
        fn write_thumbnail(
            &self,
            thumbnail_path: &Option<PathBuf>,
            dimensions: &str,
            crop: &str,
        ) -> Result<()> {
            if thumbnail_path.is_none() {
                return Ok(());
            }
            let thumbnail_path = thumbnail_path.as_ref().unwrap();
            super::create_parent_directories(thumbnail_path)?;
            let result = process::Command::new("convert")
                .arg(&self.source_path)
                .args(&[
                    "-resize", dimensions, "-gravity", "center", "-crop", crop, "+repage",
                    "-quality", "80",
                ])
                .arg(thumbnail_path)
                .output()
                .with_context(|| {
                    format!(
                        "Failed to run imagemagick 'convert': \"{}\"",
                        self.source_path.to_string_lossy()
                    )
                })?;
            if !result.status.success() {
                return Err(anyhow!(
                    "Failed to create thumbnail: \"{}\"",
                    self.source_path.to_string_lossy()
                ));
            }
            Ok(())
        }

        fn write_thumbnails(&self, config: &Config) -> Result<()> {
            match &config.run_mode {
                RunMode::Normal => {
                    self.write_thumbnail(&self.thumbnail_path_small, "400x", "400x267+0+0")?;
                    self.write_thumbnail(&self.thumbnail_path_large, "2000x", "2000x1335+0+0")
                }
                RunMode::DryRun => Ok(()), // Thumbnails are silent in dry-run mode.
            }
        }

        fn write_image(&self, config: &Config) -> Result<()> {
            let output_path = match &self.output_path {
                Some(p) => p,
                None => return Ok(()),
            };
            match &config.run_mode {
                RunMode::Normal => {
                    create_parent_directories(output_path)?;
                    fs::copy(&self.source_path, output_path).with_context(|| {
                        format!(
                            "Failed to copy image \"{}\" -> \"{}\"",
                            self.source_path.to_string_lossy(),
                            output_path.to_string_lossy()
                        )
                    })?;
                }
                RunMode::DryRun => {
                    println!("Image: \"{}\"", output_path.to_string_lossy());
                }
            }
            Ok(())
        }

        fn write(&self, config: &Config) -> Result<()> {
            self.write_image(config)?;
            self.write_thumbnails(config)
        }
    }
}
