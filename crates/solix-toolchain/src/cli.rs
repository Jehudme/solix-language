//! Command-line argument parser specifications for Solix.

#![forbid(unsafe_code)]

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// Root command-line parser definition for the Solix toolchain.
#[derive(Debug, Parser)]
#[command(
    name = "solix",
    author = "Jehud Metellus",
    version,
    about = "The Solix programming language compiler and runtime toolchain",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands in the Solix CLI driver.
#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Commands {
    /// Creates a new Solix project or native bridge
    New(NewArgs),

    /// Compiles Solix source files into an executable binary container (.slxb)
    Compile(CompileArgs),

    /// Builds a Solix project into a .slxp package container
    Build(BuildArgs),

    /// Executes compiled Solix binary containers or packages in the virtual machine
    Run(RunArgs),

    /// Inspects metadata, export manifests, and bytecode of an SLXB binary container or .slxp package
    Info(InfoArgs),

    /// Installs a .slxb binary or .slxp package into the local package store
    Install(InstallArgs),

    /// Uninstalls a package or binary from the local package store
    Uninstall(UninstallArgs),

    /// Lists all installed packages in the local package store
    List(ListArgs),

    /// Exports a packaged .slxp bundle or standalone compiled binary (.slxb)
    Export(ExportArgs),

    /// Starts the Solix Language Server Protocol (LSP) service
    Lsp(LspArgs),

    /// Starts the Solix Debug Adapter Protocol (DAP) service
    Dap(DapArgs),

    /// Formats Solix source files according to official style guidelines
    Fmt(FmtArgs),

    /// Generates static HTML or Markdown API documentation from source doc comments
    Doc(DocArgs),

    /// Opens the bundled offline Solix language documentation and wiki in your browser
    Wiki(WikiArgs),

    /// Removes project build artifacts and intermediate caches
    Clean(CleanArgs),

    /// Type-checks a Solix project without emitting bytecode
    Check(CheckArgs),

    /// Compiles and runs discovered `test_*` functions in a Solix project
    Test(TestArgs),

    /// Initializes a manifest.yaml in an existing directory without overwriting sources
    Init(InitArgs),

    /// Adds a package dependency to the project's manifest.yaml
    Add(AddArgs),
}

/// Arguments for `solix dap`.
#[derive(Debug, Args, Clone)]
pub struct DapArgs {
    /// Run debug adapter using standard input and output transport
    #[arg(long, default_value_t = true)]
    pub stdio: bool,

    /// Optional TCP port to listen on for remote debugger connections
    #[arg(long)]
    pub port: Option<u16>,

    /// Optional project root directory [default: current directory]
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

/// Arguments for `solix lsp`.
#[derive(Debug, Args, Clone)]
pub struct LspArgs {
    /// Run language server using standard input and output transport
    #[arg(long, default_value_t = true)]
    pub stdio: bool,

    /// Optional project root directory [default: current directory]
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,
}

/// Arguments for `solix fmt`.
#[derive(Debug, Args, Clone)]
pub struct FmtArgs {
    /// Solix source files or directories to format [default: current directory]
    #[arg(default_value = ".")]
    pub files: Vec<PathBuf>,

    /// Check if files are formatted without writing changes to disk (exits with code 1 if unformatted)
    #[arg(long)]
    pub check: bool,
}

/// Arguments for `solix doc`.
#[derive(Debug, Args, Clone)]
pub struct DocArgs {
    /// Workspace root or path to project manifest [default: current directory]
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Output directory for generated documentation [default: "docs/api"]
    #[arg(short, long, default_value = "docs/api")]
    pub output: PathBuf,

    /// Documentation format: html (default) or markdown
    #[arg(short, long, default_value = "html")]
    pub format: String,

    /// Open generated documentation in default web browser
    #[arg(long)]
    pub open: bool,

    /// Target build profile (e.g. "release", "debug")
    #[arg(short, long)]
    pub profile: Option<String>,
}

/// Arguments for `solix wiki`.
#[derive(Debug, Args, Clone)]
pub struct WikiArgs {
    /// Optional topic or search query to pre-filter in the documentation portal
    #[arg(value_name = "TOPIC")]
    pub query: Option<String>,

    /// Explicit search query flag (alternative to positional TOPIC)
    #[arg(long, alias = "topic", value_name = "QUERY")]
    pub search: Option<String>,

    /// Print the resolved local HTML entry point without launching a web browser
    #[arg(long)]
    pub no_open: bool,

    /// Export a copy of the offline HTML documentation portal to a target directory
    #[arg(short, long, value_name = "DIR")]
    pub output: Option<PathBuf>,

    /// Print only the resolved `index.html` filesystem path
    #[arg(long)]
    pub path: bool,
}

/// Arguments for `solix build`.
#[derive(Debug, Args, Clone)]
pub struct BuildArgs {
    /// Project root directory containing manifest.yaml and scripts/build.py [default: current directory]
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Filter native build targets by platform family (e.g. 'windows', 'linux', 'darwin', 'all')
    #[arg(long)]
    pub platform: Option<String>,

    /// Target build profile override: debug, release, or test
    #[arg(long)]
    pub profile: Option<String>,

