//! Cross-platform networking native bridge using std::net.

use crate::{
    lock_poison_safe, read_solix_bytes, read_solix_chars, register_symbol_variants,
    set_native_error, NativeCFunctionPtr, RegisterNativeCallback, SolixCValue,
};
use std::collections::HashMap;
use std::ffi::c_void;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs, UdpSocket};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

enum SocketHandle {
    TcpListener(TcpListener),
    TcpStream(TcpStream),
    UdpSocket(UdpSocket),
}

enum ListenerLookup {
    Ready(TcpListener),
    CloneFailed(std::io::Error),
    Invalid,
}

static NEXT_SOCKET_ID: LazyLock<Mutex<i64>> = LazyLock::new(|| Mutex::new(1));
static SOCKET_MAP: LazyLock<Mutex<HashMap<i64, SocketHandle>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn allocate_socket_id() -> i64 {
    let mut lock = lock_poison_safe(&NEXT_SOCKET_ID);
    let identifier = *lock;
    *lock += 1;
    identifier
}

fn insert_socket(handle: SocketHandle) -> i64 {
    let socket_id = allocate_socket_id();
    lock_poison_safe(&SOCKET_MAP).insert(socket_id, handle);
    socket_id
}

fn signal_net_error(vm_context: *mut c_void, out_error: *mut SolixCValue, detail: &str) {
    let error_message = format!("[E0333] Network operation failed on socket: {detail}");
    unsafe { set_native_error(vm_context, out_error, &error_message) };
}

fn apply_socket_timeout(
    vm_context: *mut c_void,
    socket_id: i64,
    timeout_millis: i32,
    is_read: bool,
    out_error: *mut SolixCValue,
) -> bool {
    let duration_opt = if timeout_millis <= 0 {
        None
    } else {
        Some(Duration::from_millis(timeout_millis as u64))
    };
    let map = lock_poison_safe(&SOCKET_MAP);
    let result = match map.get(&socket_id) {
        Some(SocketHandle::TcpStream(stream)) => {
            if is_read {
                stream.set_read_timeout(duration_opt)
            } else {
                stream.set_write_timeout(duration_opt)
            }
        }
        Some(SocketHandle::UdpSocket(socket)) => {
            if is_read {
                socket.set_read_timeout(duration_opt)
            } else {
                socket.set_write_timeout(duration_opt)
            }
        }
        _ => {
            let kind_label = if is_read { "read" } else { "write" };
            signal_net_error(
                vm_context,
                out_error,
                &format!("cannot set {kind_label} timeout on invalid socket ID {socket_id}"),
            );
            return false;
        }
    };
    match result {
        Ok(()) => true,
        Err(timeout_err) => {
            let kind_label = if is_read { "read" } else { "write" };
            signal_net_error(
                vm_context,
                out_error,
                &format!("set_{kind_label}_timeout failed: {timeout_err}"),
            );
            false
        }
    }
}

pub unsafe extern "C" fn tcp_listener_native_bind(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i64_val(0);
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let host_port = unsafe { read_solix_chars(vm_context, address) };

    match TcpListener::bind(&host_port) {
        Ok(listener) => SolixCValue::i64_val(insert_socket(SocketHandle::TcpListener(listener))),
        Err(bind_error) => {
            signal_net_error(
                vm_context,
                out_error,
                &format!("bind to '{host_port}' failed: {bind_error}"),
            );
            SolixCValue::i64_val(0)
        }
    }
}

