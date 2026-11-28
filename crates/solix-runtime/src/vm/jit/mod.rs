//! Cranelift JIT compilation engine, ABI context, and runtime helpers.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod compiler;

use crate::memory::BlockTag;
use crate::options::JitMode;
use crate::value::Value;
use crate::vm::engine::VirtualMachine;
use crate::vm::fast::{VmTrap, alloc_array_trap, alloc_object_trap, inc_ref_inline};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module};
use solix_common::bytecode::OpCode;

/// Native register type inferred for a Solix virtual register inside JIT code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegType {
    I32,
    I64,
    F64,
    Ref,
}

impl RegType {
    #[inline(always)]
    pub(crate) const fn to_code(self) -> u32 {
        match self {
            Self::I32 => 0,
            Self::I64 => 1,
            Self::F64 => 2,
            Self::Ref => 3,
        }
    }

    #[inline(always)]
    pub(crate) const fn from_code(code: u32) -> Self {
        match code & 3 {
            0 => Self::I32,
            1 => Self::I64,
            2 => Self::F64,
            _ => Self::Ref,
        }
    }
}

/// Function signature metadata for a JIT-compiled Solix function.
#[derive(Debug, Clone)]
pub(crate) struct JitFuncSig {
    pub param_types: Vec<RegType>,
    pub return_type: Option<RegType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JitCompileState {
    Pending,
    Compiled,
    Unsupported,
}

pub(crate) struct JitFunctionSlot {
    pub ptr: *const u8,
    pub clif_id: Option<FuncId>,
    pub sig: Option<JitFuncSig>,
    pub call_count: u32,
    pub has_loop: bool,
    pub state: JitCompileState,
}

// SAFETY: Compiled JIT code pointers live inside `JitEngine.module` and are owned by `VirtualMachine`.
unsafe impl Send for JitFunctionSlot {}
unsafe impl Sync for JitFunctionSlot {}

/// C-ABI execution context passed as the first pointer argument to every JIT function.
#[repr(C)]
pub struct JitContext {
    pub heap_ptr: *mut u8,
    pub heap_len: usize,
    pub vm_ptr: *mut VirtualMachine,
    pub static_size: usize,
    pub bump_cursor: usize,
    pub bump_limit: usize,
    pub jit_alloc_count: u64,
    pub jit_alloc_bytes: usize,
    pub has_trap: u8,
    pub cycle_collector_enabled: u8,
    pub(crate) pending_trap: Option<VmTrap>,
}

impl JitContext {
    #[inline(always)]
    fn from_vm(vm: &mut VirtualMachine) -> Self {
        let mut ctx = Self {
            heap_ptr: std::ptr::null_mut(),
            heap_len: 0,
            vm_ptr: vm as *mut VirtualMachine,
            static_size: 0,
            bump_cursor: 0,
            bump_limit: 0,
            jit_alloc_count: 0,
            jit_alloc_bytes: 0,
            has_trap: 0,
            cycle_collector_enabled: u8::from(vm.cycle_collector_enabled),
            pending_trap: None,
        };
        ctx.refresh_heap(vm);
        ctx
    }

    #[inline(always)]
    fn sync_to_vm(&mut self, vm: &mut VirtualMachine) {
        vm.heap
            .commit_jit_bump(self.bump_cursor, self.jit_alloc_count, self.jit_alloc_bytes);
        self.jit_alloc_count = 0;
        self.jit_alloc_bytes = 0;
    }

    #[inline(always)]
    fn refresh_heap(&mut self, vm: &mut VirtualMachine) {
        self.heap_ptr = vm.heap.buffer.as_mut_ptr();
        self.heap_len = vm.heap.buffer.len();
        let static_size = vm.heap.static_size();
        self.static_size = static_size;
        self.bump_cursor = vm.heap.bump_cursor;
        // Only expose committed zeroed tail above high_water_zeroed so inline bump alloc
        // never skips zero-initialization on reused/dirty buffer regions.
        self.bump_limit = if static_size + vm.heap.bump_cursor >= vm.heap.high_water_zeroed() {
            vm.heap.buffer.len().saturating_sub(static_size)
        } else {
            vm.heap.bump_cursor
        };
    }

