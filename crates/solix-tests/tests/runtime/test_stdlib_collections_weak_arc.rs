//! Runtime integration tests verifying stdlib Weak-ARC cycle-free collections,
//! overload resolution for `(string)` vs `(String)` and `: super(...)`, and
//! multi-chunk + chunked-encoding `HttpClient` transactions against a real local HTTP server.

#![forbid(unsafe_code)]

use solix_common::bytecode::BinaryProgram;
use solix_compilation::CompilationContext::{CompilationContext, CompilationOptions};
use solix_runtime::memory::BlockTag;
use solix_runtime::{RuntimeContext, RuntimeError, RuntimeOptions, Value, VirtualMachine};
use solix_tests::common::compile_and_run;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{LazyLock, Mutex};

static TEST_SOCKETS: LazyLock<Mutex<(i64, HashMap<i64, TcpStream>)>> =
    LazyLock::new(|| Mutex::new((1, HashMap::new())));

fn alloc_vm_string(vm: &mut VirtualMachine, text: &str) -> Result<Value, RuntimeError> {
    let raw_bytes = text.as_bytes();
    let payload_addr = vm.heap.allocate(raw_bytes.len(), BlockTag::String)?;
    vm.heap.write_bytes(payload_addr, raw_bytes)?;
    Ok(Value::Ref(payload_addr))
}

fn alloc_vm_byte_array(vm: &mut VirtualMachine, bytes: &[u8]) -> Result<Value, RuntimeError> {
    let payload_addr = vm.heap.allocate(bytes.len() + 8, BlockTag::ArrayRef)?;
    vm.heap.write_u32(payload_addr, bytes.len() as u32)?;
    vm.heap.write_u8(payload_addr + 4, 1)?;
    vm.heap.write_u16(payload_addr + 5, 0)?;
    vm.heap.write_u8(payload_addr + 7, 0)?;
    vm.heap.write_bytes(payload_addr + 8, bytes)?;
    Ok(Value::Ref(payload_addr))
}

fn alloc_vm_char_array(vm: &mut VirtualMachine, text: &str) -> Result<Value, RuntimeError> {
    let chars: Vec<char> = text.chars().collect();
    let payload_addr = vm.heap.allocate(chars.len() * 8 + 8, BlockTag::ArrayRef)?;
    vm.heap.write_u32(payload_addr, chars.len() as u32)?;
    vm.heap.write_u8(payload_addr + 4, 8)?;
    vm.heap.write_u16(payload_addr + 5, 0)?;
    vm.heap.write_u8(payload_addr + 7, 12)?;
    for (char_index, ch_val) in chars.iter().enumerate() {
        vm.heap.write_u64(
            payload_addr + 8 + (char_index as u32) * 8,
            u64::from(*ch_val as u32),
        )?;
    }
    Ok(Value::Ref(payload_addr))
}

fn read_vm_string(vm: &VirtualMachine, val: &Value) -> Result<String, RuntimeError> {
    match val {
        Value::Ref(addr) if *addr > 0 => vm.heap.read_string(*addr),
        _ => Ok(String::new()),
    }
}

fn read_vm_byte_array(vm: &VirtualMachine, val: &Value) -> Result<Vec<u8>, RuntimeError> {
    match val {
        Value::Ref(addr) if *addr > 0 => {
            let array_len = vm.heap.read_u32(*addr)? as usize;
            let bytes = vm.heap.read_bytes(*addr + 8, array_len)?;
            Ok(bytes.to_vec())
        }
        _ => Ok(Vec::new()),
    }
}

