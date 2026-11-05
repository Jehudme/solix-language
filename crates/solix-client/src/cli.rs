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
pub enum Commands {
    /// Compiles Solix source files into an executable binary container (.slxb)
    Compile(CompileArgs),

    /// Builds a Solix project into a .slxarc archive container
    Build(BuildArgs),

    /// Executes compiled Solix binary containers in the virtual machine
    Run(RunArgs),

    /// Inspects metadata, export manifests, and bytecode of an SLXB binary container or .slxarc archive
    Info(InfoArgs),

    /// Installs a .slxb or .slxarc package into the local package store
    Install(InstallArgs),

    /// Uninstalls a package or package version from the local package store
    Uninstall(UninstallArgs),

    /// Lists all installed packages in the local package store
    List(ListArgs),
}

/// Arguments for `solix build`.
#[derive(Debug, Args, Clone)]
pub struct BuildArgs {
    /// Project root directory containing manifest.toml and scripts/build.py [default: current directory]
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Target build profile: debug, release, or test
    #[arg(long, default_value = "debug")]
    pub profile: String,

    /// Shorthand flag for debug build profile
    #[arg(long, conflicts_with_all = ["release", "test"])]
    pub debug: bool,

    /// Shorthand flag for release build profile
    #[arg(long, conflicts_with_all = ["debug", "test"])]
    pub release: bool,

    /// Shorthand flag for test build profile
    #[arg(long, conflicts_with_all = ["debug", "release"])]
    pub test: bool,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,
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
        } else {
            &self.profile
        }
    }
}

/// Arguments for `solix compile`.
#[derive(Debug, Args, Clone)]
pub struct CompileArgs {
    /// Input Solix source files (*.slx)
    #[arg(required = true)]
    pub sources: Vec<PathBuf>,

    /// Output compiled binary file path [default: <name>.slxb]
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Package dependency requirements formatted as <name>@<version> (e.g. raylib@1.0.0)
    #[arg(short = 'd', long = "deps", visible_alias = "dep")]
    pub deps: Vec<String>,

    /// Paths to library binary containers (*.slxb) or archives (*.slxarc) whose API interfaces should be extracted
    #[arg(short = 'l', long = "lib")]
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

    /// Do not automatically include built-in standard library definitions (blt.io)
    #[arg(long)]
    pub no_builtins: bool,

    /// Perform syntax and semantic checks without emitting bytecode
    #[arg(long)]
    pub check: bool,

    /// Print wall-clock duration for each compilation phase
    #[arg(long)]
    pub time: bool,

    /// Print compiler throughput metrics and phase duration breakdown
    #[arg(long)]
    pub metrics: bool,

    /// Output compiler metrics formatted as machine-readable JSON
    #[arg(long)]
    pub metrics_json: bool,

    /// Inlining statement threshold limit for AstInlinerPhase [default: 8]
    #[arg(long, default_value_t = 8)]
    pub inline_threshold: usize,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,
}

/// Arguments for `solix run`.
#[derive(Debug, Args, Clone)]
pub struct RunArgs {
    /// Executable binary containers (*.slxb), archive containers (*.slxarc), and/or source files (*.slx) to execute
    #[arg(required = true)]
    pub binaries: Vec<PathBuf>,

    /// Library files to load (*.slxb, *.slxarc, *.so, *.dll, *.dylib)
    #[arg(short = 'l', long = "lib")]
    pub libs: Vec<PathBuf>,

    /// Override default entry point symbol (e.g. "Main::main")
    #[arg(short, long)]
    pub entry: Option<String>,

    /// Execute test entrypoint defined in archive manifest ([entrypoints].test)
    #[arg(long)]
    pub test: bool,

    /// Do not automatically include built-in standard library definitions (blt.io)
    #[arg(long)]
    pub no_builtins: bool,

    /// Inlining statement threshold limit for AstInlinerPhase [default: 8]
    #[arg(long, default_value_t = 8)]
    pub inline_threshold: usize,

    /// Virtual heap size limit (e.g., "32MB", "64MB") [default: 32MB]
    #[arg(long, default_value = "32MB")]
    pub heap_size: String,

    /// Maximum call stack depth limit [default: 1024]
    #[arg(long, default_value_t = 1024)]
    pub max_stack: usize,

    /// Print execution statistics and memory report upon completion
    #[arg(short, long)]
    pub verbose: bool,

    /// Execute in safe mode (bounds-checked execution with defensive invariant validation)
    #[arg(long)]
    pub safe: bool,

    /// Print execution telemetry and memory statistics
    #[arg(long)]
    pub metrics: bool,

    /// Output execution telemetry formatted as machine-readable JSON
    #[arg(long)]
    pub metrics_json: bool,

    /// Arguments passed to the Solix application entry point
    #[arg(last = true)]
    pub args: Vec<String>,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,
}

/// Arguments for `solix info`.
#[derive(Debug, Args, Clone)]
pub struct InfoArgs {
    /// Target binary container file (*.slxb)
    #[arg(required = true)]
    pub binary: PathBuf,

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
}

/// Arguments for `solix install`.
#[derive(Debug, Args, Clone)]
pub struct InstallArgs {
    /// Path to a .slxb binary or .slxarc archive container to install into the package store
    #[arg(required = true)]
    pub path: PathBuf,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,
}

/// Arguments for `solix uninstall`.
#[derive(Debug, Args, Clone)]
pub struct UninstallArgs {
    /// Package name (to remove all versions) or package@version (e.g. raylib@1.0.0)
    #[arg(required = true)]
    pub spec: String,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,
}

/// Arguments for `solix list`.
#[derive(Debug, Args, Clone)]
pub struct ListArgs {
    /// Output package listing as machine-readable JSON
    #[arg(long)]
    pub json: bool,

    /// Optional package store root directory override
    #[arg(long)]
    pub pkg_dir: Option<PathBuf>,
}
