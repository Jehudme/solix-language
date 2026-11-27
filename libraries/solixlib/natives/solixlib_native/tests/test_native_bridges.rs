//! Integration and unit tests for solixlib native bridges (formatting, JSON, compression).

use solixlib_native::string_prim::{format_template_with_arguments, format_with_specifier};

#[test]
fn test_format_hex_binary_octal_specifiers() {
    let arguments = vec![String::from("255"), String::from("10"), String::from("64")];
    let template = "hex: {:#08x}, bin: {:b}, oct: {:o}";
    let formatted_output = format_template_with_arguments(template, &arguments)
        .expect("Formatting must succeed for valid specifiers");

    assert_eq!(formatted_output, "hex: 0x0000ff, bin: 1010, oct: 100");
}

#[test]
fn test_format_uppercase_hex() {
    let arguments = vec![String::from("255")];
    let template = "{:X}";
    let formatted_output = format_template_with_arguments(template, &arguments)
        .expect("Uppercase hex formatting must succeed");

    assert_eq!(formatted_output, "FF");
}

#[test]
fn test_format_alignment_and_padding() {
    let arguments = vec![String::from("solix")];

    let left_aligned = format_template_with_arguments("{:<10}", &arguments).unwrap();
    assert_eq!(left_aligned, "solix     ");

    let right_aligned = format_template_with_arguments("{:>10}", &arguments).unwrap();
    assert_eq!(right_aligned, "     solix");

    let center_aligned = format_template_with_arguments("{:^11}", &arguments).unwrap();
    assert_eq!(center_aligned, "   solix   ");

    let custom_fill_left = format_template_with_arguments("{:*<10}", &arguments).unwrap();
    assert_eq!(custom_fill_left, "solix*****");

    let custom_fill_right = format_template_with_arguments("{:_>10}", &arguments).unwrap();
    assert_eq!(custom_fill_right, "_____solix");
}

#[test]
fn test_format_floating_point_precision() {
    let arguments = vec![String::from("3.14159265")];
    let formatted_output = format_template_with_arguments("{:.2f}", &arguments)
        .expect("Float precision formatting must succeed");

    assert_eq!(formatted_output, "3.14");
}

#[test]
fn test_format_explicit_positional_indices() {
    let arguments = vec![String::from("first"), String::from("second")];
    let template = "{1} comes after {0}, repeated: {0}";
    let formatted_output = format_template_with_arguments(template, &arguments)
        .expect("Positional indices must succeed");

    assert_eq!(
        formatted_output,
        "second comes after first, repeated: first"
    );
}

#[test]
fn test_format_escaped_curly_braces() {
    let arguments = vec![String::from("value")];
    let template = "{{ escaped }} and replaced: {}";
    let formatted_output =
        format_template_with_arguments(template, &arguments).expect("Escaped braces must succeed");

    assert_eq!(formatted_output, "{ escaped } and replaced: value");
}

#[test]
fn test_format_negative_unclosed_brace_throws_e0331() {
    let arguments = vec![String::from("test")];
    let template = "unclosed {specifier";
    let format_result = format_template_with_arguments(template, &arguments);

    assert!(format_result.is_err());
    let error_message = format_result.unwrap_err();
    assert!(
        error_message.contains("[E0331]"),
        "Error message must contain diagnostic code [E0331], got: {error_message}"
    );
}

#[test]
fn test_format_negative_invalid_type_character_throws_e0331() {
    let arguments = vec![String::from("123")];
    let template = "invalid specifier: {:q}";
    let format_result = format_template_with_arguments(template, &arguments);

    assert!(format_result.is_err());
    let error_message = format_result.unwrap_err();
    assert!(
        error_message.contains("[E0331]"),
        "Error message must contain diagnostic code [E0331], got: {error_message}"
    );
}

#[test]
fn test_format_negative_trailing_garbage_throws_e0331() {
    let format_result = format_with_specifier("123", "08x_extra");

    assert!(format_result.is_err());
    let error_message = format_result.unwrap_err();
    assert!(
        error_message.contains("[E0331]"),
        "Error message must contain diagnostic code [E0331], got: {error_message}"
    );
}

