//! Build script for `solix-toolchain`.
//!
//! Stages `assets/` into the target output directory alongside the `solix` binary.

#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn copy_dir_all(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let target_path = destination.join(entry.file_name());
        if entry_type.is_dir() {
            copy_dir_all(&entry.path(), &target_path)?;
        } else {
            fs::copy(entry.path(), target_path)?;
        }
    }
    Ok(())
}

fn main() {
    println!("cargo:rerun-if-changed=assets");

    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => return,
    };

    // OUT_DIR is target/<profile>/build/solix-toolchain-<hash>/out
    // Navigating 3 ancestors up yields target/<profile>
    let target_dir = match out_dir.ancestors().nth(3) {
        Some(dir) => dir,
        None => return,
    };

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    let source_assets = manifest_dir.join("assets");
    if source_assets.exists() {
        let target_assets = target_dir.join("assets");
        let _ = copy_dir_all(&source_assets, &target_assets);
    }
}