fn read_vm_char_array_as_string(vm: &VirtualMachine, val: &Value) -> Result<String, RuntimeError> {
    match val {
        Value::Ref(addr) if *addr > 0 => {
            let array_len = vm.heap.read_u32(*addr)? as usize;
            let elem_size = vm.heap.read_u8(*addr + 4)?;
            if elem_size <= 1 {
                let bytes = vm.heap.read_bytes(*addr + 8, array_len)?;
                return Ok(String::from_utf8_lossy(bytes).into_owned());
            }
            let mut decoded = String::with_capacity(array_len);
            for char_index in 0..array_len {
                let code_point = if elem_size == 8 {
                    vm.heap.read_u64(*addr + 8 + (char_index as u32) * 8)? as u32
                } else {
                    vm.heap.read_u32(*addr + 8 + (char_index as u32) * 4)?
                };
                decoded.push(char::from_u32(code_point).unwrap_or(char::REPLACEMENT_CHARACTER));
            }
            Ok(decoded)
        }
        _ => Ok(String::new()),
    }
}

fn native_str_from_int(vm: &mut VirtualMachine, args: &[Value]) -> Result<Value, RuntimeError> {
    let num = match args.first() {
        Some(Value::I64(val)) => *val,
        Some(Value::I32(val)) => i64::from(*val),
        _ => 0,
    };
    alloc_vm_string(vm, &num.to_string())
}

fn native_str_try_parse_int(
    vm: &mut VirtualMachine,
    args: &[Value],
) -> Result<Value, RuntimeError> {
    let raw = args
        .first()
        .map(|val| read_vm_string(vm, val))
        .transpose()?
        .unwrap_or_default();
    let trimmed = raw.trim();
    if let Ok(parsed) = trimmed.parse::<i64>() {
        if let Some(Value::Ref(out_addr)) = args.get(1)
            && *out_addr > 0
        {
            vm.heap.write_u64(*out_addr + 8, parsed as u64)?;
        }
        return Ok(Value::I32(1));
    }
    Ok(Value::I32(0))
}

fn native_str_to_lower(vm: &mut VirtualMachine, args: &[Value]) -> Result<Value, RuntimeError> {
    let raw = args
        .first()
        .map(|val| read_vm_string(vm, val))
        .transpose()?
        .unwrap_or_default();
    alloc_vm_string(vm, &raw.to_lowercase())
}

fn native_str_to_upper(vm: &mut VirtualMachine, args: &[Value]) -> Result<Value, RuntimeError> {
    let raw = args
        .first()
        .map(|val| read_vm_string(vm, val))
        .transpose()?
        .unwrap_or_default();
    alloc_vm_string(vm, &raw.to_uppercase())
}

fn native_str_index_of(vm: &mut VirtualMachine, args: &[Value]) -> Result<Value, RuntimeError> {
    let haystack = args
        .first()
        .map(|val| read_vm_string(vm, val))
        .transpose()?
        .unwrap_or_default();
    let needle = args
        .get(1)
        .map(|val| read_vm_string(vm, val))
        .transpose()?
        .unwrap_or_default();
    let start_char = match args.get(2) {
        Some(Value::I32(idx)) if *idx > 0 => *idx as usize,
        _ => 0,
    };
    let byte_start = haystack
        .char_indices()
        .nth(start_char)
        .map(|(byte_off, _)| byte_off)
        .unwrap_or(haystack.len());
    let result = if byte_start > haystack.len() {
        -1
    } else if let Some(rel_pos) = haystack[byte_start..].find(&needle) {
        let found_chars = haystack[..byte_start + rel_pos].chars().count();
        found_chars as i64
    } else {
        -1
    };
    Ok(Value::I64(result))
}

fn native_str_chars_to_string(
    vm: &mut VirtualMachine,
    args: &[Value],
) -> Result<Value, RuntimeError> {
    let decoded = args
        .first()
        .map(|val| read_vm_char_array_as_string(vm, val))
        .transpose()?
        .unwrap_or_default();
    alloc_vm_string(vm, &decoded)
}

fn native_str_to_chars(vm: &mut VirtualMachine, args: &[Value]) -> Result<Value, RuntimeError> {
    let text = args
        .first()
        .map(|val| read_vm_string(vm, val))
        .transpose()?
        .unwrap_or_default();
    alloc_vm_char_array(vm, &text)
}

fn native_str_to_utf8_bytes(
    vm: &mut VirtualMachine,
    args: &[Value],
) -> Result<Value, RuntimeError> {
    let text = args
        .first()
        .map(|val| read_vm_string(vm, val))
        .transpose()?
        .unwrap_or_default();
    alloc_vm_byte_array(vm, text.as_bytes())
}

