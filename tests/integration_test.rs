//! Integration tests.
//!
//! The tests here require imagemagick (the `convert` binary) to be in the `PATH`
//! or the tests will fail when trying to generate thumbnails.
use std::{fs, path::Path, process::Command};

/// A valid 1-pixel sized webp image.
/// The image needs to be valid to test the generation of thumbnails.
const DUMMY_WEBP: &[u8] = include_bytes!("dummy.webp");

fn run_main(
    inputdir: &Path,
    outputdir: &Path,
    page_title: &str,
    footer: &str,
    extra_args: &[&str],
) {
    let output = Command::new(env!("CARGO_BIN_EXE_gallery"))
        .args([
            &("--page_title=".to_owned() + page_title),
            &("--footer=".to_owned() + footer),
            &("--input=".to_owned() + inputdir.to_str().unwrap()),
            &("--output=".to_owned() + outputdir.to_str().unwrap()),
        ])
        .args(extra_args)
        .output()
        .expect("Failed to run main");
    if !output.status.success() {
        panic!(
            "Error:\nstderr:\n{}\n\nstdout:\n{}\n",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn test_empty_input() {
    let tempdir_raw = tempfile::tempdir().unwrap();
    let tempdir = tempdir_raw.path();
    let inputdir = &tempdir.join("input");
    let outputdir = &tempdir.join("output");

    fs::create_dir(tempdir.join("input")).unwrap();

    run_main(
        inputdir,
        outputdir,
        "Some title",
        "Some footer <a>link</a>",
        &[],
    );

    let index = fs::read_to_string(outputdir.join("index.html")).unwrap();
    assert!(index.contains("Some title"));
    assert!(index.contains("Some footer <a>link</a>"));
}

#[test]
fn test_simple_input() {
    let tempdir_raw = tempfile::tempdir().unwrap();
    let tempdir = tempdir_raw.path();
    let inputdir = &tempdir.join("input");
    let outputdir = &tempdir.join("output");

    fs::create_dir(inputdir).unwrap();
    fs::create_dir(inputdir.join("2021-01-01 Fuji, Japan")).unwrap();
    fs::write(
        inputdir.join("2021-01-01 Fuji, Japan/Summit.webp"),
        DUMMY_WEBP,
    )
    .unwrap();

    run_main(inputdir, outputdir, "Title", "Footer", &[]);

    // The overview page should reference the image.
    let index = fs::read_to_string(outputdir.join("index.html")).unwrap();
    assert!(index.contains("Fuji, Japan"));
    assert!(index.contains("Summit"));
    assert!(index.contains("href=\"2021-01-01-fuji-japan/summit.webp\""));

    assert!(outputdir
        .join("2021-01-01-fuji-japan/summit.webp")
        .is_file());
    assert!(outputdir
        .join("thumbnails/small/2021-01-01-fuji-japan/summit.webp")
        .is_file());
}

#[test]
fn test_dry_run_mode() {
    let tempdir_raw = tempfile::tempdir().unwrap();
    let tempdir = tempdir_raw.path();
    let inputdir = &tempdir.join("input");
    let outputdir = &tempdir.join("output");

    fs::create_dir(inputdir).unwrap();
    fs::create_dir(inputdir.join("2021-01-01 Fuji, Japan")).unwrap();
    fs::write(
        inputdir.join("2021-01-01 Fuji, Japan/Summit.webp"),
        DUMMY_WEBP,
    )
    .unwrap();

    run_main(inputdir, outputdir, "Title", "Footer", &["--dry_run"]);

    assert!(
        fs::read_dir(outputdir).is_err(),
        "Created output directory in dry-run mode."
    );
}