    #[inline(never)]
    #[cold]
    fn set_trap(&mut self, trap: VmTrap) {
        self.has_trap = 1;
        self.pending_trap = Some(trap);
    }
}

/// Cached Cranelift `FuncId`s for `extern "C"` runtime helper symbols.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct JitRuntimeSymbols {
    pub rt_alloc_object: FuncId,
    pub rt_alloc_array: FuncId,
    pub rt_inc_ref: FuncId,
    pub rt_dec_ref: FuncId,
    pub rt_get_static: FuncId,
    pub rt_set_static: FuncId,
    pub rt_load_const: FuncId,
    pub rt_call_slow: FuncId,
    pub rt_call_symbol_slow: FuncId,
    pub rt_call_virtual_slow: FuncId,
    pub rt_call_interface_slow: FuncId,
    pub rt_set_field_ref: FuncId,
    pub rt_get_array_slow: FuncId,
    pub rt_set_array_slow: FuncId,
    pub rt_string_eq: FuncId,
    pub rt_trap_null: FuncId,
    pub rt_trap_div_zero: FuncId,
    pub rt_trap_oob: FuncId,
}

/// Self-contained Cranelift JIT module and per-function compilation state.
pub(crate) struct JitEngine {
    pub module: JITModule,
    pub compiled_count: u64,
    pub compile_time_ns: u64,
    pub slots: Vec<JitFunctionSlot>,
    pub rt_symbols: JitRuntimeSymbols,
}

impl std::fmt::Debug for JitEngine {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("JitEngine")
            .field("compiled_count", &self.compiled_count)
            .field("compile_time_ns", &self.compile_time_ns)
            .field("slots_len", &self.slots.len())
            .finish()
    }
}

impl JitEngine {
    /// Initializes a new Cranelift `JITModule` configured for the host CPU with `speed` optimization.
    pub fn try_new(vm: &VirtualMachine) -> Option<Self> {
        let mut flag_builder = settings::builder();
        flag_builder.set("opt_level", "speed").ok()?;
        flag_builder.set("use_colocated_libcalls", "false").ok()?;
        flag_builder.set("is_pic", "false").ok()?;
        let isa_builder = cranelift_native::builder().ok()?;
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .ok()?;

        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        builder.symbol("solix_rt_alloc_object", rt_alloc_object as *const u8);
        builder.symbol("solix_rt_alloc_array", rt_alloc_array as *const u8);
        builder.symbol("solix_rt_inc_ref", rt_inc_ref as *const u8);
        builder.symbol("solix_rt_dec_ref", rt_dec_ref as *const u8);
        builder.symbol("solix_rt_get_static", rt_get_static as *const u8);
        builder.symbol("solix_rt_set_static", rt_set_static as *const u8);
        builder.symbol("solix_rt_load_const", rt_load_const as *const u8);
        builder.symbol("solix_rt_call_slow", rt_call_slow as *const u8);
        builder.symbol(
            "solix_rt_call_symbol_slow",
            rt_call_symbol_slow as *const u8,
        );
        builder.symbol(
            "solix_rt_call_virtual_slow",
            rt_call_virtual_slow as *const u8,
        );
        builder.symbol(
            "solix_rt_call_interface_slow",
            rt_call_interface_slow as *const u8,
        );
        builder.symbol("solix_rt_set_field_ref", rt_set_field_ref as *const u8);
        builder.symbol("solix_rt_get_array_slow", rt_get_array_slow as *const u8);
        builder.symbol("solix_rt_set_array_slow", rt_set_array_slow as *const u8);
        builder.symbol("solix_rt_string_eq", rt_string_eq as *const u8);
        builder.symbol("solix_rt_trap_null", rt_trap_null as *const u8);
        builder.symbol("solix_rt_trap_div_zero", rt_trap_div_zero as *const u8);
        builder.symbol("solix_rt_trap_oob", rt_trap_oob as *const u8);

        let mut module = JITModule::new(builder);
        let rt_symbols = declare_runtime_symbols(&mut module).ok()?;

        let slots = vm
            .context
            .all_functions
            .iter()
            .zip(vm.context.function_meta.iter())
            .map(|(compiled_fn, meta)| {
                let is_native = meta.native_fn.is_some();
                let has_loop = !is_native
                    && compiled_fn.instructions.iter().any(|inst| {
                        matches!(
                            OpCode::from_raw_u8(inst.raw_opcode()),
                            Some(OpCode::ForLoopI32)
                        ) || (matches!(
                            OpCode::from_raw_u8(inst.raw_opcode()),
                            Some(
                                OpCode::Jump
                                    | OpCode::JumpIfTrue
                                    | OpCode::JumpIfFalse
                                    | OpCode::JumpIfI32Imm
                                    | OpCode::JumpIfLtI32
                                    | OpCode::JumpIfLteI32
                                    | OpCode::JumpIfGtI32
                                    | OpCode::JumpIfGteI32
                                    | OpCode::JumpIfEqI32
                                    | OpCode::JumpIfNeqI32
                            )
                        ) && inst.jump_offset() < 0)
                    });
                let state = if is_native {
                    JitCompileState::Unsupported
                } else {
                    JitCompileState::Pending
                };
                JitFunctionSlot {
                    ptr: std::ptr::null(),
                    clif_id: None,
                    sig: None,
                    call_count: 0,
                    has_loop,
                    state,
                }
            })
            .collect();

        Some(Self {
            module,
            compiled_count: 0,
            compile_time_ns: 0,
            slots,
            rt_symbols,
        })
    }
}

