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

#[test]
fn test_slxb_rejects_invalid_flag_bytes_malformed_type_json_and_huge_instance_size() {
    use solix_common::diagnostics::EmissionError;

    let mut program = BinaryProgram::new();
    program
        .classes
        .push(CompiledClass::new("SizedObj", 0, None, 24));
    let mut func = CompiledFunction::new("main", 0, 0, 2);
    func.instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));
    program.functions.push(func);
    program.entry_point_id = Some(0);

    let valid_bytes = program
        .try_to_bytes()
        .expect("Valid BinaryProgram must serialize cleanly");

    // 1. Corrupt option/bool flag byte at Header entry_point_id tag (offset 12) to > 1
    let mut bad_flag = valid_bytes.clone();
    bad_flag[12] = 2;
    assert!(
        matches!(
            BinaryProgram::from_bytes(&bad_flag),
            Err(EmissionError::CorruptBinaryContainer { section, .. }) if section == "Header"
        ),
        "Flag byte > 1 must return CorruptBinaryContainer instead of coercing to false/None"
    );

    // 2. Corrupt JSON Type descriptor inside Functions section
    let mut bad_json = valid_bytes.clone();
    let json_pos = bad_json
        .windows(b"{\"Primitive\"".len())
        .position(|window| window == b"{\"Primitive\"")
        .expect("Serialized function return_type JSON must be present");
    bad_json[json_pos] = b'!';
    assert!(
        matches!(
            BinaryProgram::from_bytes(&bad_json),
            Err(EmissionError::CorruptBinaryContainer { section, .. }) if section == "Functions"
        ),
        "Malformed Type JSON descriptor must return CorruptBinaryContainer instead of Void"
    );

    // 3. Inject unreasonable class instance_size (128 MiB > MAX_INSTANCE_SIZE_BYTES)
    let mut bad_size = valid_bytes;
    let needle = 24u64.to_le_bytes();
    let class_name_pos = bad_size
        .windows(b"SizedObj".len())
        .position(|window| window == b"SizedObj")
        .expect("Serialized class symbol must be present");
    let size_offset = bad_size[class_name_pos..]
        .windows(8)
        .position(|window| window == needle)
        .map(|rel| class_name_pos + rel)
        .expect("Class instance_size u64 must follow symbol");
    bad_size[size_offset..size_offset + 8].copy_from_slice(&(128u64 * 1024 * 1024).to_le_bytes());
    assert!(
        matches!(
            BinaryProgram::from_bytes(&bad_size),
            Err(EmissionError::CorruptBinaryContainer { section, .. }) if section == "Classes"
        ),
        "Unreasonable instance_size must return CorruptBinaryContainer in Classes"
    );
}