    /// Shorthand flag for debug build profile
    #[arg(long, hide = true, conflicts_with_all = ["release", "test"])]
    pub debug: bool,

    /// Shorthand flag for release build profile
    #[arg(long, hide = true, conflicts_with_all = ["debug", "test"])]
    pub release: bool,

    /// Shorthand flag for test build profile
    #[arg(long, hide = true, conflicts_with_all = ["debug", "release"])]
    pub test: bool,

    /// Optional package store root directory override
    #[arg(long, hide = true)]
    pub pkg_dir: Option<PathBuf>,

    /// Automatically install the built archive container into the local package store
    #[arg(short = 'i', long, visible_alias = "sync")]
    pub install: bool,

    /// Force overwrite if package is already installed
    #[arg(short = 'f', long)]
    pub force: bool,

    /// Run only pre-flight environment and toolchain verification without building
    #[arg(long, visible_alias = "verify-only")]
    pub check_env: bool,

    /// Compile native artifacts for a single Rust target triple or 'host'
    #[arg(long)]
    pub target: Option<String>,
}

impl BuildArgs {
    /// Resolves the effective build profile string considering shorthand flags.
    #[must_use]
    pub fn resolved_profile(&self) -> &str {
        if self.release {
            "release"
        } else if self.test {
            "test"
        } else if self.debug {
            "debug"
        } else if let Some(ref prof) = self.profile {
            prof.as_str()
        } else {
            "debug"
        }
    }
}

/// Subcommands under `solix compile`.
#[derive(Debug, Subcommand, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum CompileSubcommand {
    /// Compiles raw Solix source files into an executable binary container (.slxb)
    Source(CompileSourceArgs),

    /// Compiles a package's sources into binaries.slxb
    Package(CompilePackageArgs),
}

/// Arguments for `solix compile source <files...>`.
#[derive(Debug, Args, Clone)]
pub struct CompileSourceArgs {
    /// Input Solix source files (*.slx)
    #[arg(value_name = "FILES", required = true)]
    pub sources: Vec<PathBuf>,

    /// Output compiled binary file path [default: <name>.slxb]
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Raw Solix source dependencies (*.slx) to include in compilation
    #[arg(long = "deps-source", visible_alias = "deps-file")]
    pub deps_source: Vec<PathBuf>,

    /// Package archive dependencies (*.slxp) whose API interfaces should be extracted
    #[arg(long = "deps-package")]
    pub deps_package: Vec<PathBuf>,

    /// Standalone compiled binary dependencies (*.slxb) whose API interfaces should be extracted
    #[arg(long = "deps-binaries")]
    pub deps_binaries: Vec<PathBuf>,

    /// Package dependency requirements formatted as <name>@<version> (e.g. raylib@1.0.0)
    #[arg(short = 'd', long = "deps", visible_alias = "dep")]
    pub deps: Vec<String>,

    /// Legacy library binary containers (*.slxb) or archives (*.slxp, *.slxarc)
    #[arg(short = 'l', long = "lib", hide = true)]
    pub libs: Vec<PathBuf>,

    /// Package or library name
    #[arg(short, long)]
    pub name: Option<String>,

    /// Package semantic version [default: 0.1.0]
    #[arg(short = 'V', long = "version", default_value = "0.1.0")]
    pub version: String,

    /// Human-readable package description
    #[arg(long)]
    pub description: Option<String>,

    /// Target build profile: debug, release (default), or test
    #[arg(long, default_value = "release")]
    pub profile: String,

    /// Save synthesized API interface source to disk (*.slx)
    #[arg(long)]
    pub emit_api: Option<PathBuf>,

    /// Perform syntax and semantic checks without emitting bytecode
    #[arg(long)]
    pub check: bool,

    /// Print wall-clock duration for each compilation phase
    #[arg(long)]
    pub time: bool,

    /// Print compiler throughput metrics and phase duration breakdown
    #[arg(long, value_name = "FORMAT", num_args = 0..=1, default_missing_value = "text", value_parser = ["text", "json"])]
    pub metrics: Option<String>,

    /// Output compiler metrics formatted as machine-readable JSON
    #[arg(long, hide = true)]
    pub metrics_json: bool,

    /// Inlining statement threshold limit for AstInlinerPhase
    #[arg(long, default_value_t = 8)]
    pub inline_threshold: usize,

    /// Explicit entry point function symbol (e.g. main or sample_workspace::main)
    #[arg(short = 'e', long = "entry")]
    pub entry: Option<String>,

    /// Linking mode for dependencies and executable emission: dynamic (default) or static
    #[arg(long, default_value = "dynamic")]
    pub link: String,

    /// Shorthand flag for static linking mode (--link static)
    #[arg(long)]
    pub r#static: bool,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,

    /// Automatically install the built archive container into the local package store
    #[arg(long, visible_alias = "sync")]
    pub install: bool,

    /// Force overwrite if package is already installed
    #[arg(short = 'f', long = "force-install", alias = "force")]
    pub force: bool,

    /// Bypass dependency cryptographic SHA-256 checksum verification
    #[arg(long, visible_alias = "ignore-checksum", alias = "yolo")]
    pub no_verify: bool,
}

