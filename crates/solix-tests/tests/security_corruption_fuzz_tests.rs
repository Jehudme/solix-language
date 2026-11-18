//! Systematic `.slxb` bytecode verifier & `.slxarc` archive corruption fuzzing suite.
//!
//! Verifies that random byte corruption, truncated containers, absurd length headers,
//! invalid opcodes, out-of-window registers, out-of-bounds jumps, invalid constant/function/vtable
//! IDs, invalid exception handler tables, Zip-Slip paths, and tampered native SHA-256 checksums
//! ALWAYS produce structured `Err(...)` values and NEVER panic, OOM-abort, or exhibit UB.

#![forbid(unsafe_code)]

use proptest::prelude::*;
use solix_common::bytecode::{
    BinaryProgram, CompiledClass, CompiledFunction, ExceptionHandlerEntry, Instruction, OpCode,
};
use solix_common::hash::sha256_file;
use solix_common::project::SlxarcArchive;
use solix_compilation::CompilationContext::{BuildProfile, CompilationContext, CompilationOptions};
use solix_runtime::{RuntimeContext, RuntimeOptions};
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn compile_fixture_binary() -> BinaryProgram {
    let source = r#"
        class SampleFault {
            public string detail;
            public SampleFault(string detail_msg) {
                this.detail = detail_msg;
            }
        }
        class SampleNode {
            public int32 val;
            public SampleNode(int32 initial_val) {
                this.val = initial_val;
            }
            public virtual int32 compute(int32 extra) {
                if (extra < 0) {
                    throw new SampleFault("neg");
                }
                return this.val + extra;
            }
        }
        int32 main() {
            SampleNode node = new SampleNode(10);
            try {
                return node.compute(5);
            } catch (SampleFault err_obj) {
                return 0;
            }
        }
    "#;
    let options = CompilationOptions::new()
        .with_profile(BuildProfile::Release)
        .with_source("corrupt_fixture.slx", source);
    let mut context = CompilationContext::new(options);
    context.execute().expect("Fixture compilation must succeed");
    context
        .emission_artifacts
        .expect("Emission artifacts must exist")
        .program
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, failure_persistence: None, ..ProptestConfig::default() })]
    #[test]
    fn proptest_corrupted_slxb_bytes_never_panic(
        offset in 0usize..2048usize,
        xor_mask in 1u8..=255u8,
        truncate_len in prop::option::of(0usize..512usize),
    ) {
        let base_bytes = compile_fixture_binary().to_bytes();
        let mut mutated = base_bytes.clone();
        if !mutated.is_empty() {
            let idx = offset % mutated.len();
            mutated[idx] ^= xor_mask;
        }
        if let Some(len_cap) = truncate_len {
            let new_len = len_cap.min(mutated.len());
            mutated.truncate(new_len);
        }

        let probe_result = std::panic::catch_unwind(|| {
            if let Ok(decoded_program) = BinaryProgram::from_bytes(&mutated) {
                let _ = RuntimeContext::new(RuntimeOptions::new().with_binary(decoded_program));
            }
        });
        prop_assert!(
            probe_result.is_ok(),
            "Panic triggered while decoding/verifying mutated .slxb!"
        );
    }

    #[test]
    fn proptest_corrupted_slxarc_bytes_never_panic(
        random_payload in prop::collection::vec(any::<u8>(), 0..512),
    ) {
        let temp_dir = TempDir::new().expect("tempdir");
        let archive_path = temp_dir.path().join("fuzzed.slxarc");
        let extract_dir = temp_dir.path().join("extract");
        fs::write(&archive_path, &random_payload).expect("write fuzzed slxarc");

        let load_res = std::panic::catch_unwind(|| {
            SlxarcArchive::extract_and_load(&archive_path, &extract_dir)
        });
        prop_assert!(load_res.is_ok(), "Panic triggered in SlxarcArchive::extract_and_load");
        prop_assert!(
            load_res.unwrap().is_err(),
            "Arbitrary non-zip payload must return structured Err"
        );
    }
}

#[test]
fn test_targeted_slxb_header_absurd_counts_and_truncations_return_err() {
    let valid_bytes = compile_fixture_binary().to_bytes();

    // 1. Corrupt magic header
    let mut bad_magic = valid_bytes.clone();
    bad_magic[0] = b'X';
    assert!(BinaryProgram::from_bytes(&bad_magic).is_err());

    // 2. Corrupt version major
    let mut bad_version = valid_bytes.clone();
    bad_version[4] = 99;
    assert!(BinaryProgram::from_bytes(&bad_version).is_err());

    // 3. Inject 0xFFFF_FFFF static symbol / constant pool count to prove zero OOM panic
    if valid_bytes.len() > 24 {
        for pos in 12..valid_bytes.len().saturating_sub(4).min(96) {
            let mut count_bomb = valid_bytes.clone();
            count_bomb[pos..pos + 4].copy_from_slice(&u32::MAX.to_le_bytes());
            let caught = std::panic::catch_unwind(|| {
                if let Ok(program) = BinaryProgram::from_bytes(&count_bomb) {
                    let _ = RuntimeContext::new(RuntimeOptions::new().with_binary(program));
                }
            });
            assert!(
                caught.is_ok(),
                "Absurd count injection at offset {pos} caused panic!"
            );
        }
    }
}

