//! Virtual register allocator with lexical scope tracking, scratch register recycling, and contiguous window reservations.

#![forbid(unsafe_code)]

use solix_common::diagnostics::EmissionError;
use std::collections::{BTreeSet, HashMap};

/// Maximum register index allowed in a single function frame ($2^{16} - 1 = 65,535$).
pub const MAX_REGISTERS_PER_FRAME: usize = u16::MAX as usize;

/// Maximum number of arguments that can be passed in a windowed call ($2^8 - 1 = 255$).
pub const MAX_ARGUMENTS_PER_CALL: usize = u8::MAX as usize;

/// Lexical scope tracking local variable bindings allocated within this scope level.
#[derive(Debug, Clone, Default)]
struct ScopeLevel {
    /// Mapping of variable names to their bound register indices.
    bindings: HashMap<String, u16>,
    /// Registers allocated in this scope level (to be freed when exiting scope).
    allocated: Vec<u16>,
}

/// Allocates and manages 16-bit virtual registers for a single function execution frame.
#[derive(Debug, Clone)]
pub struct RegisterAllocator {
    /// Number of incoming parameters occupying the first `param_count` registers (`r0 .. r(param_count-1)`).
    param_count: u16,
    /// Stack of active lexical scopes for variable resolution.
    scopes: Vec<ScopeLevel>,
    /// Next monotonic register index to allocate when no recycled registers are available.
    next_register: u16,
    /// Pool of recycled temporary/local registers available for reuse.
    free_registers: BTreeSet<u16>,
    /// Peak register count required by this frame (`max(register_index) + 1`).
    peak_count: u16,
}

impl RegisterAllocator {
    /// Creates a new register allocator for a function with `param_count` incoming parameters.
    #[must_use]
    pub fn new(param_count: u16) -> Self {
        let mut allocator = Self {
            param_count,
            scopes: vec![ScopeLevel::default()],
            next_register: param_count,
            free_registers: BTreeSet::new(),
            peak_count: param_count,
        };
        allocator.scopes.reserve(8);
        allocator
    }

    /// Allocates an incoming parameter register bound to `name`.
    ///
    /// # Errors
    /// Returns `EmissionError::RegisterOverflow` if parameter count exceeds `u16::MAX`.
    pub fn alloc_param(&mut self, name: impl Into<String>) -> Result<u16, EmissionError> {
        let reg = self.scopes[0].allocated.len() as u16;
        if (reg as usize) >= MAX_REGISTERS_PER_FRAME {
            return Err(EmissionError::RegisterOverflow {
                register_index: reg as usize,
                max: MAX_REGISTERS_PER_FRAME,
            });
        }
        let var_name = name.into();
        self.scopes[0].bindings.insert(var_name, reg);
        self.scopes[0].allocated.push(reg);
        self.update_peak(reg);
        Ok(reg)
    }

    /// Enters a new lexical block scope.
    pub fn enter_scope(&mut self) {
        self.scopes.push(ScopeLevel::default());
    }