fn declare_runtime_symbols(module: &mut JITModule) -> Result<JitRuntimeSymbols, ()> {
    use cranelift_codegen::ir::{AbiParam, Signature, types};

    let call_conv = module.isa().default_call_conv();
    let ptr_ty = module.target_config().pointer_type();

    let mut declare =
        |name: &str, params: &[types::Type], ret: Option<types::Type>| -> Result<FuncId, ()> {
            let mut sig = Signature::new(call_conv);
            for &ty in params {
                sig.params.push(AbiParam::new(ty));
            }
            if let Some(ret_ty) = ret {
                sig.returns.push(AbiParam::new(ret_ty));
            }
            module
                .declare_function(name, Linkage::Import, &sig)
                .map_err(|_| ())
        };

    Ok(JitRuntimeSymbols {
        rt_alloc_object: declare(
            "solix_rt_alloc_object",
            &[ptr_ty, types::I32, types::I32, types::I8],
            Some(types::I32),
        )?,
        rt_alloc_array: declare(
            "solix_rt_alloc_array",
            &[ptr_ty, types::I32, types::I32, types::I32, types::I32],
            Some(types::I32),
        )?,
        rt_inc_ref: declare("solix_rt_inc_ref", &[ptr_ty, types::I32], None)?,
        rt_dec_ref: declare("solix_rt_dec_ref", &[ptr_ty, types::I32], None)?,
        rt_get_static: declare(
            "solix_rt_get_static",
            &[ptr_ty, types::I32],
            Some(types::I64),
        )?,
        rt_set_static: declare(
            "solix_rt_set_static",
            &[ptr_ty, types::I32, types::I64],
            None,
        )?,
        rt_load_const: declare(
            "solix_rt_load_const",
            &[ptr_ty, types::I32, types::I32],
            Some(types::I64),
        )?,
        rt_call_slow: declare(
            "solix_rt_call_slow",
            &[
                ptr_ty,
                types::I32,
                ptr_ty,
                types::I32,
                ptr_ty,
                types::I32,
                types::I32,
            ],
            Some(types::I8),
        )?,
        rt_call_symbol_slow: declare(
            "solix_rt_call_symbol_slow",
            &[
                ptr_ty,
                types::I32,
                types::I32,
                ptr_ty,
                types::I32,
                ptr_ty,
                types::I32,
                types::I32,
            ],
            Some(types::I8),
        )?,
        rt_call_virtual_slow: declare(
            "solix_rt_call_virtual_slow",
            &[
                ptr_ty,
                types::I32,
                ptr_ty,
                types::I32,
                ptr_ty,
                types::I32,
                types::I32,
            ],
            Some(types::I8),
        )?,
        rt_call_interface_slow: declare(
            "solix_rt_call_interface_slow",
            &[
                ptr_ty,
                types::I32,
                ptr_ty,
                types::I32,
                ptr_ty,
                types::I32,
                types::I32,
            ],
            Some(types::I8),
        )?,
        rt_set_field_ref: declare(
            "solix_rt_set_field_ref",
            &[ptr_ty, types::I32, types::I32, types::I32, types::I8],
            None,
        )?,
        rt_get_array_slow: declare(
            "solix_rt_get_array_slow",
            &[ptr_ty, types::I32, types::I64, types::I32],
            Some(types::I64),
        )?,
        rt_set_array_slow: declare(
            "solix_rt_set_array_slow",
            &[
                ptr_ty,
                types::I32,
                types::I64,
                types::I64,
                types::I32,
                types::I8,
            ],
            None,
        )?,
        rt_string_eq: declare(
            "solix_rt_string_eq",
            &[ptr_ty, types::I32, types::I32],
            Some(types::I8),
        )?,
        rt_trap_null: declare("solix_rt_trap_null", &[ptr_ty], None)?,
        rt_trap_div_zero: declare("solix_rt_trap_div_zero", &[ptr_ty], None)?,
        rt_trap_oob: declare("solix_rt_trap_oob", &[ptr_ty, types::I64, types::I64], None)?,
    })
}

