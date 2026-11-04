//! Master entry for all end-to-end compilation & execution conformance test suites.

#![forbid(unsafe_code)]

#[path = "compilation/modules/mod.rs"]
mod modules;

#[path = "compilation/declarations/mod.rs"]
mod declarations;

#[path = "compilation/control_flow/mod.rs"]
mod control_flow;

#[path = "compilation/expressions/mod.rs"]
mod expressions;

#[path = "compilation/integration/mod.rs"]
mod integration;

#[path = "compilation/pipeline/mod.rs"]
mod pipeline;