    /// Exits the current lexical scope, reclaiming its local variable registers.
    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            let popped_scope = self.scopes.pop();
            if let Some(scope) = popped_scope {
                for reg in scope.allocated {
                    self.free_registers.insert(reg);
                }
            }
        }
    }

    /// Allocates a register for a local variable in the current scope.
    ///
    /// # Errors
    /// Returns `EmissionError::RegisterOverflow` if register capacity is exceeded.
    pub fn alloc_local(&mut self, name: impl Into<String>) -> Result<u16, EmissionError> {
        let reg = self.take_register()?;
        let var_name = name.into();
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.bindings.insert(var_name, reg);
            current_scope.allocated.push(reg);
        }
        self.update_peak(reg);
        Ok(reg)
    }

    /// Binds an existing allocated register to a local variable in the current scope.
    pub fn bind_local(&mut self, name: impl Into<String>, reg: u16) {
        let var_name = name.into();
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.bindings.insert(var_name, reg);
            current_scope.allocated.push(reg);
        }
        self.update_peak(reg);
    }

    /// Looks up a variable name across all active lexical scopes (innermost first).
    #[must_use]
    pub fn lookup_local(&self, name: &str) -> Option<u16> {
        for scope in self.scopes.iter().rev() {
            if let Some(&reg) = scope.bindings.get(name) {
                return Some(reg);
            }
        }
        None
    }

    /// Returns the current nesting depth of lexical scopes.
    #[must_use]
    pub fn scope_depth(&self) -> usize {
        self.scopes.len()
    }

    /// Returns the registers allocated in the innermost lexical scope.
    #[must_use]
    pub fn current_scope_allocated(&self) -> Option<Vec<u16>> {
        self.scopes.last().map(|s| s.allocated.clone())
    }

    /// Collects all registers allocated in scopes at depth >= `from_depth`.
    #[must_use]
    pub fn scopes_allocated_from(&self, from_depth: usize) -> Vec<u16> {
        let mut result = Vec::new();
        for scope in self.scopes.iter().skip(from_depth) {
            result.extend_from_slice(&scope.allocated);
        }
        result
    }

    /// Collects all currently allocated local and parameter registers across all active scopes.
    #[must_use]
    pub fn all_allocated_registers(&self) -> Vec<u16> {
        let mut result = Vec::new();
        for scope in &self.scopes {
            result.extend_from_slice(&scope.allocated);
        }
        result
    }

    /// Checks whether a register is actively bound to an incoming parameter or local variable.
    #[must_use]
    pub fn is_active_local_or_param(&self, reg: u16) -> bool {
        if reg < self.param_count {
            return true;
        }
        for scope in &self.scopes {
            if scope.allocated.contains(&reg) {
                return true;
            }
        }
        false
    }

    /// Allocates a temporary scratch register for intermediate expression evaluation.
    ///
    /// # Errors
    /// Returns `EmissionError::RegisterOverflow` if register capacity is exceeded.
    pub fn alloc_temp(&mut self) -> Result<u16, EmissionError> {
        let reg = self.take_register()?;
        self.update_peak(reg);
        Ok(reg)
    }

    /// Releases a temporary scratch register back to the pool for reuse.
    pub fn free_temp(&mut self, reg: u16) {
        if !self.is_active_local_or_param(reg) {
            self.free_registers.insert(reg);
        }
    }

    /// Reserves a strictly contiguous window of `arg_count` registers for a function call.
    ///
    /// # Errors
    /// - Returns `EmissionError::ArgumentCountOverflow` if `arg_count > 255`.
    /// - Returns `EmissionError::RegisterOverflow` if the window exceeds frame limits.
    pub fn alloc_window(&mut self, arg_count: usize) -> Result<u16, EmissionError> {
        if arg_count == 0 {
            return Ok(0);
        }
        if arg_count > MAX_ARGUMENTS_PER_CALL {
            return Err(EmissionError::ArgumentCountOverflow {
                argument_count: arg_count,
                max: MAX_ARGUMENTS_PER_CALL,
            });
        }

        // Allocate consecutive registers above current peak/next_register to guarantee contiguity
        let base = self.next_register;
        let end = base as usize + arg_count;
        if end > MAX_REGISTERS_PER_FRAME {
            return Err(EmissionError::RegisterOverflow {
                register_index: end,
                max: MAX_REGISTERS_PER_FRAME,
            });
        }

        self.next_register = end as u16;
        for reg in base..self.next_register {
            self.free_registers.remove(&reg);
            self.update_peak(reg);
        }

        Ok(base)
    }

    /// Releases a contiguous window of registers reserved by `alloc_window`.
    pub fn free_window(&mut self, base: u16, arg_count: usize) {
        for offset in 0..arg_count {
            let reg = base + offset as u16;
            if reg >= self.param_count {
                self.free_registers.insert(reg);
            }
        }
    }

    /// Returns the total peak frame size (number of registers) required by this function.
    #[must_use]
    pub fn peak_registers(&self) -> u16 {
        self.peak_count
    }

    /// Returns the number of parameter registers.
    #[must_use]
    pub fn param_count(&self) -> u16 {
        self.param_count
    }

    fn take_register(&mut self) -> Result<u16, EmissionError> {
        if let Some(&recycled) = self.free_registers.iter().next() {
            self.free_registers.remove(&recycled);
            return Ok(recycled);
        }

        let reg = self.next_register;
        if (reg as usize) >= MAX_REGISTERS_PER_FRAME {
            return Err(EmissionError::RegisterOverflow {
                register_index: reg as usize,
                max: MAX_REGISTERS_PER_FRAME,
            });
        }
        self.next_register = reg.saturating_add(1);
        Ok(reg)
    }

    fn update_peak(&mut self, reg: u16) {
        let count = reg.saturating_add(1);
        if count > self.peak_count {
            self.peak_count = count;
        }
    }
}