pub unsafe extern "C" fn tcp_listener_native_accept(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i64_val(0);
    }
    let socket_id = unsafe { (*args.add(0)).as_i64() };

    let lookup_result = {
        let map = lock_poison_safe(&SOCKET_MAP);
        match map.get(&socket_id) {
            Some(SocketHandle::TcpListener(listener)) => match listener.try_clone() {
                Ok(cloned) => ListenerLookup::Ready(cloned),
                Err(clone_error) => ListenerLookup::CloneFailed(clone_error),
            },
            _ => ListenerLookup::Invalid,
        }
    };

    match lookup_result {
        ListenerLookup::Ready(listener) => match listener.accept() {
            Ok((stream, _peer_addr)) => {
                SolixCValue::i64_val(insert_socket(SocketHandle::TcpStream(stream)))
            }
            Err(accept_error) => {
                signal_net_error(
                    vm_context,
                    out_error,
                    &format!("accept failed: {accept_error}"),
                );
                SolixCValue::i64_val(0)
            }
        },
        ListenerLookup::CloneFailed(clone_error) => {
            signal_net_error(
                vm_context,
                out_error,
                &format!("failed to clone listener socket ID {socket_id}: {clone_error}"),
            );
            SolixCValue::i64_val(0)
        }
        ListenerLookup::Invalid => {
            signal_net_error(
                vm_context,
                out_error,
                &format!("invalid or closed listener socket ID {socket_id}"),
            );
            SolixCValue::i64_val(0)
        }
    }
}

pub unsafe extern "C" fn tcp_stream_native_connect(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i64_val(0);
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let host_port = unsafe { read_solix_chars(vm_context, address) };

    match TcpStream::connect(&host_port) {
        Ok(stream) => SolixCValue::i64_val(insert_socket(SocketHandle::TcpStream(stream))),
        Err(connect_error) => {
            signal_net_error(
                vm_context,
                out_error,
                &format!("connect to '{host_port}' failed: {connect_error}"),
            );
            SolixCValue::i64_val(0)
        }
    }
}

pub unsafe extern "C" fn tcp_stream_native_connect_timeout(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::i64_val(0);
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let timeout_millis = unsafe { (*args.add(1)).as_i32() };
    let host_port = unsafe { read_solix_chars(vm_context, address) };

    if timeout_millis <= 0 {
        match TcpStream::connect(&host_port) {
            Ok(stream) => {
                return SolixCValue::i64_val(insert_socket(SocketHandle::TcpStream(stream)))
            }
            Err(connect_error) => {
                signal_net_error(
                    vm_context,
                    out_error,
                    &format!("connect to '{host_port}' failed: {connect_error}"),
                );
                return SolixCValue::i64_val(0);
            }
        }
    }

    let timeout_duration = Duration::from_millis(timeout_millis as u64);
    let resolved_addrs = match host_port.to_socket_addrs() {
        Ok(addrs) => addrs,
        Err(resolve_error) => {
            signal_net_error(
                vm_context,
                out_error,
                &format!("failed to resolve '{host_port}': {resolve_error}"),
            );
            return SolixCValue::i64_val(0);
        }
    };

    let mut last_error: Option<std::io::Error> = None;
    for resolved_addr in resolved_addrs {
        match TcpStream::connect_timeout(&resolved_addr, timeout_duration) {
            Ok(stream) => {
                return SolixCValue::i64_val(insert_socket(SocketHandle::TcpStream(stream)));
            }
            Err(connect_error) => {
                last_error = Some(connect_error);
            }
        }
    }
    let reason = last_error.map_or_else(
        || format!("no socket addresses resolved for '{host_port}'"),
        |err| format!("connect_timeout to '{host_port}' ({timeout_millis} ms) failed: {err}"),
    );
    signal_net_error(vm_context, out_error, &reason);
    SolixCValue::i64_val(0)
}

pub unsafe extern "C" fn socket_native_set_read_timeout(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::bool_val(false);
    }
    let socket_id = unsafe { (*args.add(0)).as_i64() };
    let timeout_millis = unsafe { (*args.add(1)).as_i32() };
    SolixCValue::bool_val(apply_socket_timeout(
        vm_context,
        socket_id,
        timeout_millis,
        true,
        out_error,
    ))
}

pub unsafe extern "C" fn socket_native_set_write_timeout(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::bool_val(false);
    }
    let socket_id = unsafe { (*args.add(0)).as_i64() };
    let timeout_millis = unsafe { (*args.add(1)).as_i32() };
    SolixCValue::bool_val(apply_socket_timeout(
        vm_context,
        socket_id,
        timeout_millis,
        false,
        out_error,
    ))
}

