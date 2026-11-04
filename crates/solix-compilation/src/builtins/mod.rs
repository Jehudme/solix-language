//! Standard built-in language library sources.

#![forbid(unsafe_code)]

/// Built-in I/O definitions for `package blt.io`.
pub const BLT_IO_SOURCE: &str = include_str!("io.slx");

/// Built-in virtual file name for `blt.io`.
pub const BLT_IO_FILE_NAME: &str = "blt@0.1.0/io.slx";
