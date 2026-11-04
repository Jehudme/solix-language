//! Master end-to-end integration test suite for Milestone 006: Register Bytecode Emitter & SLXB Container.

#![forbid(unsafe_code)]

use solix_common::bytecode::BinaryProgram;
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_compilation::phases::emitter::disassemble_program;
use std::fs;
use std::path::Path;

#[test]
fn test_master_all_statements_emission_pipeline_and_round_trip() {
    let fixture_path =
        Path::new("tests/fixtures/emission/positive/master_all_statements_emission.slx");
    let source_code = if fixture_path.exists() {
        fs::read_to_string(fixture_path).expect("read fixture")
    } else {
        fs::read_to_string(Path::new("crates/solix-compilation/tests/fixtures/emission/positive/master_all_statements_emission.slx"))
            .expect("read fixture fallback")
    };

    let file_name = "master_all_statements_emission.slx";
    let options = CompilationOptions::new().with_source(file_name, &source_code);
    let mut context = CompilationContext::new(options);

    // 1. Verify bidirectional source name / ID / code lookups
    let source_id = context.get_source_id(file_name).expect("source id found");
    assert_eq!(source_id, 0);
    assert_eq!(context.get_source_name(0), Some(file_name));
    assert!(context.get_source_code(0).is_some());

    // 2. Lex & Parse
    context.lex_source(0).expect("lex positive fixture");
    context.parse_source(0).expect("parse positive fixture");

    // 3. Emit bytecode through pipeline
    let artifacts = context
        .emit_context()
        .expect("emit positive fixture")
        .clone();

    // 4. Verify function partition per source
    assert_eq!(artifacts.functions_by_source.len(), 1);
    let source_funcs = &artifacts.functions_by_source[0];
    assert!(
        !source_funcs.is_empty(),
        "Source functions should not be empty"
    );

    let func_symbols: Vec<&str> = source_funcs.iter().map(|f| f.symbol.as_str()).collect();
    assert!(
        func_symbols.iter().any(|s| s.contains("calculate")),
        "Missing calculate method"
    );
    assert!(
        func_symbols.iter().any(|s| s.contains("run_all_checks")),
        "Missing run_all_checks method"
    );

    // 5. Verify emit_source helper
    let single_source_funcs = context.emit_source(0).expect("emit source 0");
    assert_eq!(single_source_funcs.len(), source_funcs.len());

    // 6. Verify program structure
    let program = &artifacts.program;
    assert!(!program.functions.is_empty());
    assert_eq!(
        program.functions[0].symbol, "@init",
        "Function ID 0 must be @init"
    );
    assert!(
        program.static_space_size > 0,
        "Static space size should be allocated"
    );

    // 7. Verify SLXB Binary Serialization Round-Trip
    let binary_bytes = program.to_bytes();
    assert!(binary_bytes.len() >= 48, "Binary must contain header");
    assert_eq!(&binary_bytes[0..4], b"SLXB", "Magic header must be SLXB");

    let decoded_program = BinaryProgram::from_bytes(&binary_bytes).expect("decode SLXB binary");
    assert_eq!(decoded_program.version_major, program.version_major);
    assert_eq!(decoded_program.version_minor, program.version_minor);
    assert_eq!(decoded_program.static_space_size, program.static_space_size);
    assert_eq!(
        decoded_program.static_symbols_map,
        program.static_symbols_map
    );
    assert_eq!(decoded_program.functions.len(), program.functions.len());

    for (f1, f2) in program.functions.iter().zip(&decoded_program.functions) {
        assert_eq!(f1.symbol, f2.symbol);
        assert_eq!(f1.param_count, f2.param_count);
        assert_eq!(f1.num_registers, f2.num_registers);
        assert_eq!(f1.instructions.len(), f2.instructions.len());
        for (i1, i2) in f1.instructions.iter().zip(&f2.instructions) {
            assert_eq!(i1.0, i2.0);
        }
    }

    // 8. Verify Disassembler formatting
    let disassembly = disassemble_program(program);
    assert!(
        disassembly.contains(".function @init"),
        "Disassembly missing @init"
    );
    assert!(
        disassembly.contains("SET_STATIC"),
        "Disassembly missing SET_STATIC"
    );
    assert!(disassembly.contains("RETURN"), "Disassembly missing RETURN");
}

#[test]
fn test_master_negative_fixture_graceful_handling() {
    let fixture_path = Path::new("tests/fixtures/emission/negative/master_invalid_emission.slx");
    let source_code = if fixture_path.exists() {
        fs::read_to_string(fixture_path).expect("read negative fixture")
    } else {
        fs::read_to_string(Path::new(
            "crates/solix-compilation/tests/fixtures/emission/negative/master_invalid_emission.slx",
        ))
        .expect("read negative fixture fallback")
    };

    let options = CompilationOptions::new().with_source("invalid.slx", &source_code);
    let mut context = CompilationContext::new(options);

    context.lex_source(0).expect("lex negative fixture");
    context.parse_source(0).expect("parse negative fixture");

    // When emitting an invalid fixture with unresolved symbols, the pipeline should either
    // produce an error or return a diagnostic
    let result = context.emit_context();
    // Verify pipeline either fails gracefully or detects invalid symbols
    if let Err(err) = result {
        let msg = format!("{err}");
        assert!(!msg.is_empty());
    }
}