/// Converts a raw `Value` into a 64-bit JIT register payload according to `RegType`.
#[inline(always)]
fn value_to_jit_bits(val: Value, reg_ty: RegType) -> Result<u64, VmTrap> {
    match reg_ty {
        RegType::I32 => match val {
            Value::I32(num) => Ok(num as u32 as u64),
            Value::Bool(flag) => Ok(u64::from(flag)),
            Value::I64(num) => Ok(num as i32 as u32 as u64),
            _ => Err(VmTrap::type_mismatch("int32", &val)),
        },
        RegType::I64 => match val {
            Value::I64(num) => Ok(num as u64),
            Value::I32(num) => Ok(i64::from(num) as u64),
            _ => Err(VmTrap::type_mismatch("int64", &val)),
        },
        RegType::F64 => match val {
            Value::F64(num) => Ok(num.to_bits()),
            Value::I32(num) => Ok(f64::from(num).to_bits()),
            Value::I64(num) => Ok((num as f64).to_bits()),
            _ => Err(VmTrap::type_mismatch("float64", &val)),
        },
        RegType::Ref => match val {
            Value::Ref(addr) => Ok(u64::from(addr)),
            Value::Null => Ok(0),
            _ => Err(VmTrap::type_mismatch("ref", &val)),
        },
    }
}

/// Converts a 64-bit JIT return payload back into a Solix `Value`.
#[inline(always)]
fn jit_bits_to_value(bits: u64, reg_ty: RegType) -> Value {
    match reg_ty {
        RegType::I32 => Value::I32(bits as i32),
        RegType::I64 => Value::I64(bits as i64),
        RegType::F64 => Value::F64(f64::from_bits(bits)),
        RegType::Ref => {
            let addr = bits as u32;
            if addr == 0 {
                Value::Null
            } else {
                Value::Ref(addr)
            }
        }
    }
}

/// Attempts to execute `func_id` via Cranelift JIT when `JitMode` and eligibility permit.
///
/// Returns `None` if the function is cold or unsupported by JIT (caller should run `fast.rs`).
#[inline(never)]
pub(crate) fn try_invoke_jit(
    vm: &mut VirtualMachine,
    func_id: u32,
    caller_dst: u16,
    caller_arg_start: usize,
    arg_count: usize,
) -> Option<Result<(), VmTrap>> {
    if vm.jit_mode == JitMode::Off || arg_count > 4 {
        return None;
    }
    let idx = func_id as usize;
    if vm.jit_engine.is_none() {
        let engine = JitEngine::try_new(vm)?;
        vm.jit_engine = Some(Box::new(engine));
    }

    let (should_compile, slot_ptr, sig_opt) = {
        let engine = vm.jit_engine.as_mut()?;
        let slot = engine.slots.get_mut(idx)?;
        if slot.state == JitCompileState::Unsupported {
            return None;
        }
        slot.call_count = slot.call_count.saturating_add(1);
        let trigger = match vm.jit_mode {
            JitMode::Off => false,
            JitMode::Eager => true,
            JitMode::Tiered => slot.has_loop || slot.call_count >= 50,
        };
        if slot.state == JitCompileState::Pending && !trigger {
            return None;
        }
        (
            slot.state == JitCompileState::Pending && trigger,
            slot.ptr,
            slot.sig.clone(),
        )
    };

    let (fn_ptr, sig) = if should_compile {
        let compiled = compiler::ensure_compiled(vm, func_id);
        match compiled {
            Some((ptr, sig)) => (ptr, sig),
            None => return None,
        }
    } else if !slot_ptr.is_null() {
        (slot_ptr, sig_opt?)
    } else {
        return None;
    };

    if sig.param_types.len() != arg_count {
        return None;
    }

    let mut raw_args = [0u64; 4];
    for (arg_idx, &reg_ty) in sig.param_types.iter().enumerate() {
        let val = vm.register_stack[caller_arg_start + arg_idx];
        match value_to_jit_bits(val, reg_ty) {
            Ok(bits) => raw_args[arg_idx] = bits,
            Err(_) => {
                if let Some(engine) = vm.jit_engine.as_mut()
                    && let Some(slot) = engine.slots.get_mut(idx)
                {
                    slot.state = JitCompileState::Unsupported;
                }
                return None;
            }
        }
    }

    let mut jit_ctx = JitContext::from_vm(vm);

    type JitFn4 = unsafe extern "C" fn(*mut JitContext, u64, u64, u64, u64) -> u64;
    // SAFETY: `fn_ptr` was finalized by `JITModule` with signature `(ptr, i64, i64, i64, i64) -> i64`.
    let jit_fn: JitFn4 = unsafe { std::mem::transmute(fn_ptr) };
    let ret_bits = unsafe {
        jit_fn(
            &mut jit_ctx,
            raw_args[0],
            raw_args[1],
            raw_args[2],
            raw_args[3],
        )
    };
    jit_ctx.sync_to_vm(vm);

    if jit_ctx.has_trap != 0 {
        let trap = jit_ctx
            .pending_trap
            .take()
            .unwrap_or_else(VmTrap::null_reference);
        return Some(Err(trap));
    }

    let ret_val = match sig.return_type {
        Some(ret_ty) => jit_bits_to_value(ret_bits, ret_ty),
        None => Value::Null,
    };

    if caller_dst != u16::MAX {
        if let Some(frame) = vm.call_frames.last() {
            let dst_idx = frame.register_base as usize + caller_dst as usize;
            if dst_idx < vm.register_stack.len() {
                vm.register_stack[dst_idx] = ret_val;
            }
        }
    } else {
        vm.last_return_value = ret_val;
    }
    Some(Ok(()))
}