#[test]
fn test_targeted_verifier_rejects_every_malformed_bytecode_category() {
    let base_valid = || {
        let mut program = BinaryProgram::new();
        let mut func = CompiledFunction::new("main", 0, 0, 4);
        func.instructions
            .push(Instruction::format_b_signed(OpCode::LoadI32, 0, 0, 1));
        func.instructions
            .push(Instruction::format_a(OpCode::ReturnValue, 0, 0, 0, 0));
        program.functions.push(func);
        program.entry_point_id = Some(0);
        program
    };

    // 1. Invalid raw opcode 0xFE
    let mut prog_bad_op = base_valid();
    prog_bad_op.functions[0].instructions[0] =
        Instruction::from_le_bytes([0xFE, 0, 0, 0, 0, 0, 0, 0]);
    assert!(RuntimeContext::new(RuntimeOptions::new().with_binary(prog_bad_op)).is_err());

    // 2. Out-of-window register in arithmetic and closure instructions
    let mut prog_bad_reg = base_valid();
    prog_bad_reg.functions[0].instructions[0] = Instruction::format_a(OpCode::AddI32, 0, 0, 1, 9);
    assert!(RuntimeContext::new(RuntimeOptions::new().with_binary(prog_bad_reg)).is_err());

    let mut prog_bad_closure = base_valid();
    prog_bad_closure.functions[0].instructions[0] =
        Instruction::format_call(OpCode::PackClosure, 0, 0, 3, 2);
    assert!(RuntimeContext::new(RuntimeOptions::new().with_binary(prog_bad_closure)).is_err());

    // 3. Out-of-bounds Jump, ForLoopI32, and JumpIfLtI32
    let mut prog_bad_jump = base_valid();
    prog_bad_jump.functions[0].instructions[0] = Instruction::format_c(OpCode::Jump, 0, 0, 99);
    assert!(RuntimeContext::new(RuntimeOptions::new().with_binary(prog_bad_jump)).is_err());

    let mut prog_bad_for = base_valid();
    prog_bad_for.functions[0].instructions[0] =
        Instruction::format_a(OpCode::ForLoopI32, 0, 0, 1, -20i16 as u16);
    assert!(RuntimeContext::new(RuntimeOptions::new().with_binary(prog_bad_for)).is_err());

    // 4. Out-of-bounds constant pool ID (including empty pool!)
    let mut prog_bad_const = base_valid();
    prog_bad_const.functions[0].instructions[0] = Instruction::format_b(OpCode::LoadConst, 0, 0, 0);
    assert!(RuntimeContext::new(RuntimeOptions::new().with_binary(prog_bad_const)).is_err());

    // 5. Out-of-bounds direct call function ID
    let mut prog_bad_call = base_valid();
    prog_bad_call.functions[0].instructions[0] =
        Instruction::format_call(OpCode::CallDirect, 0, 42, 0, 0);
    assert!(RuntimeContext::new(RuntimeOptions::new().with_binary(prog_bad_call)).is_err());

    // 6. Out-of-bounds static slot
    let mut prog_bad_static = base_valid();
    prog_bad_static.static_space_size = 1;
    prog_bad_static.functions[0].instructions[0] =
        Instruction::format_b(OpCode::GetStaticDirect, 0, 0, 5);
    assert!(RuntimeContext::new(RuntimeOptions::new().with_binary(prog_bad_static)).is_err());

    // 7. Invalid exception handler PCs / register
    let mut prog_bad_eh = base_valid();
    prog_bad_eh.functions[0]
        .exception_handlers
        .push(ExceptionHandlerEntry {
            try_start_pc: 0,
            try_end_pc: 99,
            catch_type_symbol: None,
            catch_reg: 0,
            handler_pc: 1,
            finally_pc: None,
            try_ref_regs: vec![],
        });
    assert!(RuntimeContext::new(RuntimeOptions::new().with_binary(prog_bad_eh)).is_err());

    // 8. Absurd vtable_id (u32::MAX) must return Err cleanly without OOM/overflow
    let mut prog_bad_vtable = base_valid();
    prog_bad_vtable
        .classes
        .push(CompiledClass::new("EvilClass", u32::MAX, None, 8));
    assert!(RuntimeContext::new(RuntimeOptions::new().with_binary(prog_bad_vtable)).is_err());
}