pub unsafe extern "C" fn tcp_stream_native_write(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::i32_val(0);
    }
    let socket_id = unsafe { (*args.add(0)).as_i64() };
    let buffer_address = unsafe { (*args.add(1)).as_ref() };
    let write_bytes = unsafe { read_solix_bytes(vm_context, buffer_address) };

    let mut map = lock_poison_safe(&SOCKET_MAP);
    if let Some(SocketHandle::TcpStream(stream)) = map.get_mut(&socket_id) {
        match stream.write(&write_bytes) {
            Ok(bytes_written) => SolixCValue::i32_val(bytes_written as i32),
            Err(write_error) => {
                signal_net_error(
                    vm_context,
                    out_error,
                    &format!("write failed: {write_error}"),
                );
                SolixCValue::i32_val(-1)
            }
        }
    } else {
        signal_net_error(
            vm_context,
            out_error,
            &format!("invalid or closed stream socket ID {socket_id}"),
        );
        SolixCValue::i32_val(-1)
    }
}

pub unsafe extern "C" fn tcp_stream_native_read(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::null();
    }
    let socket_id = unsafe { (*args.add(0)).as_i64() };
    let max_bytes = unsafe { (*args.add(1)).as_i32() }.max(1) as usize;

    let mut map = lock_poison_safe(&SOCKET_MAP);
    if let Some(SocketHandle::TcpStream(stream)) = map.get_mut(&socket_id) {
        let mut buffer = vec![0u8; max_bytes];
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                buffer.truncate(bytes_read);
                let allocated_address = unsafe { crate::alloc_solix_bytes(vm_context, &buffer) };
                SolixCValue::ref_val(allocated_address)
            }
            Err(read_error) => {
                signal_net_error(vm_context, out_error, &format!("read failed: {read_error}"));
                SolixCValue::null()
            }
        }
    } else {
        signal_net_error(
            vm_context,
            out_error,
            &format!("invalid or closed stream socket ID {socket_id}"),
        );
        SolixCValue::null()
    }
}

pub unsafe extern "C" fn socket_native_close(
    _vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::bool_val(false);
    }
    let socket_id = unsafe { (*args.add(0)).as_i64() };
    let removed = lock_poison_safe(&SOCKET_MAP).remove(&socket_id).is_some();
    SolixCValue::bool_val(removed)
}

pub unsafe extern "C" fn udp_socket_native_bind(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::i64_val(0);
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let host_port = unsafe { read_solix_chars(vm_context, address) };

    match UdpSocket::bind(&host_port) {
        Ok(socket) => SolixCValue::i64_val(insert_socket(SocketHandle::UdpSocket(socket))),
        Err(bind_error) => {
            signal_net_error(
                vm_context,
                out_error,
                &format!("UDP bind to '{host_port}' failed: {bind_error}"),
            );
            SolixCValue::i64_val(0)
        }
    }
}

pub unsafe extern "C" fn udp_socket_native_send_to(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 3 || args.is_null() {
        return SolixCValue::i32_val(0);
    }
    let socket_id = unsafe { (*args.add(0)).as_i64() };
    let buffer_address = unsafe { (*args.add(1)).as_ref() };
    let target_address = unsafe { (*args.add(2)).as_ref() };
    let write_bytes = unsafe { read_solix_bytes(vm_context, buffer_address) };
    let target_str = unsafe { read_solix_chars(vm_context, target_address) };

    let map = lock_poison_safe(&SOCKET_MAP);
    if let Some(SocketHandle::UdpSocket(socket)) = map.get(&socket_id) {
        match socket.send_to(&write_bytes, &target_str) {
            Ok(bytes_sent) => SolixCValue::i32_val(bytes_sent as i32),
            Err(send_error) => {
                signal_net_error(
                    vm_context,
                    out_error,
                    &format!("UDP send_to '{target_str}' failed: {send_error}"),
                );
                SolixCValue::i32_val(-1)
            }
        }
    } else {
        signal_net_error(
            vm_context,
            out_error,
            &format!("invalid or closed UDP socket ID {socket_id}"),
        );
        SolixCValue::i32_val(-1)
    }
}