fn native_str_from_utf8_bytes(
    vm: &mut VirtualMachine,
    args: &[Value],
) -> Result<Value, RuntimeError> {
    let bytes = args
        .first()
        .map(|val| read_vm_byte_array(vm, val))
        .transpose()?
        .unwrap_or_default();
    let decoded = String::from_utf8_lossy(&bytes).into_owned();
    alloc_vm_string(vm, &decoded)
}

fn native_tcp_connect(vm: &mut VirtualMachine, args: &[Value]) -> Result<Value, RuntimeError> {
    let endpoint = args
        .first()
        .map(|val| read_vm_char_array_as_string(vm, val))
        .transpose()?
        .unwrap_or_default();
    let stream =
        TcpStream::connect(&endpoint).map_err(|err| RuntimeError::NativeFunctionError {
            name: "TcpStream::native_connect".to_string(),
            reason: format!("[E0333] connect failed for '{endpoint}': {err}"),
        })?;
    let mut guard = TEST_SOCKETS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let next_id = guard.0;
    guard.0 += 1;
    guard.1.insert(next_id, stream);
    Ok(Value::I64(next_id))
}

fn native_tcp_write(vm: &mut VirtualMachine, args: &[Value]) -> Result<Value, RuntimeError> {
    let socket_id = match args.first() {
        Some(Value::I64(id)) => *id,
        _ => return Ok(Value::I32(0)),
    };
    let bytes = args
        .get(1)
        .map(|val| read_vm_byte_array(vm, val))
        .transpose()?
        .unwrap_or_default();
    let mut guard = TEST_SOCKETS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(stream) = guard.1.get_mut(&socket_id) else {
        return Ok(Value::I32(-1));
    };
    let written = stream
        .write(&bytes)
        .map_err(|err| RuntimeError::NativeFunctionError {
            name: "TcpStream::native_write".to_string(),
            reason: format!("[E0333] write failed: {err}"),
        })?;
    Ok(Value::I32(written as i32))
}

fn native_tcp_read(vm: &mut VirtualMachine, args: &[Value]) -> Result<Value, RuntimeError> {
    let socket_id = match args.first() {
        Some(Value::I64(id)) => *id,
        _ => return alloc_vm_byte_array(vm, &[]),
    };
    let max_bytes = match args.get(1) {
        Some(Value::I32(count)) if *count > 0 => *count as usize,
        _ => 1,
    };
    let mut guard = TEST_SOCKETS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(stream) = guard.1.get_mut(&socket_id) else {
        return alloc_vm_byte_array(vm, &[]);
    };
    let mut buf = vec![0u8; max_bytes];
    let bytes_read = stream
        .read(&mut buf)
        .map_err(|err| RuntimeError::NativeFunctionError {
            name: "TcpStream::native_read".to_string(),
            reason: format!("[E0333] read failed: {err}"),
        })?;
    buf.truncate(bytes_read);
    alloc_vm_byte_array(vm, &buf)
}

fn native_tcp_close(_vm: &mut VirtualMachine, args: &[Value]) -> Result<Value, RuntimeError> {
    let socket_id = match args.first() {
        Some(Value::I64(id)) => *id,
        _ => return Ok(Value::Bool(false)),
    };
    let mut guard = TEST_SOCKETS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let removed = guard.1.remove(&socket_id).is_some();
    Ok(Value::Bool(removed))
}

fn native_prim_nan(_vm: &mut VirtualMachine, _args: &[Value]) -> Result<Value, RuntimeError> {
    Ok(Value::F64(f64::NAN))
}

fn native_prim_pos_inf(_vm: &mut VirtualMachine, _args: &[Value]) -> Result<Value, RuntimeError> {
    Ok(Value::F64(f64::INFINITY))
}

fn native_prim_neg_inf(_vm: &mut VirtualMachine, _args: &[Value]) -> Result<Value, RuntimeError> {
    Ok(Value::F64(f64::NEG_INFINITY))
}