#[test]
fn test_json_parse_and_pointer_query_logic() {
    let raw_json_text = r#"{"name": "Solix", "version": 2, "features": ["fast", "safe"]}"#;
    let parsed_root: serde_json::Value =
        serde_json::from_str(raw_json_text).expect("Valid JSON must parse cleanly");

    let name_leaf = parsed_root
        .pointer("/name")
        .expect("Pointer /name must exist");
    assert_eq!(name_leaf.as_str().unwrap(), "Solix");

    let first_feature = parsed_root
        .pointer("/features/0")
        .expect("Pointer /features/0 must exist");
    assert_eq!(first_feature.as_str().unwrap(), "fast");

    let non_existent = parsed_root.pointer("/missing/item");
    assert!(non_existent.is_none());
}

#[test]
fn test_json_negative_syntax_error_reporting() {
    let malformed_json = r#"{"name": "Solix", unquoted_key: 123}"#;
    let parse_result = serde_json::from_str::<serde_json::Value>(malformed_json);

    assert!(parse_result.is_err());
    let parse_error = parse_result.unwrap_err();
    let diagnostic_string = format!(
        "[E0332] Failed to parse JSON string at line {}, column {}: {}",
        parse_error.line(),
        parse_error.column(),
        parse_error
    );

    assert!(diagnostic_string.contains("[E0332]"));
    assert!(diagnostic_string.contains("line 1"));
}

#[test]
fn test_gzip_compression_and_decompression_roundtrip() {
    use flate2::read::GzDecoder;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::{Read, Write};

    let original_payload = b"Solix standard library compression test payload string!";
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(original_payload)
        .expect("Write to GzEncoder must succeed");
    let compressed_bytes = encoder.finish().expect("Finishing GzEncoder must succeed");

    assert!(!compressed_bytes.is_empty());
    assert_ne!(&compressed_bytes[..], original_payload);

    let mut decoder = GzDecoder::new(&compressed_bytes[..]);
    let mut decompressed_buffer = Vec::new();
    decoder
        .read_to_end(&mut decompressed_buffer)
        .expect("GzDecoder read must succeed");

    assert_eq!(&decompressed_buffer[..], original_payload);
}

#[test]
fn test_zlib_compression_and_decompression_roundtrip() {
    use flate2::read::ZlibDecoder;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::{Read, Write};

    let original_payload = b"Zlib compression verification payload for Solix standard library.";
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(original_payload)
        .expect("Write to ZlibEncoder must succeed");
    let compressed_bytes = encoder
        .finish()
        .expect("Finishing ZlibEncoder must succeed");

    let mut decoder = ZlibDecoder::new(&compressed_bytes[..]);
    let mut decompressed_buffer = Vec::new();
    decoder
        .read_to_end(&mut decompressed_buffer)
        .expect("ZlibDecoder read must succeed");

    assert_eq!(&decompressed_buffer[..], original_payload);
}

#[test]
fn test_deflate_negative_corrupted_payload_throws_e0334() {
    use flate2::read::DeflateDecoder;
    use std::io::Read;

    let corrupted_bytes = [0xFF, 0xFE, 0xFD, 0xFC, 0x01, 0x02];
    let mut decoder = DeflateDecoder::new(&corrupted_bytes[..]);
    let mut output_buffer = Vec::new();
    let read_result = decoder.read_to_end(&mut output_buffer);

    assert!(read_result.is_err());
    let decode_error = read_result.unwrap_err();
    let formatted_error = format!(
        "[E0334] Failed to decompress buffer using algorithm 'deflate': {}",
        decode_error
    );

    assert!(formatted_error.contains("[E0334]"));
    assert!(formatted_error.contains("algorithm 'deflate'"));
}

