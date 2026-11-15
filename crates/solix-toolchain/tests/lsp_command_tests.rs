//! End-to-end stdio transport integration tests for `solix lsp`.

#![forbid(unsafe_code)]

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};

#[test]
fn test_stdio_lsp_server_handshake() {
    let binary_path = env!("CARGO_BIN_EXE_solix");

    let mut child = Command::new(binary_path)
        .arg("lsp")
        .arg("--stdio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn solix lsp process");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout);

    // Prepare initialize request
    let request_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "capabilities": {},
            "rootUri": "file:///test_workspace"
        }
    });

    let request_str = request_payload.to_string();
    let header = format!("Content-Length: {}\r\n\r\n", request_str.len());

    stdin.write_all(header.as_bytes()).unwrap();
    stdin.write_all(request_str.as_bytes()).unwrap();
    stdin.flush().unwrap();

    // Read Content-Length header
    let mut content_length: usize = 0;
    let mut line = String::new();

    loop {
        line.clear();
        reader.read_line(&mut line).unwrap();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(stripped) = trimmed.strip_prefix("Content-Length:") {
            content_length = stripped.trim().parse().unwrap();
        }
    }

    assert!(
        content_length > 0,
        "Expected non-zero Content-Length from solix-lsp"
    );

    // Read JSON payload
    let mut buffer = vec![0u8; content_length];
    reader.read_exact(&mut buffer).unwrap();
    let response: serde_json::Value = serde_json::from_slice(&buffer).unwrap();

    assert_eq!(response["id"], 1);
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(
        response["result"]["capabilities"]["definitionProvider"].is_boolean()
            || response["result"]["capabilities"]["definitionProvider"].is_object()
    );

    // Send shutdown
    let shutdown_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "shutdown",
        "params": null
    });
    let shutdown_str = shutdown_payload.to_string();
    let shutdown_header = format!("Content-Length: {}\r\n\r\n", shutdown_str.len());
    let _ = stdin.write_all(shutdown_header.as_bytes());
    let _ = stdin.write_all(shutdown_str.as_bytes());
    let _ = stdin.flush();

    // Kill or wait
    let _ = child.kill();
    let _ = child.wait();
}