/// Attempts to run the current top-of-stack entrypoint frame (`call_frames.len() == 1`) in JIT.
#[inline(never)]
pub(crate) fn try_run_entry_jit(
    vm: &mut VirtualMachine,
    target_depth: usize,
) -> Option<Result<(), VmTrap>> {
    if vm.jit_mode == JitMode::Off || vm.call_frames.len() != target_depth + 1 {
        return None;
    }
    let frame = *vm.call_frames.last()?;
    let func_id = frame.function_id;
    let arg_count = vm.context.all_functions.get(func_id as usize)?.param_count as usize;
    let caller_arg_start = frame.register_base as usize;
    let res = try_invoke_jit(vm, func_id, u16::MAX, caller_arg_start, arg_count)?;
    if res.is_ok() {
        let _ = vm.call_frames.pop();
    }
    Some(res)
}

unsafe extern "C" fn rt_alloc_object(
    ctx_ptr: *mut JitContext,
    vtable_id: u32,
    size: u32,
    is_pod: u8,
) -> u32 {
    // SAFETY: `ctx_ptr` is a valid pointer to `JitContext` on the caller stack.
    let ctx = unsafe { &mut *ctx_ptr };
    // SAFETY: `ctx.vm_ptr` points to the active `VirtualMachine` for the duration of JIT execution.
    let vm = unsafe { &mut *ctx.vm_ptr };
    ctx.sync_to_vm(vm);
    let tag = if is_pod != 0 {
        BlockTag::ObjectPod
    } else {
        BlockTag::Object
    };
    match alloc_object_trap(&mut vm.heap, size as usize, tag, vtable_id) {
        Ok(addr) => {
            ctx.refresh_heap(vm);
            addr
        }
        Err(trap) => {
            ctx.set_trap(trap);
            0
        }
    }
}

unsafe extern "C" fn rt_alloc_array(
    ctx_ptr: *mut JitContext,
    array_len: i32,
    elem_size: u32,
    type_tag: u32,
    vtable_id: u32,
) -> u32 {
    let ctx = unsafe { &mut *ctx_ptr };
    let vm = unsafe { &mut *ctx.vm_ptr };
    ctx.sync_to_vm(vm);
    if array_len < 0 {
        ctx.set_trap(VmTrap::index_out_of_bounds(i64::from(array_len), 0));
        return 0;
    }
    match alloc_array_trap(
        &mut vm.heap,
        array_len as usize,
        elem_size as usize,
        type_tag as u8,
        vtable_id as u16,
    ) {
        Ok(addr) => {
            ctx.refresh_heap(vm);
            addr
        }
        Err(trap) => {
            ctx.set_trap(trap);
            0
        }
    }
}

unsafe extern "C" fn rt_inc_ref(ctx_ptr: *mut JitContext, addr: u32) {
    if addr == 0 {
        return;
    }
    let ctx = unsafe { &mut *ctx_ptr };
    let vm = unsafe { &mut *ctx.vm_ptr };
    if let Err(trap) = inc_ref_inline(&mut vm.heap, addr, vm.cycle_collector_enabled) {
        ctx.set_trap(trap);
    }
}

