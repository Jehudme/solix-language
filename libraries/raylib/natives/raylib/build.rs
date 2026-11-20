use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn strip_windows_verbatim_prefix(path: PathBuf) -> PathBuf {
    if cfg!(windows) {
        if let Some(raw_text) = path.to_str() {
            if let Some(unc_tail) = raw_text.strip_prefix(r"\\?\UNC\") {
                return PathBuf::from(format!(r"\\{unc_tail}"));
            }
            if let Some(drive_tail) = raw_text.strip_prefix(r"\\?\") {
                return PathBuf::from(drive_tail);
            }
        }
    }
    path
}

fn find_libraylib(cache_directory: &Path, prefer_msvc: bool) -> Option<PathBuf> {
    let msvc_candidates = [
        cache_directory
            .join("lib")
            .join("Release")
            .join("raylib.lib"),
        cache_directory.join("lib").join("raylib.lib"),
        cache_directory
            .join("raylib")
            .join("Release")
            .join("raylib.lib"),
        cache_directory.join("raylib").join("raylib.lib"),
        cache_directory
            .join("_deps")
            .join("raylib-build")
            .join("raylib")
            .join("Release")
            .join("raylib.lib"),
        cache_directory
            .join("_deps")
            .join("raylib-build")
            .join("raylib")
            .join("raylib.lib"),
        cache_directory.join("raylib.lib"),
    ];
    let unix_candidates = [
        cache_directory.join("lib").join("libraylib.a"),
        cache_directory.join("raylib").join("libraylib.a"),
        cache_directory
            .join("_deps")
            .join("raylib-build")
            .join("raylib")
            .join("libraylib.a"),
        cache_directory.join("libraylib.a"),
    ];

    let (primary, secondary) = if prefer_msvc {
        (&msvc_candidates[..], &unix_candidates[..])
    } else {
        (&unix_candidates[..], &msvc_candidates[..])
    };

    for candidate in primary.iter().chain(secondary.iter()) {
        if candidate.exists() {
            return Some(candidate.clone());
        }
    }
    None
}

