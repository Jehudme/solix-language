use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn find_libraylib(cache_directory: &Path) -> Option<PathBuf> {
    let candidate_paths = [
        cache_directory.join("_deps").join("raylib-build").join("raylib").join("libraylib.a"),
        cache_directory.join("raylib").join("libraylib.a"),
        cache_directory.join("libraylib.a"),
        cache_directory.join("host").join("raylib").join("libraylib.a"),
        cache_directory.join("host").join("_deps").join("raylib-build").join("raylib").join("libraylib.a"),
    ];

    for candidate in &candidate_paths {
        if candidate.exists() {
            return Some(candidate.clone());
        }
    }
    None
}

fn main() {
    let manifest_directory = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let project_root = manifest_directory
        .parent()
        .expect("natives directory")
        .parent()
        .expect("raylib root directory")
        .to_path_buf();

    let host_cache_directory = project_root
        .join("build")
        .join(".cache")
        .join("cmake")
        .join("host");

    let target_cache_directory = if let Ok(target_triple) = env::var("TARGET") {
        project_root
            .join("build")
            .join(".cache")
            .join("cmake")
            .join(target_triple)
    } else {
        host_cache_directory.clone()
    };

    let library_file_option = if let Ok(custom_lib_directory) = env::var("RAYLIB_LIB_DIR") {
        find_libraylib(Path::new(&custom_lib_directory))
    } else {
        find_libraylib(&target_cache_directory).or_else(|| find_libraylib(&host_cache_directory))
    };

    let final_library_file = match library_file_option {
        Some(found_library_path) => found_library_path,
        None => {
            let configure_status = Command::new("cmake")
                .arg("-B")
                .arg(&host_cache_directory)
                .arg("-S")
                .arg(&manifest_directory)
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
                .arg(&host_cache_directory)
                .arg("--target")
                .arg("raylib")
                .status()
                .expect("Failed to execute cmake build");

            if !build_status.success() {
                panic!("cmake build failed with status: {:?}", build_status);
            }

            find_libraylib(&host_cache_directory).expect("libraylib.a must exist after cmake build")
        }
    };

    let search_directory = final_library_file.parent().expect("library directory");

    println!("cargo:rustc-link-search=native={}", search_directory.display());
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
