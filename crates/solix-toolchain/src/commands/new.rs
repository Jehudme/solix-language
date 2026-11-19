//! The `solix new` project and native bridge scaffolding command driver.
//!
//! Generates declarative `manifest.yaml` Solix projects and dual-mode C, C++, or Rust
//! native subprojects (`solix new native`) directly in Rust with zero Python boilerplate.

#![forbid(unsafe_code)]

use crate::DEFAULT_FFI_GIT_TAG;
use crate::cli::{NewArgs, NewNativeArgs, NewSubcommand};
use anyhow::{Context, Result, bail};
use solix_common::project::{NativeTargetConfig, ProjectManifest, find_manifest_path};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Supported native subproject implementation languages/backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeLanguage {
    Rust,
    CLang,
    Cpp,
}

impl NativeLanguage {
    /// Parses a backend/language flag value case-insensitively.
    pub fn parse(raw: &str) -> Result<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "rust" | "rs" | "cargo" => Ok(Self::Rust),
            "c" | "cmake" => Ok(Self::CLang),
            "cpp" | "c++" | "cxx" => Ok(Self::Cpp),
            other => bail!(
                "Unsupported native backend language '{other}'. Expected 'rust', 'c', or 'cpp'."
            ),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::CLang => "c",
            Self::Cpp => "cpp",
        }
    }
}

/// Converts a raw name into PascalCase for Solix class identifiers.
#[must_use]
fn to_pascal_case(raw_name: &str) -> String {
    let mut output = String::new();
    for segment in raw_name.split(|ch: char| !ch.is_ascii_alphanumeric()) {
        if segment.is_empty() {
            continue;
        }
        let mut chars = segment.chars();
        if let Some(first_char) = chars.next() {
            output.push(first_char.to_ascii_uppercase());
            for ch in chars {
                output.push(ch.to_ascii_lowercase());
            }
        }
    }
    if output.is_empty() {
        "App".to_string()
    } else {
        output
    }
}

/// Converts a raw name into lowercase snake_case suitable for package/crate/target names.
#[must_use]
fn to_snake_case(raw_name: &str) -> String {
    let parts: Vec<String> = raw_name
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_ascii_lowercase())
        .collect();
    if parts.is_empty() {
        "app".to_string()
    } else {
        parts.join("_")
    }
}

/// Executes `solix new` or delegates `solix new native`.
///
/// # Errors
/// Returns an error if required arguments are missing, existing files conflict without
/// `--force`, or filesystem scaffolding fails.
pub fn execute_new(args: NewArgs) -> Result<()> {
    if let Some(NewSubcommand::Native(native_args)) = args.subcommand {
        return execute_new_native(native_args);
    }

    let Some(raw_target_path) = args.path else {
        bail!(
            "Missing required project path. Usage: `solix new <PATH>` or `solix new native <NAME>`"
        );
    };

    let target_path = solix_common::canonicalize_host_path(&raw_target_path)
        .unwrap_or_else(|_| raw_target_path.clone());
    let fallback_name = raw_target_path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("untitled");
    let project_name = args
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or(fallback_name)
        .to_string();

    if let Some(existing_manifest) = find_manifest_path(&target_path)
        && !args.force
    {
        bail!(
            "Target directory '{}' already contains a Solix project ({}). Use --force to overwrite.",
            target_path.display(),
            existing_manifest
                .file_name()
                .and_then(|val| val.to_str())
                .unwrap_or("manifest.yaml")
        );
    }

    let include_native = !args.no_native;
    let native_lang = if include_native {
        Some(NativeLanguage::parse(
            args.native.as_deref().unwrap_or("rust"),
        )?)
    } else {
        None
    };

    let parsed_deps = parse_dependency_flags(&args.deps);
    let safe_snake_name = to_snake_case(&project_name);
    let bridge_folder_name = format!("{safe_snake_name}_bridge");

    fs::create_dir_all(target_path.join("src"))?;
    fs::create_dir_all(target_path.join("tests"))?;
    fs::create_dir_all(target_path.join("resources"))?;
    fs::create_dir_all(target_path.join("lib"))?;
    fs::write(target_path.join("lib").join(".gitkeep"), "")?;
    fs::write(
        target_path.join("resources").join("config.txt"),
        format!("# Solix resources directory for {project_name}\n"),
    )?;
    fs::write(
        target_path.join(".gitignore"),
        "/build/\n*.slxb\n*.slxarc\n",
    )?;

    let manifest_yaml = render_project_manifest_yaml(&ProjectManifestRenderSpec {
        project_name: &project_name,
        version: &args.version,
        description: &args.description,
        license: &args.license,
        authors: &args.authors,
        main_entry: &args.main,
        test_entry: &args.test,
        is_library: args.lib,
        native_key: native_lang.map(|_| bridge_folder_name.as_str()),
        dependencies: &parsed_deps,
    });
    fs::write(target_path.join("manifest.yaml"), manifest_yaml)?;

    write_project_solix_sources(
        &target_path,
        &project_name,
        &safe_snake_name,
        args.lib,
        include_native,
    )?;

    if let Some(lang) = native_lang {
        let native_dir = target_path.join("natives").join(&bridge_folder_name);
        scaffold_native_directory(&NativeScaffoldSpec {
            target_dir: &native_dir,
            package_name: &bridge_folder_name,
            fn_name: "fast_add",
            solix_symbol: "MathBridge::fast_add",
            c_symbol_prefix: &bridge_folder_name,
            version: &args.version,
            description: &format!("Dual-mode native bridge for {project_name}"),
            license: &args.license,
            authors: &args.authors,
            lang,
            force: args.force,
        })?;
    } else {
        fs::create_dir_all(target_path.join("natives"))?;
        fs::write(target_path.join("natives").join(".gitkeep"), "")?;
    }

    println!(
        "Initialized Solix project '{project_name}' at '{}'",
        target_path.display()
    );
    Ok(())
}