unsafe extern "C" fn rt_dec_ref(ctx_ptr: *mut JitContext, addr: u32) {
    if addr == 0 {
        return;
    }
    let ctx = unsafe { &mut *ctx_ptr };
    let vm = unsafe { &mut *ctx.vm_ptr };
    let addr_usize = addr as usize;
    if addr_usize < 8 || addr_usize > vm.heap.buffer.len() {
        ctx.set_trap(VmTrap::null_reference());
        return;
    }
    // SAFETY: `addr_usize >= 8 && addr_usize <= vm.heap.buffer.len()` checked above.
    let word_ptr = unsafe {
        vm.heap
            .buffer
            .as_mut_ptr()
            .add(addr_usize - 4)
            .cast::<u32>()
    };
    let tag_and_ref = unsafe { u32::from_le(std::ptr::read_unaligned(word_ptr)) };
    let ref_count = tag_and_ref & crate::memory::MAX_REF_COUNT;
    if ref_count > 1 && ref_count < crate::memory::MAX_REF_COUNT {
        let updated = (tag_and_ref & !crate::memory::MAX_REF_COUNT) | (ref_count - 1);
        unsafe {
            std::ptr::write_unaligned(word_ptr, updated.to_le());
        }
        if vm.cycle_collector_enabled
            && ((updated >> 26) & 3) != 3
            && crate::memory::layout::can_contain_references(BlockTag::from_nibble(
                ((updated >> 28) & 0x0F) as u8,
            ))
        {
            ctx.sync_to_vm(vm);
            if let Err(trap) = vm.push_purple_candidate_cold(addr, updated) {
                ctx.set_trap(trap);
            } else {
                ctx.refresh_heap(vm);
            }
        }
    } else if ref_count == 1 {
        ctx.sync_to_vm(vm);
        if let Err(trap) = vm.release_reference_cold(addr) {
            ctx.set_trap(trap);
        } else {
            ctx.refresh_heap(vm);
        }
    }
}

unsafe extern "C" fn rt_get_static(ctx_ptr: *mut JitContext, abs_offset: u32) -> u64 {
    let ctx = unsafe { &mut *ctx_ptr };
    let vm = unsafe { &mut *ctx.vm_ptr };
    match vm.heap.read_u64(abs_offset) {
        Ok(raw) => raw,
        Err(err) => {
            ctx.set_trap(VmTrap::from_error(err));
            0
        }
    }
}

unsafe extern "C" fn rt_set_static(ctx_ptr: *mut JitContext, abs_offset: u32, raw_val: u64) {
    let ctx = unsafe { &mut *ctx_ptr };
    let vm = unsafe { &mut *ctx.vm_ptr };
    if let Err(err) = vm.heap.write_u64(abs_offset, raw_val) {
        ctx.set_trap(VmTrap::from_error(err));
    }
}

unsafe extern "C" fn rt_load_const(ctx_ptr: *mut JitContext, binary_id: u32, pool_idx: u32) -> u64 {
    let ctx = unsafe { &mut *ctx_ptr };
    let vm = unsafe { &mut *ctx.vm_ptr };
    match vm.const_pool_cache[binary_id as usize][pool_idx as usize] {
        Value::I32(num) => num as u32 as u64,
        Value::Bool(flag) => u64::from(flag),
        Value::I64(num) => num as u64,
        Value::F64(num) => num.to_bits(),
        Value::Ref(addr) => u64::from(addr),
        _ => 0,
    }
}

#[inline(always)]
unsafe fn marshal_jit_args(
    vm: &mut VirtualMachine,
    args_ptr: *const u64,
    count: usize,
    arg_types_mask: u32,
) -> usize {
    let arg_start = vm.register_stack.len();
    vm.register_stack
        .resize(arg_start + count.max(1), Value::Null);
    for arg_idx in 0..count {
        // SAFETY: `args_ptr` points to `count` `u64` raw JIT register words on caller stack.
        let raw_word = unsafe { *args_ptr.add(arg_idx) };
        let reg_ty = RegType::from_code(arg_types_mask >> (arg_idx * 2));
        vm.register_stack[arg_start + arg_idx] = jit_bits_to_value(raw_word, reg_ty);
    }
    arg_start
}

unsafe fn finish_jit_slow_return(
    ctx: &mut JitContext,
    vm: &mut VirtualMachine,
    arg_start: usize,
    out_ptr: *mut u64,
    ret_type_code: u32,
) -> u8 {
    let ret_raw = if ret_type_code < 4 {
        if matches!(vm.last_return_value, Value::Null) {
            0
        } else {
            match value_to_jit_bits(vm.last_return_value, RegType::from_code(ret_type_code)) {
                Ok(bits) => bits,
                Err(trap) => {
                    vm.register_stack.truncate(arg_start);
                    ctx.set_trap(trap);
                    return 1;
                }
            }
        }
    } else {
        0
    };
    vm.register_stack.truncate(arg_start);
    if !out_ptr.is_null() {
        // SAFETY: `out_ptr` points to a valid `u64` slot on caller stack.
        unsafe {
            *out_ptr = ret_raw;
        }
    }
    ctx.refresh_heap(vm);
    0
}