#[test]
fn test_slxarc_rejects_zip_slip_missing_manifest_and_tampered_native_sha256() {
    let temp_dir = TempDir::new().expect("tempdir");

    // A. Zip-Slip traversal entry inside .slxarc
    let slip_archive = temp_dir.path().join("zip_slip.slxarc");
    let extract_slip = temp_dir.path().join("extract_slip");
    {
        let file = File::create(&slip_archive).expect("create zip");
        let mut writer = ZipWriter::new(file);
        writer
            .start_file("../escaped.slxb", SimpleFileOptions::default())
            .expect("start entry");
        writer.write_all(b"evil").expect("write entry");
        writer.finish().expect("finish zip");
    }
    assert!(SlxarcArchive::extract_and_load(&slip_archive, &extract_slip).is_err());

    // B. Malformed manifest.toml and incompatible supported_platforms metadata
    let bad_manifest = temp_dir.path().join("bad_manifest.slxarc");
    let extract_bad_manifest = temp_dir.path().join("extract_bad_manifest");
    {
        let file = File::create(&bad_manifest).expect("create zip");
        let mut writer = ZipWriter::new(file);
        writer
            .start_file("manifest.toml", SimpleFileOptions::default())
            .expect("start entry");
        writer
            .write_all(b"{{{ invalid toml")
            .expect("write bad toml");
        writer.finish().expect("finish zip");
    }
    assert!(SlxarcArchive::extract_and_load(&bad_manifest, &extract_bad_manifest).is_err());

    let wrong_platform = temp_dir.path().join("wrong_platform.slxarc");
    let extract_wrong_platform = temp_dir.path().join("extract_wrong_platform");
    {
        let file = File::create(&wrong_platform).expect("create zip");
        let mut writer = ZipWriter::new(file);
        writer
            .start_file("manifest.toml", SimpleFileOptions::default())
            .expect("start entry");
        writer
            .write_all(
                br#"[package]
name = "wrong_plat"
version = "1.0.0"

supported_platforms = ["nonexistent-arch-os"]

[profiles.release]
platforms = ["nonexistent-arch-os"]
"#,
            )
            .expect("write platform toml");
        writer
            .start_file("bin/program.slxb", SimpleFileOptions::default())
            .expect("start program");
        writer
            .write_all(&compile_fixture_binary().to_bytes())
            .expect("write bytes");
        writer.finish().expect("finish zip");
    }
    assert!(SlxarcArchive::extract_and_load(&wrong_platform, &extract_wrong_platform).is_err());

    // C. Tampered native library SHA-256 digest mismatch
    #[cfg(target_os = "windows")]
    let native_ext = "dll";
    #[cfg(target_os = "macos")]
    let native_ext = "dylib";
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let native_ext = "so";

    let host_triple = solix_common::project::host_target_triple();
    let native_path = format!("natives/{host_triple}/libtampered.{native_ext}");
    let fake_lib_file = temp_dir.path().join("fake.bin");
    fs::write(&fake_lib_file, b"legitimate native binary").expect("write fake lib");
    let _expected_sha = sha256_file(&fake_lib_file).expect("sha256");

    let mut binary = compile_fixture_binary();
    binary.native_library_checksums.insert(
        format!("libtampered.{native_ext}"),
        "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
    );

    let tampered_archive = temp_dir.path().join("tampered_sha.slxarc");
    let extract_tampered = temp_dir.path().join("extract_tampered");
    {
        let file = File::create(&tampered_archive).expect("create zip");
        let mut writer = ZipWriter::new(file);
        writer
            .start_file("manifest.toml", SimpleFileOptions::default())
            .expect("manifest");
        let manifest_toml = format!(
            r#"[package]
name = "tampered"
version = "1.0.0"

supported_platforms = ["{host_triple}"]

[profiles.release]
platforms = ["{host_triple}"]
"#
        );
        writer.write_all(manifest_toml.as_bytes()).expect("write");
        writer
            .start_file("bin/program.slxb", SimpleFileOptions::default())
            .expect("program");
        writer.write_all(&binary.to_bytes()).expect("write");
        writer
            .start_file(&native_path, SimpleFileOptions::default())
            .expect("native");
        writer
            .write_all(b"tampered payload bytes")
            .expect("write tampered");
        writer.finish().expect("finish zip");
    }

    let load_err = SlxarcArchive::extract_and_load(&tampered_archive, &extract_tampered);
    assert!(
        load_err.is_err(),
        "Archive with tampered native SHA-256 checksum MUST fail verification"
    );
}