/// Executes `solix new native <name>`, scaffolding a standalone/dual-mode C, C++, or Rust library
/// and optionally registering it into `<project>/manifest.yaml`.
///
/// # Errors
/// Returns an error if the language is invalid, `--project` lacks a YAML manifest, target files
/// already exist without `--force`, or writing/updating files fails.
pub fn execute_new_native(args: NewNativeArgs) -> Result<()> {
    let lang = NativeLanguage::parse(&args.lang)?;
    let package_name = args
        .package_name
        .as_deref()
        .map(str::trim)
        .filter(|val| !val.is_empty())
        .map(to_snake_case)
        .unwrap_or_else(|| to_snake_case(&args.name));
    let class_prefix = format!("{}Native", to_pascal_case(&args.name));
    let solix_symbol = format!("{class_prefix}::add");

    let cwd = std::env::current_dir().context("failed to read current working directory")?;
    let explicit_project_root = args
        .project
        .as_ref()
        .map(|path_arg| resolve_abs_path(&cwd, path_arg));

    let detected_project_root = match explicit_project_root {
        Some(ref root) => {
            if find_manifest_path(root).is_none() {
                bail!(
                    "Specified --project directory '{}' does not contain manifest.yaml (or manifest.toml)",
                    root.display()
                );
            }
            Some(root.clone())
        }
        None if args.output.is_none() && find_manifest_path(&cwd).is_some() => Some(cwd.clone()),
        None => None,
    };

    let native_dir = match &args.output {
        Some(out_path) => resolve_abs_path(&cwd, out_path),
        None => match &detected_project_root {
            Some(project_root) => project_root.join("natives").join(&package_name),
            None => cwd.join(&package_name),
        },
    };

    scaffold_native_directory(&NativeScaffoldSpec {
        target_dir: &native_dir,
        package_name: &package_name,
        fn_name: "add",
        solix_symbol: &solix_symbol,
        c_symbol_prefix: &package_name,
        version: &args.version,
        description: &args.description,
        license: &args.license,
        authors: &args.authors,
        lang,
        force: args.force,
    })?;

    if !args.no_register
        && let Some(ref project_root) = detected_project_root
    {
        register_native_in_manifest(
            project_root,
            &package_name,
            &native_dir,
            &args.profiles,
            &args.platforms,
        )?;
    }

    println!(
        "Initialized {} native project '{}' at '{}'",
        lang.label(),
        package_name,
        native_dir.display()
    );
    Ok(())
}