impl CompileSourceArgs {
    /// Resolves the effective link mode considering the `--static` shorthand.
    #[must_use]
    pub fn resolved_link_mode(&self) -> solix_compilation::LinkMode {
        if self.r#static || self.link.eq_ignore_ascii_case("static") {
            solix_compilation::LinkMode::Static
        } else {
            solix_compilation::LinkMode::Dynamic
        }
    }
}

/// Arguments for `solix compile package [path]`.
#[derive(Debug, Args, Clone)]
pub struct CompilePackageArgs {
    /// Package root directory containing manifest.yaml [default: current directory]
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Target build profile (e.g. "release", "debug", "test") [default: "release"]
    #[arg(short, long, default_value = "release")]
    pub profile: String,

    /// Output binary container path [default: binaries.slxb]
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,

    /// Perform syntax and semantic checks without emitting bytecode
    #[arg(long)]
    pub check: bool,

    /// Print wall-clock duration for each compilation phase
    #[arg(long)]
    pub time: bool,

    /// Print compiler throughput metrics and phase duration breakdown
    #[arg(long, value_name = "FORMAT", num_args = 0..=1, default_missing_value = "text", value_parser = ["text", "json"])]
    pub metrics: Option<String>,

    /// Bypass dependency cryptographic SHA-256 checksum verification
    #[arg(long, visible_alias = "ignore-checksum", alias = "yolo")]
    pub no_verify: bool,
}

/// Arguments for `solix compile`.
#[derive(Debug, Args, Clone)]
#[command(args_conflicts_with_subcommands = true)]
pub struct CompileArgs {
    /// Subcommand: `source` or `package`
    #[command(subcommand)]
    pub subcommand: Option<CompileSubcommand>,

    /// Input Solix source files (*.slx) [fallback when not using subcommands]
    #[arg(value_name = "SOURCES")]
    pub sources: Vec<PathBuf>,

    /// Output compiled binary file path [default: <name>.slxb]
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Raw Solix source dependencies (*.slx)
    #[arg(long = "deps-source")]
    pub deps_source: Vec<PathBuf>,

    /// Package archive dependencies (*.slxp)
    #[arg(long = "deps-package")]
    pub deps_package: Vec<PathBuf>,

    /// Binary container dependencies (*.slxb)
    #[arg(long = "deps-binaries")]
    pub deps_binaries: Vec<PathBuf>,

    /// Package dependency requirements formatted as <name>@<version> (e.g. raylib@1.0.0)
    #[arg(short = 'd', long = "deps", visible_alias = "dep")]
    pub deps: Vec<String>,

    /// Paths to library binary containers (*.slxb) or archives (*.slxp, *.slxarc) whose API interfaces should be extracted
    #[arg(short = 'l', long = "deps-file", visible_alias = "lib")]
    pub libs: Vec<PathBuf>,

    /// Package or library name
    #[arg(short, long)]
    pub name: Option<String>,

    /// Package semantic version [default: 0.1.0]
    #[arg(short = 'V', long = "version", default_value = "0.1.0")]
    pub version: String,

    /// Human-readable package description
    #[arg(long)]
    pub description: Option<String>,

    /// Target build profile: debug, release (default), or test
    #[arg(long, default_value = "release")]
    pub profile: String,

    /// Save synthesized API interface source to disk (*.slx)
    #[arg(long)]
    pub emit_api: Option<PathBuf>,

    /// Perform syntax and semantic checks without emitting bytecode
    #[arg(long)]
    pub check: bool,

    /// Print wall-clock duration for each compilation phase
    #[arg(long)]
    pub time: bool,

    /// Print compiler throughput metrics and phase duration breakdown
    #[arg(long, value_name = "FORMAT", num_args = 0..=1, default_missing_value = "text", value_parser = ["text", "json"])]
    pub metrics: Option<String>,

    /// Output compiler metrics formatted as machine-readable JSON
    #[arg(long, hide = true)]
    pub metrics_json: bool,

    /// Inlining statement threshold limit for AstInlinerPhase
    #[arg(long, default_value_t = 8)]
    pub inline_threshold: usize,

    /// Explicit entry point function symbol (e.g. main or sample_workspace::main)
    #[arg(short = 'e', long = "entry")]
    pub entry: Option<String>,

    /// Linking mode for dependencies and executable emission: dynamic (default) or static
    #[arg(long, default_value = "dynamic")]
    pub link: String,

    /// Shorthand flag for static linking mode (--link static)
    #[arg(long)]
    pub r#static: bool,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,

    /// Automatically install the built archive container into the local package store
    #[arg(long, visible_alias = "sync")]
    pub install: bool,

    /// Force overwrite if package is already installed
    #[arg(short = 'f', long = "force-install", alias = "force")]
    pub force: bool,

    /// Bypass dependency cryptographic SHA-256 checksum verification
    #[arg(long, visible_alias = "ignore-checksum", alias = "yolo")]
    pub no_verify: bool,
}

impl CompileArgs {
    /// Resolves the effective link mode considering the `--static` shorthand.
    #[must_use]
    pub fn resolved_link_mode(&self) -> solix_compilation::LinkMode {
        if self.r#static || self.link.eq_ignore_ascii_case("static") {
            solix_compilation::LinkMode::Static
        } else {
            solix_compilation::LinkMode::Dynamic
        }
    }
}