mod bridge_hardening_tests {
    use solixlib_native::fs_path::{file_native_read_all_bytes, file_native_read_all_text};
    use solixlib_native::net_bridge::{
        socket_native_close, socket_native_set_read_timeout, socket_native_set_write_timeout,
        tcp_listener_native_accept, tcp_listener_native_bind, tcp_stream_native_connect_timeout,
        tcp_stream_native_read, tcp_stream_native_write,
    };
    use solixlib_native::sys_process::{
        env_native_get_all_env, env_native_get_env, env_native_set_env,
    };
    use solixlib_native::{
        alloc_solix_chars, read_solix_chars, solix_register_natives, SolixCValue, SolixVmContext,
        SOLIX_VM_CONTEXT_MAGIC,
    };
    use std::collections::HashMap;
    use std::ffi::{c_char, c_void};

    #[derive(Clone)]
    enum MockHeapItem {
        StringUtf8(Vec<u8>),
        CharArray(Vec<u32>),
        ByteArray(Vec<u8>),
    }

    struct MockBridgeHeap {
        next_addr: u32,
        entries: HashMap<u32, MockHeapItem>,
    }

    impl MockBridgeHeap {
        fn new() -> Self {
            Self {
                next_addr: 100,
                entries: HashMap::new(),
            }
        }

        fn insert(&mut self, item: MockHeapItem) -> u32 {
            let addr = self.next_addr;
            self.next_addr += 16;
            self.entries.insert(addr, item);
            addr
        }
    }

    unsafe extern "C" fn mock_get_i32(_: *mut c_void, _: u32, _: u32) -> i32 {
        0
    }
    unsafe extern "C" fn mock_set_i32(_: *mut c_void, _: u32, _: u32, _: i32) {}
    unsafe extern "C" fn mock_array_get(_: *mut c_void, _: u32, _: u32) -> u64 {
        0
    }
    unsafe extern "C" fn mock_array_set(_: *mut c_void, _: u32, _: u32, _: u64) {}
    unsafe extern "C" fn mock_alloc_array(_: *mut c_void, _: u32, _: u8) -> u32 {
        0
    }
    unsafe extern "C" fn mock_alloc_object(_: *mut c_void, _: u32, _: u32) -> u32 {
        0
    }
    unsafe extern "C" fn mock_read_field_u64(_: *mut c_void, _: u32, _: u32) -> u64 {
        0
    }
    unsafe extern "C" fn mock_write_field_u64(_: *mut c_void, _: u32, _: u32, _: u64) {}
    unsafe extern "C" fn mock_get_vtable(_: *mut c_void, _: *const c_char) -> u32 {
        0
    }
    unsafe extern "C" fn mock_get_metric(_: *mut c_void, _: u32) -> u64 {
        0
    }
    unsafe extern "C" fn mock_collect_cycles(_: *mut c_void) -> u64 {
        0
    }
    unsafe extern "C" fn mock_set_threshold(_: *mut c_void, _: u32) {}