fn resolve_abs_path(base: &Path, input: &Path) -> PathBuf {
    if input.is_absolute() {
        input.to_path_buf()
    } else {
        base.join(input)
    }
}

fn parse_dependency_flags(deps: &[String]) -> BTreeMap<String, String> {
    let mut parsed = BTreeMap::new();
    for raw_item in deps {
        let (dep_name, dep_ver) = if let Some((left, right)) = raw_item.split_once('@') {
            (left, right)
        } else if let Some((left, right)) = raw_item.split_once(':') {
            (left, right)
        } else if let Some((left, right)) = raw_item.split_once('=') {
            (left, right)
        } else {
            (raw_item.as_str(), "*")
        };
        let cleaned_name = dep_name.trim();
        let cleaned_ver = dep_ver.trim();
        if !cleaned_name.is_empty() {
            parsed.insert(
                cleaned_name.to_string(),
                if cleaned_ver.is_empty() {
                    "*".to_string()
                } else {
                    cleaned_ver.to_string()
                },
            );
        }
    }
    parsed
}

struct ProjectManifestRenderSpec<'a> {
    project_name: &'a str,
    version: &'a str,
    description: &'a str,
    license: &'a str,
    authors: &'a [String],
    main_entry: &'a str,
    test_entry: &'a str,
    is_library: bool,
    native_key: Option<&'a str>,
    dependencies: &'a BTreeMap<String, String>,
}

fn render_project_manifest_yaml(spec: &ProjectManifestRenderSpec<'_>) -> String {
    let mut lines = Vec::new();
    lines.push("vars:".to_string());
    lines.push(format!(
        "  project_version: &proj_version \"{}\"",
        spec.version
    ));
    lines.push("  default_platforms: &host_platform".to_string());
    lines.push("    - host".to_string());
    lines.push(String::new());

    lines.push("configuration:".to_string());
    lines.push(format!("  name: {}", spec.project_name));
    lines.push("  version: *proj_version".to_string());
    lines.push(format!(
        "  description: {}",
        yaml_quoted_scalar(spec.description)
    ));
    lines.push(format!("  license: {}", yaml_quoted_scalar(spec.license)));
    if spec.authors.is_empty() {
        lines.push("  authors: []".to_string());
    } else {
        lines.push("  authors:".to_string());
        for author in spec.authors {
            lines.push(format!("    - {}", yaml_quoted_scalar(author)));
        }
    }
    lines.push(String::new());

    lines.push("profiles:".to_string());
    lines.push("  release:".to_string());
    if !spec.is_library {
        lines.push(format!("    entrypoint: {}", spec.main_entry));
    }
    lines.extend([
        "    optimization_level: 3".to_string(),
        "    inline_threshold: 16".to_string(),
        "    debug_symbols: false".to_string(),
        "    static: true".to_string(),
        "    sources:".to_string(),
        "      - src".to_string(),
        "    platforms: *host_platform".to_string(),
    ]);
    if let Some(key) = spec.native_key {
        lines.extend([
            "    natives:".to_string(),
            format!("      {key}:"),
            "        profile: release".to_string(),
        ]);
    }
    append_dependencies_yaml(&mut lines, spec.dependencies);
    lines.push(String::new());

    lines.push("  debug:".to_string());
    if !spec.is_library {
        lines.push(format!("    entrypoint: {}", spec.main_entry));
    }
    lines.extend([
        "    optimization_level: 0".to_string(),
        "    inline_threshold: 8".to_string(),
        "    debug_symbols: true".to_string(),
        "    static: false".to_string(),
        "    sources:".to_string(),
        "      - src".to_string(),
        "    platforms: *host_platform".to_string(),
    ]);
    if let Some(key) = spec.native_key {
        lines.extend([
            "    natives:".to_string(),
            format!("      {key}:"),
            "        profile: debug".to_string(),
        ]);
    }
    append_dependencies_yaml(&mut lines, spec.dependencies);
    lines.push(String::new());

    lines.push("  test:".to_string());
    lines.extend([
        format!("    entrypoint: {}", spec.test_entry),
        "    optimization_level: 0".to_string(),
        "    inline_threshold: 8".to_string(),
        "    debug_symbols: true".to_string(),
        "    static: false".to_string(),
        "    sources:".to_string(),
        "      - src".to_string(),
        "      - tests".to_string(),
        "    platforms: *host_platform".to_string(),
    ]);
    if let Some(key) = spec.native_key {
        lines.extend([
            "    natives:".to_string(),
            format!("      {key}:"),
            "        profile: debug".to_string(),
        ]);
    }
    append_dependencies_yaml(&mut lines, spec.dependencies);
    lines.push(String::new());
    lines.join("\n")
}