/// Subcommands under `solix run`.
#[derive(Debug, Subcommand, Clone)]
pub enum RunSubcommand {
    /// Executes a Solix package (.slxp archive or package directory)
    Package(RunPackageArgs),

    /// Compiles on-the-fly and executes a raw Solix source file (.slx)
    Source(RunSourceArgs),
}

/// Arguments for `solix run package <path>`.
#[derive(Debug, Args, Clone)]
pub struct RunPackageArgs {
    /// Path to a .slxp package archive or project directory containing manifest.yaml
    #[arg(value_name = "PACKAGE")]
    pub target: PathBuf,

    /// Override default entry point symbol (e.g. "Main::main")
    #[arg(short, long)]
    pub entry: Option<String>,

    /// Target build profile (e.g. "release", "debug", "test")
    #[arg(short, long)]
    pub profile: Option<String>,

    /// Shorthand flag for debug build profile
    #[arg(long, hide = true, conflicts_with_all = ["release", "test"])]
    pub debug: bool,

    /// Shorthand flag for release build profile
    #[arg(long, hide = true, conflicts_with_all = ["debug", "test"])]
    pub release: bool,

    /// Shorthand flag for test build profile
    #[arg(long, hide = true, conflicts_with_all = ["debug", "release"])]
    pub test: bool,

    /// Arguments passed to the Solix application entry point
    #[arg(last = true)]
    pub args: Vec<String>,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,

    /// Virtual heap size limit (e.g., "32MB", "64MB") [default: 32MB]
    #[arg(long, default_value = "32MB")]
    pub heap_size: String,

    /// Maximum call stack depth limit [default: 1024]
    #[arg(long, default_value_t = 1024)]
    pub max_stack: usize,

    /// Enable or disable Bacon & Rajan cycle collector [default: "on"]
    #[arg(long, default_value = "on", value_parser = ["on", "off", "true", "false", "1", "0"])]
    pub cycle_collector: String,

    /// Candidate buffer size threshold triggering cycle collection [default: 1024]
    #[arg(long, default_value_t = 1024)]
    pub cycle_threshold: usize,

    /// Execution budget in microseconds before deferring remaining cycle roots
    #[arg(long)]
    pub cycle_budget_us: Option<u64>,

    /// Print execution statistics and memory report upon completion
    #[arg(short, long, alias = "gc-stats")]
    pub verbose: bool,

    /// Execute in safe mode (bounds-checked execution with defensive invariant validation)
    #[arg(long)]
    pub safe: bool,

    /// Print execution telemetry and memory statistics
    #[arg(long, value_name = "FORMAT", num_args = 0..=1, default_missing_value = "text", value_parser = ["text", "json"])]
    pub metrics: Option<String>,

    /// Output execution telemetry formatted as machine-readable JSON
    #[arg(long, hide = true)]
    pub metrics_json: bool,

    /// Bypass dependency cryptographic SHA-256 checksum verification
    #[arg(long, visible_alias = "ignore-checksum", alias = "yolo")]
    pub no_verify: bool,
}

impl RunPackageArgs {
    /// Resolves the effective build profile from explicit `--profile` or shorthand flags.
    #[must_use]
    pub fn resolved_profile(&self) -> Option<String> {
        if let Some(ref profile_name) = self.profile {
            Some(profile_name.clone())
        } else if self.test {
            Some("test".to_string())
        } else if self.debug {
            Some("debug".to_string())
        } else if self.release {
            Some("release".to_string())
        } else {
            None
        }
    }
}

/// Arguments for `solix run source <file.slx>`.
#[derive(Debug, Args, Clone)]
pub struct RunSourceArgs {
    /// Input Solix source files (*.slx)
    #[arg(value_name = "SOURCES", required = true)]
    pub sources: Vec<PathBuf>,

    /// Override default entry point symbol (e.g. "Main::main")
    #[arg(short, long)]
    pub entry: Option<String>,

    /// Target build profile (e.g. "release", "debug", "test")
    #[arg(short, long)]
    pub profile: Option<String>,

    /// Shorthand flag for debug build profile
    #[arg(long, hide = true, conflicts_with_all = ["release", "test"])]
    pub debug: bool,

    /// Shorthand flag for release build profile
    #[arg(long, hide = true, conflicts_with_all = ["debug", "test"])]
    pub release: bool,

    /// Shorthand flag for test build profile
    #[arg(long, hide = true, conflicts_with_all = ["debug", "release"])]
    pub test: bool,

    /// Inlining statement threshold limit for AstInlinerPhase
    #[arg(long, default_value_t = 8)]
    pub inline_threshold: usize,

    /// Library files to load (*.slxb, *.slxp, *.slxarc, *.so, *.dll, *.dylib)
    #[arg(short = 'l', long = "lib")]
    pub libs: Vec<PathBuf>,

    /// Arguments passed to the Solix application entry point
    #[arg(last = true)]
    pub args: Vec<String>,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,

    /// Virtual heap size limit [default: 32MB]
    #[arg(long, default_value = "32MB")]
    pub heap_size: String,

    /// Maximum call stack depth limit [default: 1024]
    #[arg(long, default_value_t = 1024)]
    pub max_stack: usize,