fn native_prim_min_double(
    _vm: &mut VirtualMachine,
    _args: &[Value],
) -> Result<Value, RuntimeError> {
    Ok(Value::F64(f64::MIN_POSITIVE))
}

fn native_prim_max_double(
    _vm: &mut VirtualMachine,
    _args: &[Value],
) -> Result<Value, RuntimeError> {
    Ok(Value::F64(f64::MAX))
}

fn native_prim_hash_any(_vm: &mut VirtualMachine, args: &[Value]) -> Result<Value, RuntimeError> {
    let raw = match args.first() {
        Some(Value::I64(val)) => *val,
        Some(Value::I32(val)) => i64::from(*val),
        _ => 0,
    };
    let folded = (raw ^ (raw >> 32)) as i32;
    Ok(Value::I32(folded))
}

fn register_stdlib_native_shims(options: RuntimeOptions) -> RuntimeOptions {
    options
        .with_native_function("Primitives::native_nan", native_prim_nan)
        .with_native_function("Primitives::native_pos_inf", native_prim_pos_inf)
        .with_native_function("Primitives::native_neg_inf", native_prim_neg_inf)
        .with_native_function("Primitives::native_min_double", native_prim_min_double)
        .with_native_function("Primitives::native_max_double", native_prim_max_double)
        .with_native_function("Primitives::native_hash_any", native_prim_hash_any)
        .with_native_function("String::native_from_int", native_str_from_int)
        .with_native_function("String::native_try_parse_int", native_str_try_parse_int)
        .with_native_function("String::native_to_lower", native_str_to_lower)
        .with_native_function("String::native_to_upper", native_str_to_upper)
        .with_native_function("String::native_index_of", native_str_index_of)
        .with_native_function("String::native_chars_to_string", native_str_chars_to_string)
        .with_native_function("String::native_string_to_chars", native_str_to_chars)
        .with_native_function("String::native_to_utf8_bytes", native_str_to_utf8_bytes)
        .with_native_function("String::native_from_utf8_bytes", native_str_from_utf8_bytes)
        .with_native_function("TcpStream::native_connect", native_tcp_connect)
        .with_native_function("TcpStream::native_write", native_tcp_write)
        .with_native_function("TcpStream::native_read", native_tcp_read)
        .with_native_function("TcpStream::native_close", native_tcp_close)
}

fn compile_with_stdlib_core_and_collections(extra_sources: &[(&str, &str)]) -> BinaryProgram {
    let core_interfaces = include_str!("../../../../libraries/solixlib/src/core/Interfaces.slx");
    let core_exceptions = include_str!("../../../../libraries/solixlib/src/core/Exceptions.slx");
    let core_optional = include_str!("../../../../libraries/solixlib/src/core/Optional.slx");
    let core_primitives = include_str!("../../../../libraries/solixlib/src/core/Primitives.slx");
    let core_string = include_str!("../../../../libraries/solixlib/src/core/String.slx");
    let core_builder = include_str!("../../../../libraries/solixlib/src/core/StringBuilder.slx");
    let col_interfaces =
        include_str!("../../../../libraries/solixlib/src/collections/Interfaces.slx");
    let col_kvp = include_str!("../../../../libraries/solixlib/src/collections/KeyValuePair.slx");
    let col_utils = include_str!("../../../../libraries/solixlib/src/collections/Collections.slx");
    let col_list = include_str!("../../../../libraries/solixlib/src/collections/List.slx");
    let col_algos = include_str!("../../../../libraries/solixlib/src/collections/Algorithms.slx");
    let col_linked = include_str!("../../../../libraries/solixlib/src/collections/LinkedList.slx");
    let col_treemap = include_str!("../../../../libraries/solixlib/src/collections/TreeMap.slx");
    let col_hashmap = include_str!("../../../../libraries/solixlib/src/collections/HashMap.slx");

    let mut options = CompilationOptions::new()
        .with_source("core/Interfaces.slx", core_interfaces)
        .with_source("core/Exceptions.slx", core_exceptions)
        .with_source("core/Optional.slx", core_optional)
        .with_source("core/Primitives.slx", core_primitives)
        .with_source("core/String.slx", core_string)
        .with_source("core/StringBuilder.slx", core_builder)
        .with_source("collections/Interfaces.slx", col_interfaces)
        .with_source("collections/KeyValuePair.slx", col_kvp)
        .with_source("collections/Collections.slx", col_utils)
        .with_source("collections/List.slx", col_list)
        .with_source("collections/Algorithms.slx", col_algos)
        .with_source("collections/LinkedList.slx", col_linked)
        .with_source("collections/TreeMap.slx", col_treemap)
        .with_source("collections/HashMap.slx", col_hashmap);

    for (name, source) in extra_sources {
        options = options.with_source(*name, *source);
    }

    let mut context = CompilationContext::new(options);
    context
        .execute()
        .expect("Failed to compile stdlib collections & runtime test program");
    context
        .emission_artifacts
        .expect("Emission artifacts must exist")
        .program
}

