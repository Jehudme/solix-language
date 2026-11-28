//! Solix compiler and virtual machine CLI binary executable.

#![forbid(unsafe_code)]

use clap::Parser;
use solix_toolchain::RunArgs;
use solix_toolchain::cli::Cli;
use std::path::PathBuf;

/// Fast zero-clap parser for direct `solix run [OPTIONS] <BINARIES>... [-- <ARGS>...]` invocations.
/// Falls back to full `clap::Parser` on help, version, unknown flags, or parse errors.
fn try_parse_fast_run_args() -> Option<RunArgs> {
    let mut os_args = std::env::args_os();
    let _bin_name = os_args.next()?;
    let subcmd = os_args.next()?;
    if subcmd.to_str()? != "run" {
        return None;
    }

    let mut run_args = RunArgs {
        subcommand: None,
        binaries: Vec::new(),
        libs: Vec::new(),
        entry: None,
        profile: None,
        debug: false,
        release: false,
        test: false,
        inline_threshold: 8,
        heap_size: "32MB".to_string(),
        max_stack: 1024,
        cycle_collector: "on".to_string(),
        cycle_threshold: 1024,
        cycle_budget_us: None,
        verbose: false,
        safe: false,
        jit: "tiered".to_string(),
        metrics: None,
        metrics_json: false,
        args: Vec::new(),
        pkg_dir: None,
        no_verify: false,
    };

    let mut parsing_trailing_args = false;
    while let Some(raw_os_arg) = os_args.next() {
        let arg_str = raw_os_arg.to_str()?;
        if parsing_trailing_args {
            run_args.args.push(arg_str.to_string());
            continue;
        }
        if arg_str == "--" {
            parsing_trailing_args = true;
            continue;
        }
        if !arg_str.starts_with('-') || arg_str == "-" {
            if arg_str == "package" || arg_str == "source" {
                return None;
            }
            run_args.binaries.push(PathBuf::from(arg_str));
            continue;
        }

        if let Some((flag_name, flag_val)) = arg_str.split_once('=') {
            match flag_name {
                "--lib" | "-l" => run_args.libs.push(PathBuf::from(flag_val)),
                "--entry" | "-e" => run_args.entry = Some(flag_val.to_string()),
                "--profile" | "-p" => run_args.profile = Some(flag_val.to_string()),
                "--inline-threshold" => {
                    run_args.inline_threshold = flag_val.parse().ok()?;
                }
                "--heap-size" => run_args.heap_size = flag_val.to_string(),
                "--max-stack" => {
                    run_args.max_stack = flag_val.parse().ok()?;
                }
                "--cycle-collector" => match flag_val {
                    "on" | "off" | "true" | "false" | "1" | "0" => {
                        run_args.cycle_collector = flag_val.to_string();
                    }
                    _ => return None,
                },
                "--cycle-threshold" => {
                    run_args.cycle_threshold = flag_val.parse().ok()?;
                }
                "--cycle-budget-us" => {
                    run_args.cycle_budget_us = Some(flag_val.parse().ok()?);
                }
                "--jit" => match flag_val {
                    "off" | "tiered" | "eager" => run_args.jit = flag_val.to_string(),
                    _ => return None,
                },
                "--metrics" => match flag_val {
                    "text" | "json" => run_args.metrics = Some(flag_val.to_string()),
                    _ => return None,
                },
                "--pkg-dir" => run_args.pkg_dir = Some(PathBuf::from(flag_val)),
                _ => return None,
            }
            continue;
        }

        match arg_str {
            "--safe" => run_args.safe = true,
            "--verbose" | "-v" | "--gc-stats" => run_args.verbose = true,
            "--metrics-json" => run_args.metrics_json = true,
            "--metrics" => {
                run_args.metrics = Some("text".to_string());
            }
            "--no-verify" | "--ignore-checksum" | "--yolo" => run_args.no_verify = true,
            "--test" => run_args.test = true,
            "--debug" => run_args.debug = true,
            "--release" => run_args.release = true,
            "--lib" | "-l" => {
                let next_val = os_args.next()?;
                run_args.libs.push(PathBuf::from(next_val));
            }
            "--entry" | "-e" => {
                let next_val = os_args.next()?.into_string().ok()?;
                run_args.entry = Some(next_val);
            }
            "--profile" | "-p" => {
                let next_val = os_args.next()?.into_string().ok()?;
                run_args.profile = Some(next_val);
            }
            "--inline-threshold" => {
                let next_val = os_args.next()?;
                run_args.inline_threshold = next_val.to_str()?.parse().ok()?;
            }
            "--heap-size" => {
                let next_val = os_args.next()?.into_string().ok()?;
                run_args.heap_size = next_val;
            }
            "--max-stack" => {
                let next_val = os_args.next()?;
                run_args.max_stack = next_val.to_str()?.parse().ok()?;
            }
            "--jit" => {
                let next_val = os_args.next()?.into_string().ok()?;
                match next_val.as_str() {
                    "off" | "tiered" | "eager" => run_args.jit = next_val,
                    _ => return None,
                }
            }
            "--cycle-collector" => {
                let next_val = os_args.next()?.into_string().ok()?;
                match next_val.as_str() {
                    "on" | "off" | "true" | "false" | "1" | "0" => {
                        run_args.cycle_collector = next_val;
                    }
                    _ => return None,
                }
            }
            "--cycle-threshold" => {
                let next_val = os_args.next()?;
                run_args.cycle_threshold = next_val.to_str()?.parse().ok()?;
            }
            "--cycle-budget-us" => {
                let next_val = os_args.next()?;
                run_args.cycle_budget_us = Some(next_val.to_str()?.parse().ok()?);
            }
            "--pkg-dir" => {
                let next_val = os_args.next()?;
                run_args.pkg_dir = Some(PathBuf::from(next_val));
            }
            _ => return None,
        }
    }

    if run_args.binaries.is_empty() {
        return None;
    }
    Some(run_args)
}

fn main() {
    let run_result = if let Some(fast_run_args) = try_parse_fast_run_args() {
        solix_toolchain::execute_run(fast_run_args)
    } else {
        let cli = Cli::parse();
        solix_toolchain::run(cli)
    };
    match run_result {
        Ok(code) => {
            std::process::exit(code);
        }
        Err(err) => {
            eprintln!("Error: {err:?}");
            std::process::exit(1);
        }
    }
}