    /// Enable or disable Bacon & Rajan cycle collector [default: "on"]
    #[arg(long, default_value = "on", value_parser = ["on", "off", "true", "false", "1", "0"])]
    pub cycle_collector: String,

    /// Candidate buffer size threshold triggering cycle collection [default: 1024]
    #[arg(long, default_value_t = 1024)]
    pub cycle_threshold: usize,

    /// Execution budget in microseconds before deferring remaining cycle roots
    #[arg(long)]
    pub cycle_budget_us: Option<u64>,

    /// Print execution statistics and memory report upon completion
    #[arg(short, long, alias = "gc-stats")]
    pub verbose: bool,

    /// Execute in safe mode
    #[arg(long)]
    pub safe: bool,

    /// Print execution telemetry
    #[arg(long, value_name = "FORMAT", num_args = 0..=1, default_missing_value = "text", value_parser = ["text", "json"])]
    pub metrics: Option<String>,

    /// Output execution telemetry formatted as machine-readable JSON
    #[arg(long, hide = true)]
    pub metrics_json: bool,

    /// Bypass dependency cryptographic SHA-256 checksum verification
    #[arg(long, visible_alias = "ignore-checksum", alias = "yolo")]
    pub no_verify: bool,
}

impl RunSourceArgs {
    /// Resolves the effective build profile from explicit `--profile` or shorthand flags.
    #[must_use]
    pub fn resolved_profile(&self) -> Option<String> {
        if let Some(ref profile_name) = self.profile {
            Some(profile_name.clone())
        } else if self.test {
            Some("test".to_string())
        } else if self.debug {
            Some("debug".to_string())
        } else if self.release {
            Some("release".to_string())
        } else {
            None
        }
    }
}

/// Arguments for `solix run`.
#[derive(Debug, Args, Clone)]
#[command(args_conflicts_with_subcommands = true)]
pub struct RunArgs {
    /// Subcommand: `package` or `source`
    #[command(subcommand)]
    pub subcommand: Option<RunSubcommand>,

    /// Executable binary containers (*.slxb), archive containers (*.slxp, *.slxarc), and/or source files (*.slx) to execute
    #[arg(value_name = "BINARIES")]
    pub binaries: Vec<PathBuf>,

    /// Library files to load (*.slxb, *.slxp, *.slxarc, *.so, *.dll, *.dylib)
    #[arg(short = 'l', long = "lib")]
    pub libs: Vec<PathBuf>,

    /// Override default entry point symbol (e.g. "Main::main")
    #[arg(short, long)]
    pub entry: Option<String>,

    /// Target build profile (e.g. "release", "debug", "test")
    #[arg(short, long)]
    pub profile: Option<String>,

    /// Shorthand flag for debug build profile
    #[arg(long, hide = true, conflicts_with_all = ["release", "test"])]
    pub debug: bool,

    /// Shorthand flag for release build profile
    #[arg(long, hide = true, conflicts_with_all = ["debug", "test"])]
    pub release: bool,

    /// Shorthand flag for test build profile
    #[arg(long, hide = true, conflicts_with_all = ["debug", "release"])]
    pub test: bool,

    /// Inlining statement threshold limit for AstInlinerPhase
    #[arg(long, default_value_t = 8)]
    pub inline_threshold: usize,

    /// Virtual heap size limit (e.g., "32MB", "64MB") [default: 32MB]
    #[arg(long, default_value = "32MB")]
    pub heap_size: String,

    /// Maximum call stack depth limit [default: 1024]
    #[arg(long, default_value_t = 1024)]
    pub max_stack: usize,

    /// Enable or disable Bacon & Rajan (2001) ARC cycle collector (e.g. "on", "off") [default: "on"]
    #[arg(long, default_value = "on", value_parser = ["on", "off", "true", "false", "1", "0"])]
    pub cycle_collector: String,

    /// Candidate buffer size threshold triggering cycle collection [default: 1024]
    #[arg(long, default_value_t = 1024)]
    pub cycle_threshold: usize,

    /// Execution budget in microseconds before deferring remaining cycle roots
    #[arg(long)]
    pub cycle_budget_us: Option<u64>,

    /// Print execution statistics and memory report upon completion
    #[arg(short, long, alias = "gc-stats")]
    pub verbose: bool,

    /// Execute in safe mode (bounds-checked execution with defensive invariant validation)
    #[arg(long)]
    pub safe: bool,

    /// Print execution telemetry and memory statistics
    #[arg(long, value_name = "FORMAT", num_args = 0..=1, default_missing_value = "text", value_parser = ["text", "json"])]
    pub metrics: Option<String>,

    /// Output execution telemetry formatted as machine-readable JSON
    #[arg(long, hide = true)]
    pub metrics_json: bool,

    /// Arguments passed to the Solix application entry point
    #[arg(last = true)]
    pub args: Vec<String>,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,

    /// Bypass dependency and native library cryptographic SHA-256 checksum verification
    #[arg(long, visible_alias = "ignore-checksum", alias = "yolo")]
    pub no_verify: bool,
}