#[test]
fn test_stdlib_linked_list_and_treemap_weak_arc_clean_without_cycle_collector() {
    let main_source = r#"
import solix.core.String;
import solix.collections.LinkedList;
import solix.collections.TreeMap;

public class Main {
    private static void exercise_collections() {
        LinkedList<int32> list = new LinkedList<int32>();
        for (int32 idx = 0; idx < 25; idx = idx + 1) {
            list.add(idx);
        }
        list.insert(5, 999);
        list.remove_first();
        list.remove_last();
        list.remove(10);
        list.reverse();
        if (list.size() != 23) {
            return;
        }
        list.clear();

        LinkedList<int32> scope_drop = new LinkedList<int32>();
        scope_drop.add(1);
        scope_drop.add(2);
        scope_drop.add(3);

        TreeMap<int32, int32> tree = new TreeMap<int32, int32>();
        for (int32 key_num = 0; key_num < 40; key_num = key_num + 1) {
            tree.put(key_num, key_num * 10);
        }
        for (int32 rem_num = 0; rem_num < 20; rem_num = rem_num + 1) {
            tree.remove(rem_num);
        }
        if (tree.size() != 20) {
            return;
        }
        // Explicitly allow scope exit without clear() as well to exercise destructor + weak links.
    }

    public static int32 main() {
        Main::exercise_collections();
        return 0;
    }
}
"#;

    let program = compile_with_stdlib_core_and_collections(&[("main.slx", main_source)]);
    let entry_symbol = program
        .functions
        .iter()
        .find(|func| func.symbol.ends_with("::main"))
        .map(|func| func.symbol.clone())
        .expect("Main::main entry present");

    let mut options = RuntimeOptions::new()
        .with_binary(program)
        .with_entry_point(&entry_symbol)
        .with_cycle_collector(false);
    options = register_stdlib_native_shims(options);

    let context = RuntimeContext::new(options).expect("RuntimeContext init failed");
    let mut vm = VirtualMachine::new(context);
    let exit_val = vm.run().expect("VM run failed");
    assert_eq!(exit_val, Value::I32(0));

    vm.verify_clean_memory()
        .expect("LinkedList weak previous_node and TreeMap weak parent_node must leave zero leaked cycles even with cycle collector disabled");
}

