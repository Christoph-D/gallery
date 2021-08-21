mod html;
mod images;

use crate::gallery::Gallery;

use anyhow::{anyhow, Context, Result};
use handlebars::Handlebars;
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
    handlebars
        .register_template_string("overview", include_str!("../templates/overview.handlebars"))?;
    handlebars.register_template_string(
        "image_group",
        include_str!("../templates/image_group.handlebars"),
    )?;

    html::render_overview_html(gallery, config, &handlebars)?.write(config)?;
    for i in &gallery.image_groups {
        html::render_image_group_html(&i, config, &handlebars)?
            .map_or(Ok(()), |f| f.write(config))?;
        images::render_images(&i, config)?.write(config)?;
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