unsafe fn dispatch_func_from_jit_slow(
    ctx: &mut JitContext,
    vm: &mut VirtualMachine,
    func_id: u32,
    arg_start: usize,
    count: usize,
    out_ptr: *mut u64,
    ret_type_code: u32,
) -> u8 {
    if vm.jit_mode != JitMode::Off
        && count <= 4
        && let Some(jit_res) = try_invoke_jit(vm, func_id, u16::MAX, arg_start, count)
    {
        if let Err(trap) = jit_res {
            vm.register_stack.truncate(arg_start);
            ctx.set_trap(trap);
            return 1;
        }
        return unsafe { finish_jit_slow_return(ctx, vm, arg_start, out_ptr, ret_type_code) };
    }

    let base_depth = vm.call_frames.len();
    if let Err(trap) = vm.dispatch_call_slow(func_id, u16::MAX, arg_start, count) {
        vm.register_stack.truncate(arg_start);
        ctx.set_trap(trap);
        return 1;
    }
    if vm.call_frames.len() > base_depth
        && let Err(trap) = vm.run_fast_trap::<false, false>(base_depth)
    {
        vm.register_stack.truncate(arg_start);
        ctx.set_trap(trap);
        return 1;
    }
    unsafe { finish_jit_slow_return(ctx, vm, arg_start, out_ptr, ret_type_code) }
}

unsafe extern "C" fn rt_call_slow(
    ctx_ptr: *mut JitContext,
    func_id: u32,
    args_ptr: *const u64,
    arg_count: u32,
    out_ptr: *mut u64,
    arg_types_mask: u32,
    ret_type_code: u32,
) -> u8 {
    let ctx = unsafe { &mut *ctx_ptr };
    let vm = unsafe { &mut *ctx.vm_ptr };
    ctx.sync_to_vm(vm);
    let count = arg_count as usize;
    let arg_start = unsafe { marshal_jit_args(vm, args_ptr, count, arg_types_mask) };
    unsafe {
        dispatch_func_from_jit_slow(ctx, vm, func_id, arg_start, count, out_ptr, ret_type_code)
    }
}

unsafe extern "C" fn rt_call_symbol_slow(
    ctx_ptr: *mut JitContext,
    binary_id: u32,
    sym_idx: u32,
    args_ptr: *const u64,
    arg_count: u32,
    out_ptr: *mut u64,
    arg_types_mask: u32,
    ret_type_code: u32,
) -> u8 {
    let ctx = unsafe { &mut *ctx_ptr };
    let vm = unsafe { &mut *ctx.vm_ptr };
    ctx.sync_to_vm(vm);
    let count = arg_count as usize;
    let arg_start = unsafe { marshal_jit_args(vm, args_ptr, count, arg_types_mask) };
    match vm.jit_resolve_symbol_call(binary_id as usize, sym_idx as usize, arg_start, count) {
        Ok(Some(func_id)) => unsafe {
            dispatch_func_from_jit_slow(ctx, vm, func_id, arg_start, count, out_ptr, ret_type_code)
        },
        Ok(None) => unsafe { finish_jit_slow_return(ctx, vm, arg_start, out_ptr, ret_type_code) },
        Err(trap) => {
            vm.register_stack.truncate(arg_start);
            ctx.set_trap(trap);
            1
        }
    }
}

unsafe extern "C" fn rt_call_virtual_slow(
    ctx_ptr: *mut JitContext,
    raw_target: u32,
    args_ptr: *const u64,
    arg_count: u32,
    out_ptr: *mut u64,
    arg_types_mask: u32,
    ret_type_code: u32,
) -> u8 {
    let ctx = unsafe { &mut *ctx_ptr };
    let vm = unsafe { &mut *ctx.vm_ptr };
    ctx.sync_to_vm(vm);
    let count = arg_count as usize;
    let arg_start = unsafe { marshal_jit_args(vm, args_ptr, count, arg_types_mask) };
    let obj_addr = match vm.register_stack.get(arg_start) {
        Some(Value::Ref(addr)) => *addr,
        _ => 0,
    };
    match vm.jit_resolve_virtual(obj_addr, raw_target as u16) {
        Ok(func_id) => unsafe {
            dispatch_func_from_jit_slow(ctx, vm, func_id, arg_start, count, out_ptr, ret_type_code)
        },
        Err(trap) => {
            vm.register_stack.truncate(arg_start);
            ctx.set_trap(trap);
            1
        }
    }
}

