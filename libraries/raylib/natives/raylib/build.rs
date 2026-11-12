use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn find_libraylib(cache_directory: &Path) -> Option<PathBuf> {
    let candidate_paths = [
        cache_directory.join("_deps").join("raylib-build").join("raylib").join("libraylib.a"),
        cache_directory.join("raylib").join("libraylib.a"),
        cache_directory.join("libraylib.a"),
    ];

    for candidate in &candidate_paths {
        if candidate.exists() {
            return Some(candidate.clone());
        }
    }
    None
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir
        .parent()
        .expect("natives dir")
        .parent()
        .expect("raylib root dir")
        .to_path_buf();

    let cmake_cache_dir = project_root.join("build").join(".cache").join("cmake");

    if find_libraylib(&cmake_cache_dir).is_none() {
        let configure_status = Command::new("cmake")
            .arg("-B")
            .arg(&cmake_cache_dir)
            .arg("-S")
            .arg(&project_root)
            .arg("-DCMAKE_POSITION_INDEPENDENT_CODE=ON")
            .arg("-DBUILD_EXAMPLES=OFF")
            .arg("-DBUILD_SHARED_LIBS=OFF")
            .status()
            .expect("Failed to execute cmake configure");

        if !configure_status.success() {
            panic!("cmake configure failed with status: {:?}", configure_status);
        }

        let build_status = Command::new("cmake")
            .arg("--build")
            .arg(&cmake_cache_dir)
            .arg("--target")
            .arg("raylib")
            .status()
            .expect("Failed to execute cmake build");

        if !build_status.success() {
            panic!("cmake build failed with status: {:?}", build_status);
        }
    }

    let library_file = find_libraylib(&cmake_cache_dir).expect("libraylib.a must exist after cmake build");
    let search_dir = library_file.parent().expect("library directory");

    println!("cargo:rustc-link-search=native={}", search_dir.display());
    println!("cargo:rustc-link-lib=static=raylib");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    match target_os.as_str() {
        "linux" => {
            println!("cargo:rustc-link-lib=GL");
            println!("cargo:rustc-link-lib=m");
            println!("cargo:rustc-link-lib=pthread");
            println!("cargo:rustc-link-lib=dl");
            println!("cargo:rustc-link-lib=rt");
            println!("cargo:rustc-link-lib=X11");
        }
        "windows" => {
            println!("cargo:rustc-link-lib=opengl32");
            println!("cargo:rustc-link-lib=gdi32");
            println!("cargo:rustc-link-lib=winmm");
            println!("cargo:rustc-link-lib=shell32");
            println!("cargo:rustc-link-lib=user32");
        }
        "macos" => {
            println!("cargo:rustc-link-lib=framework=OpenGL");
            println!("cargo:rustc-link-lib=framework=Cocoa");
            println!("cargo:rustc-link-lib=framework=IOKit");
            println!("cargo:rustc-link-lib=framework=CoreVideo");
        }
        other_os => {
            println!("cargo:warning=Untested target OS for raylib: {}", other_os);
        }
    }
}
