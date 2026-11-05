//! Core language built-in functions: print, println, assert, clock.

#![forbid(unsafe_code)]

use crate::exceptions::RuntimeError;
use crate::native::registry::NativeRegistry;
use crate::value::Value;
use crate::vm::engine::VirtualMachine;
use std::time::{SystemTime, UNIX_EPOCH};

/// Registers all standard built-in functions into the target `NativeRegistry`.
pub fn register_all_builtins(registry: &mut NativeRegistry) {
    registry.register("print", builtin_print);
    registry.register("println", builtin_println);
    registry.register("blt::io::print", builtin_print);
    registry.register("blt::io::println", builtin_println);
    registry.register("blt::io::print_chars", builtin_print);
    registry.register("blt::io::println_chars", builtin_println);
    registry.register("Console::print", builtin_print);
    registry.register("Console::println", builtin_println);
    registry.register("assert", builtin_assert);
    registry.register("clock", builtin_clock);
    registry.register("now", builtin_clock);
    registry.register("__builtin_str_concat", builtin_str_concat);
}

fn format_vm_value(vm: &VirtualMachine, val: &Value) -> String {
    match val {
        Value::Ref(addr) if *addr > 0 => {
            if let Ok(header) = vm.heap.read_header(*addr)
                && header.tag == crate::memory::BlockTag::String
            {
                vm.heap
                    .read_string(*addr)
                    .unwrap_or_else(|_| format!("{val}"))
            } else {
                format!("{val}")
            }
        }
        _ => format!("{val}"),
    }
}

/// Standard print builtin: prints values without trailing newline.
pub fn builtin_print(vm: &mut VirtualMachine, args: &[Value]) -> Result<Value, RuntimeError> {
    let mut parts = Vec::new();
    for arg in args {
        parts.push(format_vm_value(vm, arg));
    }
    let text = parts.join(" ");
    print!("{text}");
    if let Some(last) = vm.stdout.last_mut() {
        last.push_str(&text);
    } else {
        vm.stdout.push(text);
    }
    Ok(Value::Null)
}

/// Standard println builtin: prints values followed by newline.
pub fn builtin_println(vm: &mut VirtualMachine, args: &[Value]) -> Result<Value, RuntimeError> {
    let mut parts = Vec::new();
    for arg in args {
        parts.push(format_vm_value(vm, arg));
    }
    let line = parts.join(" ");
    println!("{line}");
    vm.stdout.push(line);
    Ok(Value::Null)
}

/// Standard assert builtin: raises `AssertionFailed` if condition is falsy.
pub fn builtin_assert(vm: &mut VirtualMachine, args: &[Value]) -> Result<Value, RuntimeError> {
    let cond = args.first().is_some_and(Value::is_truthy);
    if !cond {
        let msg = if args.len() > 1 {
            format_vm_value(vm, &args[1])
        } else {
            "assertion failed in guest VM".to_string()
        };
        return Err(RuntimeError::AssertionFailed { message: msg });
    }
    Ok(Value::Null)
}

/// Standard clock builtin: returns milliseconds elapsed since the Unix epoch.
pub fn builtin_clock(_vm: &mut VirtualMachine, _args: &[Value]) -> Result<Value, RuntimeError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| {
        RuntimeError::NativeFunctionError {
            name: "clock".to_string(),
            reason: format!("{e}"),
        }
    })?;
    Ok(Value::I64(now.as_millis() as i64))
}

/// Built-in string concatenation helper: concatenates two VM values into a new string.
pub fn builtin_str_concat(vm: &mut VirtualMachine, args: &[Value]) -> Result<Value, RuntimeError> {
    let first_text = args
        .first()
        .map(|first_arg| format_vm_value(vm, first_arg))
        .unwrap_or_default();
    let second_text = args
        .get(1)
        .map(|second_arg| format_vm_value(vm, second_arg))
        .unwrap_or_default();
    let concatenated = format!("{first_text}{second_text}");
    let string_bytes = concatenated.as_bytes();
    let allocated_address = vm
        .heap
        .allocate(string_bytes.len(), crate::memory::BlockTag::String)?;
    vm.heap.write_bytes(allocated_address, string_bytes)?;
    Ok(Value::Ref(allocated_address))
}