#[test]
fn test_overloaded_string_vs_boxed_null_args_and_super_ctor_runtime() {
    let source_code = r#"
public class BoxedText {
    public string raw_text;
    public BoxedText(string initial) {
        this.raw_text = initial;
    }
    public int32 len() {
        return this.raw_text.length;
    }
}

public class PairMap<KeyType, ValueType> {
    public int32 tag;
    public PairMap(int32 initial_tag) {
        this.tag = initial_tag;
    }
}

public class BaseCustomError {
    public string detail;
    public BaseCustomError() {
        this.detail = "empty";
    }
    public BaseCustomError(string message) {
        this.detail = message;
    }
}

public class DerivedCustomError extends BaseCustomError {
    public DerivedCustomError() : super("default") {}
    public DerivedCustomError(string message) : super(message) {}
}

public class OverloadDispatch {
    public int32 measure(string raw) {
        return this.measure(new BoxedText(raw));
    }
    public int32 measure(BoxedText boxed) {
        return (boxed != null) ? boxed.len() : -1;
    }

    public static int32 route(string verb, string path) {
        return OverloadDispatch::route(new BoxedText(verb), new BoxedText(path), null, null);
    }
    public static int32 route(
        BoxedText verb,
        BoxedText path,
        BoxedText body,
        PairMap<BoxedText, BoxedText> headers
    ) {
        int32 acc = verb.len() + path.len();
        if (body != null) {
            acc = acc + body.len();
        }
        if (headers != null) {
            acc = acc + headers.tag;
        }
        return acc;
    }
}

public class Main {
    public static int32 main() {
        OverloadDispatch dispatcher = new OverloadDispatch();
        int32 m1 = dispatcher.measure("solix");
        int32 m2 = dispatcher.measure(new BoxedText("lang"));
        int32 r1 = OverloadDispatch::route("GET", "/status");
        int32 r2 = OverloadDispatch::route(
            new BoxedText("POST"),
            new BoxedText("/v1"),
            new BoxedText("payload"),
            new PairMap<BoxedText, BoxedText>(5)
        );
        DerivedCustomError derived = new DerivedCustomError("custom-msg");
        if (m1 != 5 || m2 != 4 || r1 != 10 || r2 != 19) {
            return 1;
        }
        if (derived.detail != "custom-msg") {
            return 2;
        }
        return 77;
    }
}
"#;

    let (result, _vm) = compile_and_run(source_code);
    assert_eq!(result, Value::I32(77));
}

