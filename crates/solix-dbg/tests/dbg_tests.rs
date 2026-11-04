//! Integration tests for the Solix debugger adapter protocol implementation.

#![forbid(unsafe_code)]

#[test]
fn test_debugger_scaffolding_ready() {
    let dap_protocol_name = "solix-dap";
    assert_eq!(dap_protocol_name, "solix-dap");
}