unsafe extern "C" fn rt_call_interface_slow(
    ctx_ptr: *mut JitContext,
    target: u32,
    args_ptr: *const u64,
    arg_count: u32,
    out_ptr: *mut u64,
    arg_types_mask: u32,
    ret_type_code: u32,
) -> u8 {
    let ctx = unsafe { &mut *ctx_ptr };
    let vm = unsafe { &mut *ctx.vm_ptr };
    ctx.sync_to_vm(vm);
    let count = arg_count as usize;
    let arg_start = unsafe { marshal_jit_args(vm, args_ptr, count, arg_types_mask) };
    let obj_addr = match vm.register_stack.get(arg_start) {
        Some(Value::Ref(addr)) => *addr,
        _ => 0,
    };
    match vm.jit_resolve_interface(obj_addr, target as u16) {
        Ok(func_id) => unsafe {
            dispatch_func_from_jit_slow(ctx, vm, func_id, arg_start, count, out_ptr, ret_type_code)
        },
        Err(trap) => {
            vm.register_stack.truncate(arg_start);
            ctx.set_trap(trap);
            1
        }
    }
}

unsafe extern "C" fn rt_set_field_ref(
    ctx_ptr: *mut JitContext,
    obj_addr: u32,
    offset: u32,
    new_addr: u32,
    is_move: u8,
) {
    let ctx = unsafe { &mut *ctx_ptr };
    let vm = unsafe { &mut *ctx.vm_ptr };
    ctx.sync_to_vm(vm);
    if let Err(trap) = vm.jit_set_field_ref(obj_addr, offset as usize, new_addr, is_move != 0) {
        ctx.set_trap(trap);
    } else {
        ctx.refresh_heap(vm);
    }
}

unsafe extern "C" fn rt_get_array_slow(
    ctx_ptr: *mut JitContext,
    arr_addr: u32,
    index_i64: i64,
    expected_ty_code: u32,
) -> u64 {
    let ctx = unsafe { &mut *ctx_ptr };
    let vm = unsafe { &mut *ctx.vm_ptr };
    match vm.jit_get_array_slow(arr_addr, index_i64) {
        Ok(val) => {
            if matches!(val, Value::Null) {
                0
            } else {
                match value_to_jit_bits(val, RegType::from_code(expected_ty_code)) {
                    Ok(bits) => bits,
                    Err(trap) => {
                        ctx.set_trap(trap);
                        0
                    }
                }
            }
        }
        Err(trap) => {
            ctx.set_trap(trap);
            0
        }
    }
}

unsafe extern "C" fn rt_set_array_slow(
    ctx_ptr: *mut JitContext,
    arr_addr: u32,
    index_i64: i64,
    raw_val: u64,
    val_ty_code: u32,
    is_move: u8,
) {
    let ctx = unsafe { &mut *ctx_ptr };
    let vm = unsafe { &mut *ctx.vm_ptr };
    ctx.sync_to_vm(vm);
    let val = jit_bits_to_value(raw_val, RegType::from_code(val_ty_code));
    if let Err(trap) = vm.jit_set_array_slow(arr_addr, index_i64, val, is_move != 0) {
        ctx.set_trap(trap);
    } else {
        ctx.refresh_heap(vm);
    }
}

unsafe extern "C" fn rt_string_eq(ctx_ptr: *mut JitContext, left_addr: u32, right_addr: u32) -> u8 {
    if left_addr == right_addr {
        return 1;
    }
    if left_addr == 0 || right_addr == 0 {
        return 0;
    }
    let ctx = unsafe { &*ctx_ptr };
    let vm = unsafe { &*ctx.vm_ptr };
    let Ok(left_str) = vm.heap.read_string(left_addr) else {
        return 0;
    };
    let Ok(right_str) = vm.heap.read_string(right_addr) else {
        return 0;
    };
    u8::from(left_str == right_str)
}

unsafe extern "C" fn rt_trap_null(ctx_ptr: *mut JitContext) {
    let ctx = unsafe { &mut *ctx_ptr };
    ctx.set_trap(VmTrap::null_reference());
}

unsafe extern "C" fn rt_trap_div_zero(ctx_ptr: *mut JitContext) {
    let ctx = unsafe { &mut *ctx_ptr };
    ctx.set_trap(VmTrap::divide_by_zero());
}

unsafe extern "C" fn rt_trap_oob(ctx_ptr: *mut JitContext, idx: i64, len: usize) {
    let ctx = unsafe { &mut *ctx_ptr };
    ctx.set_trap(VmTrap::index_out_of_bounds(idx, len));
}