    unsafe extern "C" fn mock_alloc_string(vm_ptr: *mut c_void, ptr: *const u8, len: u32) -> u32 {
        let heap = unsafe { &mut *(vm_ptr as *mut MockBridgeHeap) };
        let bytes = if ptr.is_null() || len == 0 {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts(ptr, len as usize) }.to_vec()
        };
        heap.insert(MockHeapItem::StringUtf8(bytes))
    }

    unsafe extern "C" fn mock_alloc_chars(vm_ptr: *mut c_void, ptr: *const u8, len: u32) -> u32 {
        let heap = unsafe { &mut *(vm_ptr as *mut MockBridgeHeap) };
        let bytes = if ptr.is_null() || len == 0 {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts(ptr, len as usize) }.to_vec()
        };
        let code_points: Vec<u32> = String::from_utf8_lossy(&bytes)
            .chars()
            .map(|ch| ch as u32)
            .collect();
        heap.insert(MockHeapItem::CharArray(code_points))
    }

    unsafe extern "C" fn mock_alloc_byte_array(
        vm_ptr: *mut c_void,
        ptr: *const u8,
        len: u32,
    ) -> u32 {
        let heap = unsafe { &mut *(vm_ptr as *mut MockBridgeHeap) };
        let bytes = if ptr.is_null() || len == 0 {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts(ptr, len as usize) }.to_vec()
        };
        heap.insert(MockHeapItem::ByteArray(bytes))
    }

    unsafe extern "C" fn mock_array_len(vm_ptr: *mut c_void, addr: u32) -> u32 {
        let heap = unsafe { &*(vm_ptr as *const MockBridgeHeap) };
        match heap.entries.get(&addr) {
            Some(MockHeapItem::StringUtf8(bytes)) => bytes.len() as u32,
            Some(MockHeapItem::CharArray(chars)) => chars.len() as u32,
            Some(MockHeapItem::ByteArray(bytes)) => bytes.len() as u32,
            None => 0,
        }
    }

    unsafe extern "C" fn mock_read_bytes(
        vm_ptr: *mut c_void,
        addr: u32,
        out_buf: *mut u8,
        cap: u32,
    ) -> u32 {
        let heap = unsafe { &*(vm_ptr as *const MockBridgeHeap) };
        let encoded: Vec<u8> = match heap.entries.get(&addr) {
            Some(MockHeapItem::StringUtf8(bytes)) | Some(MockHeapItem::ByteArray(bytes)) => {
                bytes.clone()
            }
            Some(MockHeapItem::CharArray(chars)) => {
                let text: String = chars
                    .iter()
                    .map(|&cp| char::from_u32(cp).unwrap_or(char::REPLACEMENT_CHARACTER))
                    .collect();
                text.into_bytes()
            }
            None => Vec::new(),
        };
        let copy_len = encoded.len().min(cap as usize);
        if !out_buf.is_null() && copy_len > 0 {
            unsafe {
                std::ptr::copy_nonoverlapping(encoded.as_ptr(), out_buf, copy_len);
            }
        }
        encoded.len() as u32
    }

    fn make_mock_ctx(heap: &mut MockBridgeHeap) -> SolixVmContext {
        SolixVmContext {
            magic: SOLIX_VM_CONTEXT_MAGIC,
            vm_ptr: (heap as *mut MockBridgeHeap).cast::<c_void>(),
            get_field_i32: mock_get_i32,
            set_field_i32: mock_set_i32,
            read_bytes: mock_read_bytes,
            alloc_string: mock_alloc_string,
            alloc_chars: mock_alloc_chars,
            alloc_byte_array: mock_alloc_byte_array,
            alloc_array: mock_alloc_array,
            array_len: mock_array_len,
            array_get_u64: mock_array_get,
            array_set_u64: mock_array_set,
            alloc_object: mock_alloc_object,
            read_field_u64: mock_read_field_u64,
            write_field_u64: mock_write_field_u64,
            get_class_vtable_id: mock_get_vtable,
            get_vm_metric: mock_get_metric,
            collect_cycles: mock_collect_cycles,
            set_cycle_threshold: mock_set_threshold,
            get_field_i32_by_index: mock_get_i32,
            set_field_i32_by_index: mock_set_i32,
            get_field_i32_by_offset: mock_get_i32,
            set_field_i32_by_offset: mock_set_i32,
            read_field_u64_by_index: mock_read_field_u64,
            write_field_u64_by_index: mock_write_field_u64,
            read_field_u64_by_offset: mock_read_field_u64,
            write_field_u64_by_offset: mock_write_field_u64,
        }
    }

    #[test]
    fn test_utf8_char_roundtrip_and_register_null_guards() {
        unsafe {
            solix_register_natives(std::ptr::null_mut(), None);
        }
        let mut heap = MockBridgeHeap::new();
        let mut ctx = make_mock_ctx(&mut heap);
        let ctx_ptr = (&mut ctx as *mut SolixVmContext).cast::<c_void>();

        let original = "café 🚀 Solix Ω";
        let chars_addr = unsafe { alloc_solix_chars(ctx_ptr, original) };
        let decoded = unsafe { read_solix_chars(ctx_ptr, chars_addr) };
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_virtual_env_is_thread_safe_and_does_not_mutate_os_env() {
        let mut heap = MockBridgeHeap::new();
        let mut ctx = make_mock_ctx(&mut heap);
        let ctx_ptr = (&mut ctx as *mut SolixVmContext).cast::<c_void>();

        let env_key = "SOLIX_TEST_VIRTUAL_ENV_294";
        assert!(std::env::var(env_key).is_err());

        let key_addr = unsafe { alloc_solix_chars(ctx_ptr, env_key) };
        let val_addr = unsafe { alloc_solix_chars(ctx_ptr, "virtual_value_42") };
        let set_args = [
            SolixCValue::ref_val(key_addr),
            SolixCValue::ref_val(val_addr),
        ];
        let mut err = SolixCValue::null();
        unsafe {
            env_native_set_env(ctx_ptr, set_args.as_ptr(), 2, &mut err);
        }
        assert!(err.is_null());

        // Host OS environment was NOT unsafely mutated via std::env::set_var.
        assert!(std::env::var(env_key).is_err());

        // Virtual get_env & get_all_env reflect the updated variable cleanly.
        let get_args = [SolixCValue::ref_val(key_addr)];
        let fetched = unsafe { env_native_get_env(ctx_ptr, get_args.as_ptr(), 1, &mut err) };
        assert_eq!(
            unsafe { read_solix_chars(ctx_ptr, fetched.as_ref()) },
            "virtual_value_42"
        );

        let all_env = unsafe { env_native_get_all_env(ctx_ptr, std::ptr::null(), 0, &mut err) };
        let all_text = unsafe { read_solix_chars(ctx_ptr, all_env.as_ref()) };
        assert!(all_text.contains("SOLIX_TEST_VIRTUAL_ENV_294=virtual_value_42"));
    }

    #[test]
    fn test_file_read_errors_surface_via_out_error_instead_of_empty_string() {
        let mut heap = MockBridgeHeap::new();
        let mut ctx = make_mock_ctx(&mut heap);
        let ctx_ptr = (&mut ctx as *mut SolixVmContext).cast::<c_void>();

        let missing_path = std::env::temp_dir().join("solix_missing_file_294_test.txt");
        let _ = std::fs::remove_file(&missing_path);
        let path_addr = unsafe { alloc_solix_chars(ctx_ptr, &missing_path.to_string_lossy()) };
        let args = [SolixCValue::ref_val(path_addr)];
        let mut out_error = SolixCValue::null();

        let text_res =
            unsafe { file_native_read_all_text(ctx_ptr, args.as_ptr(), 1, &mut out_error) };
        assert!(text_res.is_null());
        assert_eq!(out_error.tag, 5);
        let err_msg = unsafe { read_solix_chars(ctx_ptr, out_error.as_ref()) };
        assert!(
            err_msg.contains("IOException"),
            "unexpected error: {err_msg}"
        );

        // Invalid UTF-8 triggers out_error on read_all_text while read_all_bytes succeeds.
        let bin_path =
            std::env::temp_dir().join(format!("solix_bin_{}_294.dat", std::process::id()));
        std::fs::write(&bin_path, [0xFFu8, 0xFE, 0x00, 0x41]).expect("write test binary file");
        let bin_addr = unsafe { alloc_solix_chars(ctx_ptr, &bin_path.to_string_lossy()) };
        let bin_args = [SolixCValue::ref_val(bin_addr)];

        let mut utf8_error = SolixCValue::null();
        let utf8_res =
            unsafe { file_native_read_all_text(ctx_ptr, bin_args.as_ptr(), 1, &mut utf8_error) };
        assert!(utf8_res.is_null());
        assert!(!utf8_error.is_null());

        let mut bytes_error = SolixCValue::null();
        let bytes_res =
            unsafe { file_native_read_all_bytes(ctx_ptr, bin_args.as_ptr(), 1, &mut bytes_error) };
        assert!(bytes_error.is_null());
        assert_ne!(bytes_res.as_ref(), 0);
        let _ = std::fs::remove_file(&bin_path);
    }

    #[test]
    fn test_network_timeouts_and_error_propagation() {
        let mut heap = MockBridgeHeap::new();
        let mut ctx = make_mock_ctx(&mut heap);
        let ctx_ptr = (&mut ctx as *mut SolixVmContext).cast::<c_void>();
        let mut err = SolixCValue::null();

        // Invalid listener ID reports accurate invalid ID error.
        let invalid_id = [SolixCValue::i64_val(999_999_999)];
        let bad_accept =
            unsafe { tcp_listener_native_accept(ctx_ptr, invalid_id.as_ptr(), 1, &mut err) };
        assert_eq!(bad_accept.as_i64(), 0);
        assert!(unsafe { read_solix_chars(ctx_ptr, err.as_ref()) }.contains("invalid or closed"));

        // Bind ephemeral TCP listener on localhost, connect with timeout, set short read timeout.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("ephemeral bind");
        let port = listener.local_addr().expect("local addr").port();
        let endpoint = format!("127.0.0.1:{port}");
        let ep_addr = unsafe { alloc_solix_chars(ctx_ptr, &endpoint) };

        let mut connect_err = SolixCValue::null();
        let conn_args = [SolixCValue::ref_val(ep_addr), SolixCValue::i32_val(500)];
        let client_val = unsafe {
            tcp_stream_native_connect_timeout(ctx_ptr, conn_args.as_ptr(), 2, &mut connect_err)
        };
        assert!(connect_err.is_null());
        let client_id = client_val.as_i64();
        assert!(client_id > 0);

        let (_server_stream, _) = listener.accept().expect("accept client");

        let timeout_args = [SolixCValue::i64_val(client_id), SolixCValue::i32_val(30)];
        let mut timeout_err = SolixCValue::null();
        let set_read_ok = unsafe {
            socket_native_set_read_timeout(ctx_ptr, timeout_args.as_ptr(), 2, &mut timeout_err)
        };
        assert!(set_read_ok.as_bool());
        let set_write_ok = unsafe {
            socket_native_set_write_timeout(ctx_ptr, timeout_args.as_ptr(), 2, &mut timeout_err)
        };
        assert!(set_write_ok.as_bool());

        // Reading with no incoming bytes must time out after ~30ms and set out_error instead of blocking.
        let read_args = [SolixCValue::i64_val(client_id), SolixCValue::i32_val(16)];
        let mut read_err = SolixCValue::null();
        let read_res =
            unsafe { tcp_stream_native_read(ctx_ptr, read_args.as_ptr(), 2, &mut read_err) };
        assert!(read_res.is_null());
        assert!(!read_err.is_null());

        let close_args = [SolixCValue::i64_val(client_id)];
        let closed =
            unsafe { socket_native_close(ctx_ptr, close_args.as_ptr(), 1, std::ptr::null_mut()) };
        assert!(closed.as_bool());
        let _ = tcp_listener_native_bind;
        let _ = tcp_stream_native_write;
    }

    #[test]
    fn test_mutex_poison_recovery_across_threads() {
        let mutex = std::sync::Arc::new(std::sync::Mutex::new(41i32));
        let mutex_clone = std::sync::Arc::clone(&mutex);
        let join_handle = std::thread::spawn(move || {
            let mut guard = mutex_clone.lock().expect("lock unpoisoned mutex");
            *guard = 42;
            panic!("intentional poison for recovery test");
        });
        assert!(join_handle.join().is_err());

        // Standard lock returns Err(PoisonError), whereas lock_poison_safe recovers state cleanly.
        assert!(mutex.lock().is_err());
        let recovered = mutex
            .lock()
            .unwrap_or_else(|poisoned_guard| poisoned_guard.into_inner());
        assert_eq!(*recovered, 42);
    }
}
