//! Cross-platform networking native bridge using std::net.

use crate::{
    alloc_solix_chars, read_solix_bytes, read_solix_chars, register_symbol_variants,
    NativeCFunctionPtr, RegisterNativeCallback, SolixCValue,
};
use std::collections::HashMap;
use std::ffi::c_void;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::{LazyLock, Mutex};

enum SocketHandle {
    TcpListener(TcpListener),
    TcpStream(TcpStream),
    UdpSocket(UdpSocket),
}

static NEXT_SOCKET_ID: LazyLock<Mutex<i64>> = LazyLock::new(|| Mutex::new(1));
static SOCKET_MAP: LazyLock<Mutex<HashMap<i64, SocketHandle>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn allocate_socket_id() -> i64 {
    let mut lock = NEXT_SOCKET_ID.lock().unwrap();
    let identifier = *lock;
    *lock += 1;
    identifier
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
        Ok(listener) => {
            let socket_id = allocate_socket_id();
            SOCKET_MAP
                .lock()
                .unwrap()
                .insert(socket_id, SocketHandle::TcpListener(listener));
            SolixCValue::i64_val(socket_id)
        }
        Err(bind_error) => {
            if !out_error.is_null() {
                let error_message = format!(
                    "[E0333] Network operation failed on socket: bind to '{}' failed: {}",
                    host_port, bind_error
                );
                let allocated_error = unsafe { alloc_solix_chars(vm_context, &error_message) };
                unsafe {
                    *out_error = SolixCValue::ref_val(allocated_error);
                }
            }
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

    let listener_clone = {
        let map = SOCKET_MAP.lock().unwrap();
        match map.get(&socket_id) {
            Some(SocketHandle::TcpListener(listener)) => listener.try_clone().ok(),
            _ => None,
        }
    };

    if let Some(listener) = listener_clone {
        match listener.accept() {
            Ok((stream, _peer_addr)) => {
                let stream_socket_id = allocate_socket_id();
                SOCKET_MAP
                    .lock()
                    .unwrap()
                    .insert(stream_socket_id, SocketHandle::TcpStream(stream));
                SolixCValue::i64_val(stream_socket_id)
            }
            Err(accept_error) => {
                if !out_error.is_null() {
                    let error_message = format!(
                        "[E0333] Network operation failed on socket: accept failed: {}",
                        accept_error
                    );
                    let allocated_error = unsafe { alloc_solix_chars(vm_context, &error_message) };
                    unsafe {
                        *out_error = SolixCValue::ref_val(allocated_error);
                    }
                }
                SolixCValue::i64_val(0)
            }
        }
    } else {
        if !out_error.is_null() {
            let error_message = format!(
                "[E0333] Network operation failed on socket: invalid or closed listener socket ID {socket_id}"
            );
            let allocated_error = unsafe { alloc_solix_chars(vm_context, &error_message) };
            unsafe {
                *out_error = SolixCValue::ref_val(allocated_error);
            }
        }
        SolixCValue::i64_val(0)
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
        Ok(stream) => {
            let socket_id = allocate_socket_id();
            SOCKET_MAP
                .lock()
                .unwrap()
                .insert(socket_id, SocketHandle::TcpStream(stream));
            SolixCValue::i64_val(socket_id)
        }
        Err(connect_error) => {
            if !out_error.is_null() {
                let error_message = format!(
                    "[E0333] Network operation failed on socket: connect to '{}' failed: {}",
                    host_port, connect_error
                );
                let allocated_error = unsafe { alloc_solix_chars(vm_context, &error_message) };
                unsafe {
                    *out_error = SolixCValue::ref_val(allocated_error);
                }
            }
            SolixCValue::i64_val(0)
        }
    }
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

    let mut map = SOCKET_MAP.lock().unwrap();
    if let Some(SocketHandle::TcpStream(stream)) = map.get_mut(&socket_id) {
        match stream.write(&write_bytes) {
            Ok(bytes_written) => SolixCValue::i32_val(bytes_written as i32),
            Err(write_error) => {
                if !out_error.is_null() {
                    let error_message = format!(
                        "[E0333] Network operation failed on socket: write failed: {}",
                        write_error
                    );
                    let allocated_error = unsafe { alloc_solix_chars(vm_context, &error_message) };
                    unsafe {
                        *out_error = SolixCValue::ref_val(allocated_error);
                    }
                }
                SolixCValue::i32_val(-1)
            }
        }
    } else {
        if !out_error.is_null() {
            let error_message = format!(
                "[E0333] Network operation failed on socket: invalid or closed stream socket ID {socket_id}"
            );
            let allocated_error = unsafe { alloc_solix_chars(vm_context, &error_message) };
            unsafe {
                *out_error = SolixCValue::ref_val(allocated_error);
            }
        }
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

    let mut map = SOCKET_MAP.lock().unwrap();
    if let Some(SocketHandle::TcpStream(stream)) = map.get_mut(&socket_id) {
        let mut buffer = vec![0u8; max_bytes];
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                buffer.truncate(bytes_read);
                let allocated_address = unsafe { crate::alloc_solix_bytes(vm_context, &buffer) };
                SolixCValue::ref_val(allocated_address)
            }
            Err(read_error) => {
                if !out_error.is_null() {
                    let error_message = format!(
                        "[E0333] Network operation failed on socket: read failed: {}",
                        read_error
                    );
                    let allocated_error = unsafe { alloc_solix_chars(vm_context, &error_message) };
                    unsafe {
                        *out_error = SolixCValue::ref_val(allocated_error);
                    }
                }
                SolixCValue::null()
            }
        }
    } else {
        if !out_error.is_null() {
            let error_message = format!(
                "[E0333] Network operation failed on socket: invalid or closed stream socket ID {socket_id}"
            );
            let allocated_error = unsafe { alloc_solix_chars(vm_context, &error_message) };
            unsafe {
                *out_error = SolixCValue::ref_val(allocated_error);
            }
        }
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
    let removed = SOCKET_MAP.lock().unwrap().remove(&socket_id).is_some();
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
        Ok(socket) => {
            let socket_id = allocate_socket_id();
            SOCKET_MAP
                .lock()
                .unwrap()
                .insert(socket_id, SocketHandle::UdpSocket(socket));
            SolixCValue::i64_val(socket_id)
        }
        Err(bind_error) => {
            if !out_error.is_null() {
                let error_message = format!(
                    "[E0333] Network operation failed on socket: UDP bind to '{}' failed: {}",
                    host_port, bind_error
                );
                let allocated_error = unsafe { alloc_solix_chars(vm_context, &error_message) };
                unsafe {
                    *out_error = SolixCValue::ref_val(allocated_error);
                }
            }
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

    let map = SOCKET_MAP.lock().unwrap();
    if let Some(SocketHandle::UdpSocket(socket)) = map.get(&socket_id) {
        match socket.send_to(&write_bytes, &target_str) {
            Ok(bytes_sent) => SolixCValue::i32_val(bytes_sent as i32),
            Err(send_error) => {
                if !out_error.is_null() {
                    let error_message = format!(
                        "[E0333] Network operation failed on socket: UDP send_to '{}' failed: {}",
                        target_str, send_error
                    );
                    let allocated_error = unsafe { alloc_solix_chars(vm_context, &error_message) };
                    unsafe {
                        *out_error = SolixCValue::ref_val(allocated_error);
                    }
                }
                SolixCValue::i32_val(-1)
            }
        }
    } else {
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

    let map = SOCKET_MAP.lock().unwrap();
    if let Some(SocketHandle::UdpSocket(socket)) = map.get(&socket_id) {
        let mut buffer = vec![0u8; max_bytes];
        match socket.recv_from(&mut buffer) {
            Ok((bytes_read, _peer_addr)) => {
                buffer.truncate(bytes_read);
                let allocated_address = unsafe { crate::alloc_solix_bytes(vm_context, &buffer) };
                SolixCValue::ref_val(allocated_address)
            }
            Err(recv_error) => {
                if !out_error.is_null() {
                    let error_message = format!(
                        "[E0333] Network operation failed on socket: UDP recv_from failed: {}",
                        recv_error
                    );
                    let allocated_error = unsafe { alloc_solix_chars(vm_context, &error_message) };
                    unsafe {
                        *out_error = SolixCValue::ref_val(allocated_error);
                    }
                }
                SolixCValue::null()
            }
        }
    } else {
        SolixCValue::null()
    }
}

pub unsafe fn register_net_natives(
    collector: *mut c_void,
    callback: RegisterNativeCallback,
) {
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
