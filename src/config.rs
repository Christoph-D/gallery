//! Configuration types

/// Normal or dryrun (read-only) mode.
pub enum RunMode {
    Normal,
    DryRun,
}

/// Image group order.
pub enum GalleryOrder {
    MostRecentFirst,
    OldestFirst,
}

/// Configuration options for the output module.
pub struct Config {
    /// The target directory where to write the gallery.
    pub output_path: std::path::PathBuf,
    /// Normal or dryrun (read-only) mode.
    pub run_mode: RunMode,
    /// Image group order.
    pub order: GalleryOrder,
    /// The top-level title of the generated gallery.
    pub page_title: String,
    /// An optional footer to show (for example) a copyright notice.
    pub page_footer: Option<String>,
}