#[test]
fn test_verifier_rejects_i32_branch_overflow_and_object_tuple_step_bounds() {
    use solix_runtime::LoadedBinary;
    use solix_runtime::verifier::verify_program;

    let make_class_prog = |inst: Instruction, num_regs: u16| {
        let mut program = BinaryProgram::new();
        let mut obj_class = CompiledClass::new("Obj", 0, None, 16);
        obj_class.itable.push((1, vec![0]));
        program.classes.push(obj_class);
        let mut func = CompiledFunction::new("main", 0, 0, num_regs);
        func.instructions.push(inst);
        func.instructions
            .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));
        program.functions.push(func);
        program.entry_point_id = Some(0);
        program
    };

    // 1. i32::MAX branch offset at PC 1 (would panic on checked-less `+` in debug or wrap in release)
    let mut overflow_branch = BinaryProgram::new();
    let mut func_branch = CompiledFunction::new("branch_overflow", 0, 0, 2);
    func_branch
        .instructions
        .push(Instruction::format_a(OpCode::Nop, 0, 0, 0, 0));
    func_branch
        .instructions
        .push(Instruction::format_c(OpCode::Jump, 0, 0, i32::MAX));
    overflow_branch.functions.push(func_branch);
    assert!(
        RuntimeContext::new(RuntimeOptions::new().with_binary(overflow_branch)).is_err(),
        "PC + i32::MAX branch offset must fail cleanly without overflow panic"
    );

    // 2. ForLoopI32 dynamic step register out of window & invalid low flags
    assert!(
        RuntimeContext::new(RuntimeOptions::new().with_binary(make_class_prog(
            Instruction::format_a(OpCode::ForLoopI32, 0x80 | 5, 0, 1, -1i16 as u16),
            3,
        )))
        .is_err(),
        "ForLoopI32 step register exceeding num_registers must fail verifier"
    );
    assert!(
        RuntimeContext::new(RuntimeOptions::new().with_binary(make_class_prog(
            Instruction::format_a(OpCode::ForLoopI32, 0x03, 0, 1, -1i16 as u16),
            4,
        )))
        .is_err(),
        "ForLoopI32 reserved low flags without 0x80 must fail verifier"
    );

    // 3. AllocObject unknown vtable_id & undersized allocation when class metadata is present
    assert!(
        RuntimeContext::new(RuntimeOptions::new().with_binary(make_class_prog(
            Instruction::format_a(OpCode::AllocObject, 0, 0, 99, 16),
            2,
        )))
        .is_err(),
        "AllocObject with unknown vtable_id must fail verifier"
    );
    assert!(
        RuntimeContext::new(RuntimeOptions::new().with_binary(make_class_prog(
            Instruction::format_a(OpCode::AllocObject, 0, 0, 0, 4),
            2,
        )))
        .is_err(),
        "AllocObject smaller than class instance_size must fail verifier"
    );

    // 4. Field offset bounds & alignment checks (header overwrite, unaligned, past max_instance_size)
    assert!(
        RuntimeContext::new(RuntimeOptions::new().with_binary(make_class_prog(
            Instruction::format_a(OpCode::GetField, 0, 1, 0, 4),
            2,
        )))
        .is_err(),
        "GetField into 8-byte object header (< 8) must fail verifier"
    );
    assert!(
        RuntimeContext::new(RuntimeOptions::new().with_binary(make_class_prog(
            Instruction::format_a(OpCode::GetFieldI32, 0, 1, 0, 9),
            2,
        )))
        .is_err(),
        "Unaligned GetFieldI32 offset must fail verifier"
    );
    assert!(
        RuntimeContext::new(RuntimeOptions::new().with_binary(make_class_prog(
            Instruction::format_a(OpCode::SetFieldF64, 0, 1, 0, 12),
            2,
        )))
        .is_err(),
        "SetFieldF64 exceeding max_instance_size must fail verifier"
    );

    // 5. Tuple excessive element count / slot bounds & virtual/interface slot bounds
    assert!(
        RuntimeContext::new(RuntimeOptions::new().with_binary(make_class_prog(
            Instruction::format_a(OpCode::AllocTuple, 0, 0, 4097, 0),
            2,
        )))
        .is_err(),
        "AllocTuple beyond MAX_TUPLE_ELEMENTS must fail verifier"
    );
    assert!(
        RuntimeContext::new(RuntimeOptions::new().with_binary(make_class_prog(
            Instruction::format_a(OpCode::GetTuple, 0, 1, 0, 4096),
            2,
        )))
        .is_err(),
        "GetTuple slot beyond MAX_TUPLE_ELEMENTS must fail verifier"
    );
    assert!(
        RuntimeContext::new(RuntimeOptions::new().with_binary(make_class_prog(
            Instruction::format_call(OpCode::CallVirtual, 0, 0, 0, 1),
            2,
        )))
        .is_err(),
        "CallVirtual out-of-range vtable slot must fail verifier"
    );
    assert!(
        RuntimeContext::new(RuntimeOptions::new().with_binary(make_class_prog(
            Instruction::format_call(OpCode::CallInterface, 0, (1 << 8) | 5, 0, 1),
            2,
        )))
        .is_err(),
        "CallInterface out-of-range method slot must fail verifier"
    );

    // 6. Owning binary constant pool isolation must not fall back to another zero-static binary
    let mut first_pool = solix_common::bytecode::ConstantPool::new();
    let _ = first_pool.add_string("first_bin_const");
    let first_unit = LoadedBinary {
        binary_id: 0,
        static_base_offset: 0,
        function_id_offset: 0,
        static_space_size: 0,
        entry_point_id: None,
        entry_point_symbol: None,
        constant_pool: first_pool,
        classes: Vec::new(),
        function_count: 1,
        is_debug_profile: false,
        source_files: Vec::new(),
    };
    let second_unit = LoadedBinary {
        binary_id: 1,
        static_base_offset: 0,
        function_id_offset: 1,
        static_space_size: 0,
        entry_point_id: None,
        entry_point_symbol: None,
        constant_pool: solix_common::bytecode::ConstantPool::new(),
        classes: Vec::new(),
        function_count: 1,
        is_debug_profile: false,
        source_files: Vec::new(),
    };
    let f0 = CompiledFunction::new("bin0_fn", 0, 0, 1);
    let mut f1 = CompiledFunction::new("bin1_fn", 0, 0, 1);
    f1.instructions
        .push(Instruction::format_b(OpCode::LoadConst, 0, 0, 0));
    assert!(
        verify_program(&[first_unit, second_unit], &[f0, f1], 0).is_err(),
        "Second binary with empty pool must not resolve constant index 0 via static_base_offset fallback"
    );

    // 7. Relocating CallInterface across loaded modules must reject iface_id > 255 instead of wrapping u8
    let mut mod_a = BinaryProgram::new();
    for class_idx in 0..250u32 {
        mod_a.classes.push(CompiledClass::new(
            format!("ModAClass{class_idx}"),
            class_idx,
            None,
            8,
        ));
    }
    let mut func_a = CompiledFunction::new("mod_a_fn", 0, 0, 1);
    func_a
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));
    mod_a.functions.push(func_a);

    let mut mod_b = BinaryProgram::new();
    let mut iface_b = CompiledClass::new("ModBIface", 7, None, 8);
    iface_b.itable.push((7, vec![0]));
    mod_b.classes.push(iface_b);
    let mut func_b = CompiledFunction::new("mod_b_main", 0, 0, 2);
    func_b.instructions.push(Instruction::format_call(
        OpCode::CallInterface,
        0,
        7 << 8,
        1,
        1,
    ));
    func_b
        .instructions
        .push(Instruction::format_a(OpCode::Return, 0, 0, 0, 0));
    mod_b.functions.push(func_b);
    mod_b.entry_point = Some("mod_b_main".to_string());

    let err_reloc = RuntimeContext::new(
        RuntimeOptions::new().with_binary(mod_a).with_binary(mod_b),
    )
    .expect_err(
        "CallInterface iface_id > 255 after relocation must fail cleanly instead of truncating",
    );
    assert!(
        err_reloc.to_string().contains("255"),
        "Unexpected error message for relocated CallInterface overflow: {err_reloc}"
    );
}