impl RunArgs {
    /// Resolves the effective build profile from explicit `--profile` or shorthand flags.
    #[must_use]
    pub fn resolved_profile(&self) -> Option<String> {
        if let Some(ref profile_name) = self.profile {
            Some(profile_name.clone())
        } else if self.test {
            Some("test".to_string())
        } else if self.debug {
            Some("debug".to_string())
        } else if self.release {
            Some("release".to_string())
        } else {
            None
        }
    }
}

/// Arguments for `solix info`.
#[derive(Debug, Args, Clone)]
pub struct InfoArgs {
    /// Target binary container file (*.slxb), package container (*.slxp, *.slxarc), or installed package name (<name> or <name>@<version>)
    #[arg(required = true)]
    pub target: PathBuf,

    /// Output file destination for inspection data [default: stdout]
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Output the raw export manifest as formatted JSON
    #[arg(long)]
    pub json: bool,

    /// Print the reconstructed Solix API interface source (.slx)
    #[arg(long)]
    pub api: bool,

    /// Disassemble bytecode instructions
    #[arg(long)]
    pub disasm: bool,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,
}

/// Subcommands under `solix install`.
#[derive(Debug, Subcommand, Clone)]
pub enum InstallSubcommand {
    /// Installs a .slxp package archive into ~/.solix/packages/
    Package(InstallPackageArgs),

    /// Installs a standalone .slxb binary into ~/.solix/bin/
    Binaries(InstallBinariesArgs),
}

/// Arguments for `solix install package <path>`.
#[derive(Debug, Args, Clone)]
pub struct InstallPackageArgs {
    /// Path to a .slxp package archive or project directory
    pub path: PathBuf,

    /// Force overwrite if package is already installed
    #[arg(short, long)]
    pub force: bool,

    /// Target build profile when installing from a project directory: debug, release (default), or test
    #[arg(long)]
    pub profile: Option<String>,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,
}

impl InstallPackageArgs {
    /// Resolves the effective build profile string.
    #[must_use]
    pub fn resolved_profile(&self) -> &str {
        if let Some(ref prof) = self.profile {
            prof.as_str()
        } else {
            "release"
        }
    }
}

/// Arguments for `solix install binaries <path>`.
#[derive(Debug, Args, Clone)]
pub struct InstallBinariesArgs {
    /// Path to a .slxb binary container
    pub path: PathBuf,

    /// Force overwrite if binary is already installed
    #[arg(short, long)]
    pub force: bool,

    /// Target build profile override
    #[arg(long)]
    pub profile: Option<String>,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,
}

impl InstallBinariesArgs {
    /// Resolves the effective build profile string.
    #[must_use]
    pub fn resolved_profile(&self) -> &str {
        if let Some(ref prof) = self.profile {
            prof.as_str()
        } else {
            "release"
        }
    }
}

/// Arguments for `solix install`.
#[derive(Debug, Args, Clone)]
#[command(args_conflicts_with_subcommands = true)]
pub struct InstallArgs {
    /// Optional subcommand: `package` or `binaries`
    #[command(subcommand)]
    pub subcommand: Option<InstallSubcommand>,

    /// Path to a .slxb binary, .slxp archive container, or project directory to install into the package store
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Install or re-extract all bundled first-party default libraries from assets/packages/
    #[arg(long)]
    pub bundled: bool,

    /// Install the official Solix Visual Studio Code / VSCodium / Cursor extension
    #[arg(long, conflicts_with = "path")]
    pub vscode: bool,

    /// Force overwrite if package is already installed
    #[arg(short, long)]
    pub force: bool,

    /// Target build profile when installing from a project directory: debug, release (default), or test
    #[arg(long)]
    pub profile: Option<String>,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,
}

impl InstallArgs {
    /// Resolves the effective build profile string.
    #[must_use]
    pub fn resolved_profile(&self) -> &str {
        if let Some(ref prof) = self.profile {
            prof.as_str()
        } else {
            "release"
        }
    }
}

/// Subcommands under `solix uninstall`.
#[derive(Debug, Subcommand, Clone)]
pub enum UninstallSubcommand {
    /// Uninstalls a package from the local package store with exact matching
    Package(UninstallPackageArgs),

    /// Uninstalls a binary from ~/.solix/bin/ with exact matching
    Binaries(UninstallBinariesArgs),
}

/// Arguments for `solix uninstall package <name>`.
#[derive(Debug, Args, Clone)]
pub struct UninstallPackageArgs {
    /// Package name (or package@version)
    pub spec: String,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,
}

/// Arguments for `solix uninstall binaries <name>`.
#[derive(Debug, Args, Clone)]
pub struct UninstallBinariesArgs {
    /// Binary name (or binary@version)
    pub spec: String,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,
}

/// Arguments for `solix uninstall`.
#[derive(Debug, Args, Clone)]
#[command(args_conflicts_with_subcommands = true)]
pub struct UninstallArgs {
    /// Subcommand: `package` or `binaries`
    #[command(subcommand)]
    pub subcommand: Option<UninstallSubcommand>,

    /// Package name (to remove all versions) or package@version (e.g. raylib@1.0.0) [fallback when not using subcommands]
    #[arg(value_name = "SPEC")]
    pub spec: Option<String>,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,
}

/// Subcommands under `solix export`.
#[derive(Debug, Subcommand, Clone)]
pub enum ExportSubcommand {
    /// Exports a packaged .slxp bundle from a Solix project
    Package(ExportPackageArgs),