fn append_dependencies_yaml(lines: &mut Vec<String>, dependencies: &BTreeMap<String, String>) {
    if dependencies.is_empty() {
        return;
    }
    lines.push("    dependencies:".to_string());
    for (dep_name, dep_ver) in dependencies {
        lines.push(format!("      {dep_name}:"));
        lines.push(format!("        version: \"{}\"", dep_ver));
        lines.push("        profile: release".to_string());
    }
}

fn yaml_quoted_scalar(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn write_project_solix_sources(
    target_path: &Path,
    project_name: &str,
    safe_snake_name: &str,
    is_library: bool,
    include_native: bool,
) -> Result<()> {
    let main_class_name = to_pascal_case(project_name);

    let main_source = if include_native {
        if is_library {
            format!(
                r#"package {safe_snake_name};

public class MathBridge {{
    native public static int32 fast_add(int32 left_value, int32 right_value);
}}

public class {main_class_name}Api {{
    public static int32 add_numbers(int32 value_a, int32 value_b) {{
        return MathBridge::fast_add(value_a, value_b);
    }}
}}
"#
            )
        } else {
            format!(
                r#"package {safe_snake_name};

public class MathBridge {{
    native public static int32 fast_add(int32 left_value, int32 right_value);
}}

public class Main {{
    public static int32 main() {{
        int32 computed_sum = MathBridge::fast_add(15, 27);
        return computed_sum;
    }}
}}
"#
            )
        }
    } else if is_library {
        format!(
            r#"package {safe_snake_name};

public class {main_class_name}Api {{
    public static int32 add_numbers(int32 value_a, int32 value_b) {{
        return value_a + value_b;
    }}
}}
"#
        )
    } else {
        format!(
            r#"package {safe_snake_name};

public class Main {{
    public static int32 main() {{
        int32 value_a = 15;
        int32 value_b = 27;
        return value_a + value_b;
    }}
}}
"#
        )
    };

    let main_filename = if is_library {
        "lib.slx".to_string()
    } else {
        "main.slx".to_string()
    };
    fs::write(target_path.join("src").join(main_filename), main_source)?;

    let test_source = if is_library {
        format!(
            r#"package {safe_snake_name}.tests;

import {safe_snake_name}.{main_class_name}Api;

public class TestRunner {{
    public static int32 test() {{
        int32 sum_result = {main_class_name}Api::add_numbers(10, 32);
        if (sum_result != 42) {{
            return 1;
        }}
        return 0;
    }}
}}
"#
        )
    } else {
        format!(
            r#"package {safe_snake_name}.tests;

import {safe_snake_name}.Main;

public class TestRunner {{
    public static int32 test() {{
        int32 exit_code = Main::main();
        if (exit_code != 42) {{
            return 1;
        }}
        return 0;
    }}
}}
"#
        )
    };

    fs::write(target_path.join("tests").join("test_main.slx"), test_source)?;
    Ok(())
}

struct NativeScaffoldSpec<'a> {
    target_dir: &'a Path,
    package_name: &'a str,
    fn_name: &'a str,
    solix_symbol: &'a str,
    c_symbol_prefix: &'a str,
    version: &'a str,
    description: &'a str,
    license: &'a str,
    authors: &'a [String],
    lang: NativeLanguage,
    force: bool,
}

fn scaffold_native_directory(spec: &NativeScaffoldSpec<'_>) -> Result<()> {
    let target = spec.target_dir;
    let sentinel = match spec.lang {
        NativeLanguage::Rust => target.join("Cargo.toml"),
        NativeLanguage::CLang | NativeLanguage::Cpp => target.join("CMakeLists.txt"),
    };
    if !spec.force && sentinel.exists() {
        bail!(
            "Native directory '{}' already exists (found '{}'). Use --force to overwrite.",
            target.display(),
            sentinel.display()
        );
    }

    match spec.lang {
        NativeLanguage::Rust => write_rust_native_template(spec),
        NativeLanguage::CLang => write_c_native_template(spec),
        NativeLanguage::Cpp => write_cpp_native_template(spec),
    }
}