fn main() {
    let manifest_directory = strip_windows_verbatim_prefix(PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"),
    ));
    let project_root = manifest_directory
        .parent()
        .expect("natives directory")
        .parent()
        .expect("raylib root directory")
        .to_path_buf();

    let target_triple = env::var("TARGET").unwrap_or_else(|_| "host".to_string());
    let host_triple = env::var("HOST").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let prefer_msvc = target_env == "msvc" || target_triple.contains("-msvc");
    let cmake_base_cache = project_root.join("build").join(".cache").join("cmake");

    let mut candidate_dirs = vec![
        cmake_base_cache.join(format!("target-{target_triple}")),
        cmake_base_cache.join(&target_triple),
    ];
    if host_triple.is_empty() || host_triple == target_triple {
        candidate_dirs.push(cmake_base_cache.join("target-host"));
        candidate_dirs.push(cmake_base_cache.join("host"));
    }

    let library_file_option = if let Ok(custom_lib_directory) = env::var("RAYLIB_LIB_DIR") {
        find_libraylib(Path::new(&custom_lib_directory), prefer_msvc)
    } else {
        candidate_dirs
            .iter()
            .find_map(|dir_path| find_libraylib(dir_path, prefer_msvc))
    };

    let final_library_file = match library_file_option {
        Some(found_library_path) => found_library_path,
        None => {
            let target_cache_directory = cmake_base_cache.join(format!("target-{target_triple}"));
            let mut cmake_cmd = Command::new("cmake");
            cmake_cmd
                .arg("-B")
                .arg(&target_cache_directory)
                .arg("-S")
                .arg(&manifest_directory)
                .arg("-DBUILD_HOST_PARSER_ONLY=OFF")
                .arg("-DCMAKE_BUILD_TYPE=Release")
                .arg("-DCMAKE_POSITION_INDEPENDENT_CODE=ON")
                .arg("-DBUILD_EXAMPLES=OFF")
                .arg("-DBUILD_SHARED_LIBS=OFF");

            let underscored_triple = target_triple.replace('-', "_");
            let is_cross_target = !host_triple.is_empty() && target_triple != host_triple;

            let toolchain_file_env = env::var(format!("CMAKE_TOOLCHAIN_FILE_{underscored_triple}"))
                .or_else(|_| env::var("CMAKE_TOOLCHAIN_FILE"))
                .ok()
                .filter(|val| !val.trim().is_empty());
            if let Some(ref toolchain_path) = toolchain_file_env {
                if Path::new(toolchain_path).exists() {
                    cmake_cmd.arg(format!("-DCMAKE_TOOLCHAIN_FILE={toolchain_path}"));
                }
            }

            if is_cross_target {
                if target_triple.contains("-windows-") {
                    cmake_cmd.arg("-DCMAKE_SYSTEM_NAME=Windows");
                    cmake_cmd.arg("-DPLATFORM=Desktop");
                    if target_triple.starts_with("x86_64-") {
                        cmake_cmd.arg("-DCMAKE_SYSTEM_PROCESSOR=AMD64");
                    } else if target_triple.starts_with("aarch64-") {
                        cmake_cmd.arg("-DCMAKE_SYSTEM_PROCESSOR=ARM64");
                    }
                } else if target_triple.contains("-linux-") {
                    cmake_cmd.arg("-DCMAKE_SYSTEM_NAME=Linux");
                    if target_triple.starts_with("aarch64-") {
                        cmake_cmd.arg("-DCMAKE_SYSTEM_PROCESSOR=aarch64");
                    } else if target_triple.starts_with("x86_64-") {
                        cmake_cmd.arg("-DCMAKE_SYSTEM_PROCESSOR=x86_64");
                    }
                } else if target_triple.contains("-apple-darwin") {
                    cmake_cmd.arg("-DCMAKE_SYSTEM_NAME=Darwin");
                }
            }

            let target_cc = env::var(format!("CC_{underscored_triple}"))
                .or_else(|_| env::var("TARGET_CC"))
                .ok()
                .filter(|val| !val.trim().is_empty());
            if let Some(c_compiler) = target_cc {
                cmake_cmd.arg(format!("-DCMAKE_C_COMPILER={c_compiler}"));
            }
            let target_cxx = env::var(format!("CXX_{underscored_triple}"))
                .or_else(|_| env::var("TARGET_CXX"))
                .ok()
                .filter(|val| !val.trim().is_empty());
            if let Some(cxx_compiler) = target_cxx {
                cmake_cmd.arg(format!("-DCMAKE_CXX_COMPILER={cxx_compiler}"));
            }
            let target_ar = env::var(format!("AR_{underscored_triple}"))
                .or_else(|_| env::var("TARGET_AR"))
                .ok()
                .filter(|val| !val.trim().is_empty());
            if let Some(archiver) = target_ar {
                cmake_cmd.arg(format!("-DCMAKE_AR={archiver}"));
            }
            let target_ranlib = env::var(format!("RANLIB_{underscored_triple}"))
                .or_else(|_| env::var("TARGET_RANLIB"))
                .ok()
                .filter(|val| !val.trim().is_empty());
            if let Some(ranlib_tool) = target_ranlib {
                cmake_cmd.arg(format!("-DCMAKE_RANLIB={ranlib_tool}"));
            }

            if target_triple == "x86_64-apple-darwin" {
                cmake_cmd.arg("-DCMAKE_OSX_ARCHITECTURES=x86_64");
            } else if target_triple == "aarch64-apple-darwin" {
                cmake_cmd.arg("-DCMAKE_OSX_ARCHITECTURES=arm64");
            }

            let configure_status = cmake_cmd
                .status()
                .expect("Failed to execute cmake configure");

            if !configure_status.success() {
                panic!("cmake configure failed with status: {:?}", configure_status);
            }

            let build_status = Command::new("cmake")
                .arg("--build")
                .arg(&target_cache_directory)
                .arg("--config")
                .arg("Release")
                .arg("--target")
                .arg("raylib")
                .status()
                .expect("Failed to execute cmake build");

            if !build_status.success() {
                panic!("cmake build failed with status: {:?}", build_status);
            }

            find_libraylib(&target_cache_directory, prefer_msvc)
                .expect("raylib static library must exist after cmake build")
        }
    };

    let search_directory = final_library_file.parent().expect("library directory");

    println!(
        "cargo:rustc-link-search=native={}",
        search_directory.display()
    );
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
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=QuartzCore");
            println!("cargo:rustc-link-lib=framework=CoreAudio");
            println!("cargo:rustc-link-lib=framework=CoreVideo");
            println!("cargo:rustc-link-lib=framework=AudioToolbox");
        }
        other_os => {
            println!("cargo:warning=Untested target OS for raylib: {}", other_os);
        }
    }
}