#[test]
fn test_httpclient_real_multi_chunk_and_chunked_encoding_e2e() {
    use std::net::TcpListener;
    use std::sync::mpsc;

    let server = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral HTTP test server");
    let server_port = server.local_addr().expect("server local addr").port();

    // Generate >160 KiB body payload (170,000 ASCII bytes).
    let large_body: String = (0..170_000)
        .map(|byte_idx| (b'a' + (byte_idx % 26) as u8) as char)
        .collect();
    let expected_large_len = large_body.len();
    let large_body_clone = large_body.clone();

    let (ready_tx, ready_rx) = mpsc::channel();
    let server_handle = std::thread::spawn(move || {
        ready_tx.send(()).expect("signal server ready");
        for request_idx in 0..3 {
            let (mut conn, _) = server.accept().expect("accept HTTP client");
            let mut req_bytes = Vec::new();
            let mut chunk_buf = [0u8; 4096];
            loop {
                let nread = conn.read(&mut chunk_buf).unwrap_or(0);
                if nread == 0 {
                    break;
                }
                req_bytes.extend_from_slice(&chunk_buf[..nread]);
                if let Some(hdr_end) = req_bytes.windows(4).position(|win| win == b"\r\n\r\n") {
                    let hdr_str = String::from_utf8_lossy(&req_bytes[..hdr_end]);
                    let content_len = hdr_str
                        .lines()
                        .find_map(|line| {
                            let lower = line.to_ascii_lowercase();
                            lower
                                .strip_prefix("content-length:")
                                .and_then(|val| val.trim().parse::<usize>().ok())
                        })
                        .unwrap_or(0);
                    if req_bytes.len() >= hdr_end + 4 + content_len {
                        break;
                    }
                }
            }
            let request_text = String::from_utf8_lossy(&req_bytes).into_owned();

            match request_idx {
                0 => {
                    assert!(
                        request_text.starts_with("GET /large HTTP/1.1"),
                        "unexpected request 0: {request_text}"
                    );
                    let headers = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nX-Solix-Test: LargeChunked\r\nConnection: close\r\n\r\n",
                        large_body_clone.len()
                    );
                    conn.write_all(headers.as_bytes()).expect("write headers 0");
                    for piece in large_body_clone.as_bytes().chunks(8192) {
                        conn.write_all(piece).expect("write body piece 0");
                    }
                }
                1 => {
                    assert!(
                        request_text.starts_with("GET /chunked HTTP/1.1"),
                        "unexpected request 1: {request_text}"
                    );
                    let response_bytes = concat!(
                        "HTTP/1.1 200 OK\r\n",
                        "Transfer-Encoding: chunked\r\n",
                        "X-Transfer: Streaming\r\n",
                        "Connection: close\r\n\r\n",
                        "5\r\nHello\r\n",
                        "2\r\n, \r\n",
                        "D\r\nSolix HTTP/1!\r\n",
                        "0\r\n\r\n"
                    );
                    conn.write_all(response_bytes.as_bytes())
                        .expect("write chunked response");
                }
                _ => {
                    assert!(
                        request_text.starts_with("POST /echo HTTP/1.1"),
                        "unexpected request 2: {request_text}"
                    );
                    let body_marker = "payload=café";
                    assert!(
                        request_text.contains(body_marker),
                        "POST body missing marker: {request_text}"
                    );
                    let reply_body = "created:ok";
                    let response = format!(
                        "HTTP/1.1 201 Created\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        reply_body.len(),
                        reply_body
                    );
                    conn.write_all(response.as_bytes())
                        .expect("write post response");
                }
            }
            conn.flush().expect("flush server conn");
        }
    });

    ready_rx.recv().expect("wait for server ready");

    let net_exception = include_str!("../../../../libraries/solixlib/src/net/NetworkException.slx");
    let tcp_stream = include_str!("../../../../libraries/solixlib/src/net/TcpStream.slx");
    let http_client = include_str!("../../../../libraries/solixlib/src/net/HttpClient.slx");
    let test_main = format!(
        r#"
import solix.core.String;
import solix.net.HttpClient;
import solix.net.HttpResponse;

public class Main {{
    public static int32 main() {{
        HttpResponse large_resp = HttpClient::get_response("http://127.0.0.1:{server_port}/large");
        if (!large_resp.is_ok() || large_resp.status_code != 200) {{
            return 1;
        }}
        if (large_resp.body.length() != {expected_large_len}) {{
            return 2;
        }}
        String header_val = large_resp.get_header("X-Solix-Test");
        if (header_val == null || !header_val.equals(new String("LargeChunked"))) {{
            return 3;
        }}

        HttpResponse chunked_resp = HttpClient::get_response("http://127.0.0.1:{server_port}/chunked");
        if (!chunked_resp.is_ok() || !chunked_resp.body.equals(new String("Hello, Solix HTTP/1!"))) {{
            return 4;
        }}

        HttpResponse post_resp = HttpClient::post("http://127.0.0.1:{server_port}/echo", "payload=café");
        if (post_resp.status_code != 201 || !post_resp.body.equals(new String("created:ok"))) {{
            return 5;
        }}

        return 0;
    }}
}}
"#
    );

    let program = compile_with_stdlib_core_and_collections(&[
        ("net/NetworkException.slx", net_exception),
        ("net/TcpStream.slx", tcp_stream),
        ("net/HttpClient.slx", http_client),
        ("main.slx", &test_main),
    ]);

    let entry_symbol = program
        .functions
        .iter()
        .find(|func| func.symbol.ends_with("::main"))
        .map(|func| func.symbol.clone())
        .expect("Main::main entry present");

    let mut options = RuntimeOptions::new()
        .with_binary(program)
        .with_entry_point(&entry_symbol)
        .with_cycle_collector(false);
    options = register_stdlib_native_shims(options);

    let context = RuntimeContext::new(options).expect("RuntimeContext init failed");
    let mut vm = VirtualMachine::new(context);
    let result = vm.run().expect("HttpClient E2E VM execution failed");
    assert_eq!(result, Value::I32(0));
    server_handle.join().expect("join HTTP test server");

    let (_strings, arrays, objects, closures) = vm.heap.count_blocks_by_tag();
    assert_eq!(
        (arrays, objects, closures),
        (0, 0, 0),
        "HttpClient + TcpStream RAII must leave zero leaked objects, arrays, or closures"
    );
}