    /// Exports a standalone compiled .slxb binary from a Solix project
    Binaries(ExportBinariesArgs),
}

/// Arguments for `solix export package [path]`.
#[derive(Debug, Args, Clone)]
pub struct ExportPackageArgs {
    /// Project root directory containing manifest.yaml [default: current directory]
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Destination path for the exported .slxp package [default: <name>.slxp in project or cwd]
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Target build profile (e.g. "release", "debug")
    #[arg(short, long)]
    pub profile: Option<String>,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,
}

/// Arguments for `solix export binaries [path]`.
#[derive(Debug, Args, Clone)]
pub struct ExportBinariesArgs {
    /// Project root directory containing manifest.yaml [default: current directory]
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Destination path for the exported .slxb binary [default: <name>.slxb]
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Target build profile (e.g. "release", "debug")
    #[arg(short, long)]
    pub profile: Option<String>,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,
}

/// Arguments for `solix export`.
#[derive(Debug, Args, Clone)]
pub struct ExportArgs {
    #[command(subcommand)]
    pub subcommand: ExportSubcommand,
}

/// Arguments for `solix list`.
#[derive(Debug, Args, Clone)]
pub struct ListArgs {
    /// Filter packages matching substring in name or description
    #[arg(short, long)]
    pub filter: Option<String>,

    /// Filter packages by build profile (debug, release, test)
    #[arg(long)]
    pub profile: Option<String>,

    /// Filter packages by container kind (binaries, package)
    #[arg(short, long)]
    pub kind: Option<String>,

    /// Sort key for package listing: name (default), version, date, size, profile
    #[arg(long, default_value = "name")]
    pub sort: String,

    /// Reverse sort order
    #[arg(short, long)]
    pub reverse: bool,

    /// Output package listing as machine-readable JSON
    #[arg(long)]
    pub json: bool,

    /// Output file destination for package listing [default: stdout]
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,
}

/// Subcommands under `solix new`.
#[derive(Debug, Subcommand, Clone)]
pub enum NewSubcommand {
    /// Scaffolds a declarative Solix package project
    Package(NewPackageArgs),

    /// Scaffolds a dual-mode native C, C++, or Rust subproject and optionally registers it in manifest.yaml
    Native(NewNativeArgs),
}

/// Arguments for `solix new package <path>`.
#[derive(Debug, Args, Clone)]
pub struct NewPackageArgs {
    /// Target directory path for the new package
    #[arg(value_name = "PATH")]
    pub path: PathBuf,

    /// Package name [default: directory name]
    #[arg(long)]
    pub name: Option<String>,

    /// Initial package version [default: 0.1.0]
    #[arg(long, default_value = "0.1.0")]
    pub version: String,

    /// Package description
    #[arg(long, default_value = "A Solix package")]
    pub description: String,

    /// License identifier
    #[arg(long, default_value = "Apache-2.0")]
    pub license: String,

    /// Author name (can be repeated)
    #[arg(long = "author")]
    pub authors: Vec<String>,

    /// Main entrypoint symbol [default: Main::main]
    #[arg(long, default_value = "Main::main")]
    pub main: String,

    /// Test entrypoint symbol [default: TestRunner::test]
    #[arg(long, default_value = "TestRunner::test")]
    pub test: String,

    /// Generate a library package instead of an application
    #[arg(long)]
    pub lib: bool,

    /// Scaffold a native bridge (`rust` [default], `c`, or `cpp`)
    #[arg(
        long,
        num_args = 0..=1,
        default_missing_value = "rust",
        value_name = "LANG",
        conflicts_with = "no_native"
    )]
    pub native: Option<String>,

    /// Do not create a native bridge subproject
    #[arg(long, conflicts_with = "native")]
    pub no_native: bool,

    /// Do not automatically add standard library dependency ('solix')
    #[arg(long = "no-std", alias = "no-default-deps")]
    pub no_std: bool,

    /// Add dependency in pkg@ver format (can be repeated)
    #[arg(short = 'd', long = "deps")]
    pub deps: Vec<String>,

    /// Overwrite existing manifest.yaml if present
    #[arg(long)]
    pub force: bool,
}

/// Arguments for `solix new native <name>`.
#[derive(Debug, Args, Clone)]
pub struct NewNativeArgs {
    /// Canonical native module identifier (used for folder and default Solix class prefix)
    pub name: String,

    /// Path to an existing Solix project root containing manifest.yaml [default: "." when --output is omitted]
    #[arg(short = 'p', long)]
    pub project: Option<PathBuf>,

    /// Explicit Cargo/CMake package identifier [default: snake_case of <NAME>]
    #[arg(long = "name")]
    pub package_name: Option<String>,

    /// Initial semantic version [default: 0.1.0]
    #[arg(long, default_value = "0.1.0")]
    pub version: String,