pub unsafe extern "C" fn udp_socket_native_recv_from(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count < 2 || args.is_null() {
        return SolixCValue::null();
    }
    let socket_id = unsafe { (*args.add(0)).as_i64() };
    let max_bytes = unsafe { (*args.add(1)).as_i32() }.max(1) as usize;

    let map = lock_poison_safe(&SOCKET_MAP);
    if let Some(SocketHandle::UdpSocket(socket)) = map.get(&socket_id) {
        let mut buffer = vec![0u8; max_bytes];
        match socket.recv_from(&mut buffer) {
            Ok((bytes_read, _peer_addr)) => {
                buffer.truncate(bytes_read);
                let allocated_address = unsafe { crate::alloc_solix_bytes(vm_context, &buffer) };
                SolixCValue::ref_val(allocated_address)
            }
            Err(recv_error) => {
                signal_net_error(
                    vm_context,
                    out_error,
                    &format!("UDP recv_from failed: {recv_error}"),
                );
                SolixCValue::null()
            }
        }
    } else {
        signal_net_error(
            vm_context,
            out_error,
            &format!("invalid or closed UDP socket ID {socket_id}"),
        );
        SolixCValue::null()
    }
}

pub unsafe fn register_net_natives(collector: *mut c_void, callback: RegisterNativeCallback) {
    let mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "TcpListener_native_bind",
                "solix_net_TcpListener_native_bind",
                "TcpListener::native_bind",
            ],
            tcp_listener_native_bind,
        ),
        (
            &[
                "TcpListener_native_accept",
                "solix_net_TcpListener_native_accept",
                "TcpListener::native_accept",
            ],
            tcp_listener_native_accept,
        ),
        (
            &[
                "TcpStream_native_connect",
                "solix_net_TcpStream_native_connect",
                "TcpStream::native_connect",
            ],
            tcp_stream_native_connect,
        ),
        (
            &[
                "TcpStream_native_connect_timeout",
                "solix_net_TcpStream_native_connect_timeout",
                "TcpStream::native_connect_timeout",
            ],
            tcp_stream_native_connect_timeout,
        ),
        (
            &[
                "TcpStream_native_write",
                "solix_net_TcpStream_native_write",
                "TcpStream::native_write",
            ],
            tcp_stream_native_write,
        ),
        (
            &[
                "TcpStream_native_read",
                "solix_net_TcpStream_native_read",
                "TcpStream::native_read",
            ],
            tcp_stream_native_read,
        ),
        (
            &[
                "TcpStream_native_set_read_timeout",
                "solix_net_TcpStream_native_set_read_timeout",
                "TcpStream::native_set_read_timeout",
                "UdpSocket_native_set_read_timeout",
                "solix_net_UdpSocket_native_set_read_timeout",
                "UdpSocket::native_set_read_timeout",
                "Socket_native_set_read_timeout",
                "solix_net_Socket_native_set_read_timeout",
                "Socket::native_set_read_timeout",
            ],
            socket_native_set_read_timeout,
        ),
        (
            &[
                "TcpStream_native_set_write_timeout",
                "solix_net_TcpStream_native_set_write_timeout",
                "TcpStream::native_set_write_timeout",
                "UdpSocket_native_set_write_timeout",
                "solix_net_UdpSocket_native_set_write_timeout",
                "UdpSocket::native_set_write_timeout",
                "Socket_native_set_write_timeout",
                "solix_net_Socket_native_set_write_timeout",
                "Socket::native_set_write_timeout",
            ],
            socket_native_set_write_timeout,
        ),
        (
            &[
                "UdpSocket_native_bind",
                "solix_net_UdpSocket_native_bind",
                "UdpSocket::native_bind",
            ],
            udp_socket_native_bind,
        ),
        (
            &[
                "UdpSocket_native_send_to",
                "solix_net_UdpSocket_native_send_to",
                "UdpSocket::native_send_to",
            ],
            udp_socket_native_send_to,
        ),
        (
            &[
                "UdpSocket_native_recv_from",
                "solix_net_UdpSocket_native_recv_from",
                "UdpSocket::native_recv_from",
            ],
            udp_socket_native_recv_from,
        ),
        (
            &[
                "Socket_native_close",
                "solix_net_Socket_native_close",
                "Socket::native_close",
            ],
            socket_native_close,
        ),
    ];

    for (variants, func) in mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }
}