fn rust_ffi_dependency_spec() -> String {
    let tag = DEFAULT_FFI_GIT_TAG.trim();
    if tag.starts_with('v') {
        format!(
            "solix-ffi = {{ git = \"https://github.com/Jehudme/solix-language.git\", tag = \"{tag}\", optional = true }}"
        )
    } else {
        format!(
            "solix-ffi = {{ git = \"https://github.com/Jehudme/solix-language.git\", branch = \"{tag}\", optional = true }}"
        )
    }
}

fn write_rust_native_template(spec: &NativeScaffoldSpec<'_>) -> Result<()> {
    let src_dir = spec.target_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    fs::write(spec.target_dir.join(".gitignore"), "/target/\n")?;

    let mut author_entries = String::new();
    if !spec.authors.is_empty() {
        let joined: Vec<String> = spec
            .authors
            .iter()
            .map(|item| format!("\"{}\"", item.replace('"', "\\\"")))
            .collect();
        author_entries = format!("authors = [{}]\n", joined.join(", "));
    }

    let cargo_toml = format!(
        r#"[package]
name = "{pkg}"
version = "{ver}"
edition = "2024"
description = "{desc}"
license = "{lic}"
{authors}
[workspace]

[lib]
crate-type = ["cdylib", "rlib"]

[features]
solix-native = ["dep:solix-ffi"]

[dependencies]
{ffi_dep}
"#,
        pkg = spec.package_name,
        ver = spec.version,
        desc = spec.description.replace('"', "\\\""),
        lic = spec.license.replace('"', "\\\""),
        authors = author_entries,
        ffi_dep = rust_ffi_dependency_spec(),
    );
    fs::write(spec.target_dir.join("Cargo.toml"), cargo_toml)?;

    let lib_rs = format!(
        r#"//! Dual-mode Rust library and Solix native extension (`{pkg}`).
//!
//! - Standard Cargo build (`cargo build` / `cargo test`): compiles as a regular Rust library.
//! - Solix native plugin build (`--features solix-native`, passed automatically by `solix build`):
//!   exports `solix_register_natives` using `solix_ffi`.

/// Core library function usable directly from Rust without Solix runtime dependencies.
#[must_use]
pub fn {fn_name}(left_value: i32, right_value: i32) -> i32 {{
    left_value + right_value
}}

#[cfg(feature = "solix-native")]
mod solix_bridge {{
    use solix_ffi::{{SolixCValue, solix_export}};
    use std::ffi::c_void;

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn solix_native_entry(
        _vm_context: *mut c_void,
        arguments: *const SolixCValue,
        argument_count: u32,
        _out_error: *mut SolixCValue,
    ) -> SolixCValue {{
        if argument_count < 2 || arguments.is_null() {{
            return SolixCValue::from_i32(0);
        }}
        let left_value = unsafe {{ (*arguments.add(0)).as_i32() }};
        let right_value = unsafe {{ (*arguments.add(1)).as_i32() }};
        SolixCValue::from_i32(super::{fn_name}(left_value, right_value))
    }}

    solix_export! {{
        "{symbol}" => solix_native_entry,
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_{fn_name}_standalone() {{
        assert_eq!({fn_name}(15, 27), 42);
    }}
}}
"#,
        pkg = spec.package_name,
        fn_name = spec.fn_name,
        symbol = spec.solix_symbol,
    );
    fs::write(src_dir.join("lib.rs"), lib_rs)?;
    Ok(())
}

fn cmake_fetchcontent_ffi_block() -> String {
    let git_tag = DEFAULT_FFI_GIT_TAG.trim();
    format!(
        r#"    include(FetchContent)
    if(DEFINED ENV{{SOLIX_FFI_SOURCE_DIR}} AND EXISTS "$ENV{{SOLIX_FFI_SOURCE_DIR}}/CMakeLists.txt")
        file(TO_CMAKE_PATH "$ENV{{SOLIX_FFI_SOURCE_DIR}}" SOLIX_FFI_LOCAL_DIR)
        FetchContent_Declare(
            solix_ffi
            SOURCE_DIR "${{SOLIX_FFI_LOCAL_DIR}}"
        )
    else()
        FetchContent_Declare(
            solix_ffi
            GIT_REPOSITORY https://github.com/Jehudme/solix-language.git
            GIT_TAG        {git_tag}
            SOURCE_SUBDIR  ffi/c
            GIT_SHALLOW    TRUE
        )
    endif()
    FetchContent_MakeAvailable(solix_ffi)"#
    )
}

fn write_c_native_template(spec: &NativeScaffoldSpec<'_>) -> Result<()> {
    let include_dir = spec.target_dir.join("include");
    let src_dir = spec.target_dir.join("src");
    let tests_dir = spec.target_dir.join("tests");
    fs::create_dir_all(&include_dir)?;
    fs::create_dir_all(&src_dir)?;
    fs::create_dir_all(&tests_dir)?;
    fs::write(spec.target_dir.join(".gitignore"), "/build/\n")?;

    let pkg = spec.package_name;
    let c_fn = format!("{}_{}", spec.c_symbol_prefix, spec.fn_name);
    let header_guard = format!("{}_H", pkg.to_ascii_uppercase());

    let cmakelists = format!(
        r#"cmake_minimum_required(VERSION 3.20)
project({pkg} VERSION {ver} DESCRIPTION "{desc}" LANGUAGES C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)

option(SOLIX_NATIVE_BUILD "Build as a Solix VM native shared library plugin" OFF)
option(BUILD_TESTING "Build standalone unit tests" ON)

if(SOLIX_NATIVE_BUILD)
{fetch_block}

    add_library({pkg} SHARED
        src/{pkg}.c
        src/solix_bridge.c
    )
    target_link_libraries({pkg} PRIVATE solix::ffi)
    target_compile_definitions({pkg} PRIVATE SOLIX_NATIVE_BUILD=1)
else()
    add_library({pkg}
        src/{pkg}.c
    )
    if(BUILD_TESTING)
        enable_testing()
        add_executable({pkg}_tests tests/test_{pkg}.c)
        target_link_libraries({pkg}_tests PRIVATE {pkg})
        add_test(NAME {pkg}_unit_tests COMMAND {pkg}_tests)
    endif()
endif()

target_include_directories({pkg} PUBLIC
    "${{CMAKE_CURRENT_SOURCE_DIR}}/include"
)
"#,
        pkg = pkg,
        ver = spec.version,
        desc = spec.description.replace('"', "\\\""),
        fetch_block = cmake_fetchcontent_ffi_block(),
    );
    fs::write(spec.target_dir.join("CMakeLists.txt"), cmakelists)?;

    let header_h = format!(
        r#"#ifndef {header_guard}
#define {header_guard}

#include <stdint.h>

#ifdef __cplusplus
extern "C" {{
#endif

int32_t {c_fn}(int32_t left_value, int32_t right_value);

#ifdef __cplusplus
}}
#endif

#endif /* {header_guard} */
"#
    );
    fs::write(include_dir.join(format!("{pkg}.h")), header_h)?;

    let impl_c = format!(
        r#"#include "{pkg}.h"

int32_t {c_fn}(int32_t left_value, int32_t right_value) {{
    return left_value + right_value;
}}
"#
    );
    fs::write(src_dir.join(format!("{pkg}.c")), impl_c)?;

    let bridge_c = format!(
        r#"#include "solix_ffi.h"
#include "{pkg}.h"

static SolixCValue bridge_native_entry(
    void* raw_vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
) {{
    (void)raw_vm_context;
    (void)out_error;
    if (argument_count < 2 || arguments == NULL) {{
        return solix_cvalue_i32(0);
    }}
    int32_t left_value = solix_cvalue_as_i32(arguments[0]);
    int32_t right_value = solix_cvalue_as_i32(arguments[1]);
    return solix_cvalue_i32({c_fn}(left_value, right_value));
}}

static const SolixNativeEntry {upper_pkg}_NATIVE_ENTRIES[] = {{
    {{"{symbol}", bridge_native_entry}},
}};

SOLIX_REGISTER_MODULE({upper_pkg}_NATIVE_ENTRIES)
"#,
        pkg = pkg,
        c_fn = c_fn,
        upper_pkg = pkg.to_ascii_uppercase(),
        symbol = spec.solix_symbol,
    );
    fs::write(src_dir.join("solix_bridge.c"), bridge_c)?;

    let test_c = format!(
        r#"#include "{pkg}.h"
#include <stdio.h>

int main(void) {{
    int32_t computed = {c_fn}(15, 27);
    if (computed != 42) {{
        fprintf(stderr, "Expected 42, got %d\n", computed);
        return 1;
    }}
    return 0;
}}
"#
    );
    fs::write(tests_dir.join(format!("test_{pkg}.c")), test_c)?;
    Ok(())
}

fn write_cpp_native_template(spec: &NativeScaffoldSpec<'_>) -> Result<()> {
    let include_dir = spec.target_dir.join("include");
    let src_dir = spec.target_dir.join("src");
    let tests_dir = spec.target_dir.join("tests");
    fs::create_dir_all(&include_dir)?;
    fs::create_dir_all(&src_dir)?;
    fs::create_dir_all(&tests_dir)?;
    fs::write(spec.target_dir.join(".gitignore"), "/build/\n")?;

    let pkg = spec.package_name;
    let ns_name = pkg;
    let fn_name = spec.fn_name;
    let header_guard = format!("{}_HPP", pkg.to_ascii_uppercase());

    let cmakelists = format!(
        r#"cmake_minimum_required(VERSION 3.20)
project({pkg} VERSION {ver} DESCRIPTION "{desc}" LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

option(SOLIX_NATIVE_BUILD "Build as a Solix VM native shared library plugin" OFF)
option(BUILD_TESTING "Build standalone unit tests" ON)

if(SOLIX_NATIVE_BUILD)
{fetch_block}

    add_library({pkg} SHARED
        src/{pkg}.cpp
        src/solix_bridge.cpp
    )
    target_link_libraries({pkg} PRIVATE solix::ffi)
    target_compile_definitions({pkg} PRIVATE SOLIX_NATIVE_BUILD=1)
else()
    add_library({pkg}
        src/{pkg}.cpp
    )
    if(BUILD_TESTING)
        enable_testing()
        add_executable({pkg}_tests tests/test_{pkg}.cpp)
        target_link_libraries({pkg}_tests PRIVATE {pkg})
        add_test(NAME {pkg}_unit_tests COMMAND {pkg}_tests)
    endif()
endif()

target_include_directories({pkg} PUBLIC
    "${{CMAKE_CURRENT_SOURCE_DIR}}/include"
)
"#,
        pkg = pkg,
        ver = spec.version,
        desc = spec.description.replace('"', "\\\""),
        fetch_block = cmake_fetchcontent_ffi_block(),
    );
    fs::write(spec.target_dir.join("CMakeLists.txt"), cmakelists)?;

    let header_hpp = format!(
        r#"#ifndef {header_guard}
#define {header_guard}

#include <cstdint>

namespace {ns_name} {{

std::int32_t {fn_name}(std::int32_t left_value, std::int32_t right_value);

}} // namespace {ns_name}

#endif // {header_guard}
"#
    );
    fs::write(include_dir.join(format!("{pkg}.hpp")), header_hpp)?;

    let impl_cpp = format!(
        r#"#include "{pkg}.hpp"

namespace {ns_name} {{

std::int32_t {fn_name}(std::int32_t left_value, std::int32_t right_value) {{
    return left_value + right_value;
}}

}} // namespace {ns_name}
"#
    );
    fs::write(src_dir.join(format!("{pkg}.cpp")), impl_cpp)?;

    let bridge_cpp = format!(
        r#"#include "solix_ffi.h"
#include "{pkg}.hpp"

static SolixCValue bridge_native_entry(
    void* raw_vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
) {{
    (void)raw_vm_context;
    (void)out_error;
    if (argument_count < 2 || arguments == nullptr) {{
        return solix_cvalue_i32(0);
    }}
    std::int32_t left_value = solix_cvalue_as_i32(arguments[0]);
    std::int32_t right_value = solix_cvalue_as_i32(arguments[1]);
    return solix_cvalue_i32({ns_name}::{fn_name}(left_value, right_value));
}}

static const SolixNativeEntry {upper_pkg}_NATIVE_ENTRIES[] = {{
    {{"{symbol}", bridge_native_entry}},
}};

SOLIX_REGISTER_MODULE({upper_pkg}_NATIVE_ENTRIES)
"#,
        pkg = pkg,
        ns_name = ns_name,
        fn_name = fn_name,
        upper_pkg = pkg.to_ascii_uppercase(),
        symbol = spec.solix_symbol,
    );
    fs::write(src_dir.join("solix_bridge.cpp"), bridge_cpp)?;

    let test_cpp = format!(
        r#"#include "{pkg}.hpp"
#include <iostream>

int main() {{
    const std::int32_t computed = {ns_name}::{fn_name}(15, 27);
    if (computed != 42) {{
        std::cerr << "Expected 42, got " << computed << "\n";
        return 1;
    }}
    return 0;
}}
"#
    );
    fs::write(tests_dir.join(format!("test_{pkg}.cpp")), test_cpp)?;
    Ok(())
}

fn register_native_in_manifest(
    project_root: &Path,
    package_name: &str,
    native_dir: &Path,
    requested_profiles: &[String],
    requested_platforms: &[String],
) -> Result<()> {
    let manifest_path = find_manifest_path(project_root).ok_or_else(|| {
        anyhow::anyhow!(
            "Cannot register native project in '{}': manifest.yaml not found",
            project_root.display()
        )
    })?;

    if manifest_path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
    {
        bail!(
            "Project '{}' uses legacy manifest.toml; migrate it to manifest.yaml to register new natives via `solix new native`.",
            project_root.display()
        );
    }

    let mut manifest =
        ProjectManifest::from_file(&manifest_path).map_err(|error| anyhow::anyhow!("{error}"))?;

    let default_expected_dir = project_root.join("natives").join(package_name);
    let custom_rel_path = if native_dir == default_expected_dir {
        None
    } else {
        Some(compute_relative_forward_path(project_root, native_dir))
    };

    let target_profiles: Vec<String> = if requested_profiles.is_empty() {
        ["release", "debug"]
            .into_iter()
            .filter(|prof| manifest.profiles.contains_key(*prof))
            .map(ToString::to_string)
            .collect()
    } else if requested_profiles
        .iter()
        .any(|prof| prof.trim().eq_ignore_ascii_case("all"))
    {
        manifest.profiles.keys().cloned().collect()
    } else {
        requested_profiles
            .iter()
            .map(|prof| prof.trim().to_string())
            .filter(|prof| !prof.is_empty())
            .collect()
    };

    if target_profiles.is_empty() {
        bail!(
            "No target profile found in '{}' to register native module.",
            manifest_path.display()
        );
    }

    let platforms_opt = if requested_platforms.is_empty() {
        None
    } else {
        Some(requested_platforms.to_vec())
    };

    for prof_name in &target_profiles {
        let Some(profile_cfg) = manifest.profiles.get_mut(prof_name) else {
            bail!(
                "Profile '{}' does not exist in '{}'",
                prof_name,
                manifest_path.display()
            );
        };
        let native_profile = if prof_name == "debug" || prof_name == "test" {
            "debug"
        } else {
            "release"
        };
        profile_cfg.natives.insert(
            package_name.to_string(),
            NativeTargetConfig {
                path: custom_rel_path.clone(),
                profile: Some(native_profile.to_string()),
                platforms: platforms_opt.clone(),
                flags: Vec::new(),
            },
        );
    }

    let serialized = manifest
        .to_yaml_string()
        .map_err(|error| anyhow::anyhow!("{error}"))?;
    fs::write(&manifest_path, serialized)?;
    Ok(())
}

fn compute_relative_forward_path(base_dir: &Path, target_dir: &Path) -> String {
    if let Ok(rel) = target_dir.strip_prefix(base_dir) {
        return rel.display().to_string().replace('\\', "/");
    }
    target_dir.display().to_string().replace('\\', "/")
}