    /// Native backend language: rust (Cargo), c (CMake C11), or cpp (CMake C++17)
    #[arg(
        long = "lang",
        alias = "backend",
        default_value = "rust",
        value_name = "LANG"
    )]
    pub lang: String,

    /// Output directory for the generated native subproject [default: <project>/natives/<package_name>]
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Target OS/architecture platforms recorded in manifest.yaml (comma-separated or repeatable)
    #[arg(long = "platforms", value_delimiter = ',')]
    pub platforms: Vec<String>,

    /// Manifest profile(s) to register the native subproject under [default: release & debug]
    #[arg(long = "profile", value_delimiter = ',')]
    pub profiles: Vec<String>,

    /// Human-readable description for Cargo.toml or CMakeLists.txt
    #[arg(long, default_value = "Dual-mode native library for Solix")]
    pub description: String,

    /// SPDX license identifier [default: Apache-2.0]
    #[arg(long, default_value = "Apache-2.0")]
    pub license: String,

    /// Author name (can be repeated)
    #[arg(long = "author")]
    pub authors: Vec<String>,

    /// Scaffold files without registering into manifest.yaml
    #[arg(long)]
    pub no_register: bool,

    /// Overwrite existing files inside the target native directory
    #[arg(long)]
    pub force: bool,
}

/// Arguments for `solix new`.
#[derive(Debug, Args, Clone)]
#[command(args_conflicts_with_subcommands = true)]
pub struct NewArgs {
    /// Optional nested scaffolding subcommand (e.g. `solix new package <path>` or `solix new native <name>`)
    #[command(subcommand)]
    pub subcommand: Option<NewSubcommand>,

    /// Target directory path for the new project [when not using 'new package' or 'new native']
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Project name [default: directory name]
    #[arg(long)]
    pub name: Option<String>,

    /// Initial project version [default: 0.1.0]
    #[arg(long, default_value = "0.1.0")]
    pub version: String,

    /// Project description
    #[arg(long, default_value = "A Solix project")]
    pub description: String,

    /// License identifier
    #[arg(long, default_value = "Apache-2.0")]
    pub license: String,

    /// Author name (can be repeated)
    #[arg(long = "author")]
    pub authors: Vec<String>,

    /// Main entrypoint symbol [default: Main::main]
    #[arg(long, default_value = "Main::main")]
    pub main: String,

    /// Test entrypoint symbol [default: TestRunner::test]
    #[arg(long, default_value = "TestRunner::test")]
    pub test: String,

    /// Generate a library project instead of an application
    #[arg(long)]
    pub lib: bool,

    /// Scaffold a native bridge (`rust` [default], `c`, or `cpp`)
    #[arg(
        long,
        num_args = 0..=1,
        default_missing_value = "rust",
        value_name = "LANG",
        conflicts_with = "no_native"
    )]
    pub native: Option<String>,

    /// Do not create a native bridge subproject
    #[arg(long, conflicts_with = "native")]
    pub no_native: bool,

    /// Do not automatically add standard library dependency ('solix')
    #[arg(long = "no-std", alias = "no-default-deps")]
    pub no_std: bool,

    /// Add dependency in pkg@ver format (can be repeated)
    #[arg(short = 'd', long = "deps")]
    pub deps: Vec<String>,

    /// Overwrite existing manifest.yaml if present
    #[arg(long)]
    pub force: bool,
}

/// Arguments for `solix clean`.
#[derive(Debug, Args, Clone)]
pub struct CleanArgs {
    /// Project root directory containing manifest.yaml (or manifest.toml) [default: current directory]
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Clean only intermediate build caches (build/.cache/) keeping final distribution archives
    #[arg(long)]
    pub cache_only: bool,

    /// Preview files and reclaimed size without deleting anything
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for `solix check`.
#[derive(Debug, Args, Clone)]
pub struct CheckArgs {
    /// Project root directory containing manifest.yaml [default: current directory]
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Target build profile (e.g. "release", "debug", "test") [default: "debug"]
    #[arg(short, long, default_value = "debug")]
    pub profile: String,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,

    /// Bypass dependency cryptographic SHA-256 checksum verification
    #[arg(long, visible_alias = "ignore-checksum", alias = "yolo")]
    pub no_verify: bool,
}

/// Arguments for `solix test`.
#[derive(Debug, Args, Clone)]
pub struct TestArgs {
    /// Project root directory containing manifest.yaml [default: current directory]
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Target build profile [default: "test"]
    #[arg(short, long, default_value = "test")]
    pub profile: String,

    /// Optional substring filter for test function names
    #[arg(long)]
    pub filter: Option<String>,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,

    /// Bypass dependency cryptographic SHA-256 checksum verification
    #[arg(long, visible_alias = "ignore-checksum", alias = "yolo")]
    pub no_verify: bool,
}

/// Arguments for `solix init`.
#[derive(Debug, Args, Clone)]
pub struct InitArgs {
    /// Target directory to initialize [default: current directory]
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Project name [default: directory name]
    #[arg(long)]
    pub name: Option<String>,

    /// Overwrite existing manifest.yaml if present
    #[arg(long)]
    pub force: bool,
}

/// Arguments for `solix add`.
#[derive(Debug, Args, Clone)]
pub struct AddArgs {
    /// Package specification (e.g. "raylib" or "raylib@5.5.0")
    pub package: String,

    /// Project root directory containing manifest.yaml [default: current directory]
    #[arg(long, default_value = ".")]
    pub path: PathBuf,

    /// Dependency build profile to link against [default: "release"]
    #[arg(long = "dep-profile", default_value = "release")]
    pub dep_profile: String,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,
}
