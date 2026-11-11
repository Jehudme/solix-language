//! Debugger state, breakpoints, stepping modes, and variable reference management.

#![forbid(unsafe_code)]

use std::collections::HashMap;

/// Source line breakpoint descriptor with conditions and execution counters.
#[derive(Debug, Clone)]
pub struct BreakpointEntry {
    /// Canonical source file path.
    pub source_path: String,
    /// 1-based source line number requested by the user.
    pub requested_line: u32,
    /// 1-based source line number snapped to the nearest executable instruction.
    pub verified_line: u32,
    /// Bytecode instruction counter offset snapped to this line.
    pub instruction_pc: u32,
    /// Optional function ID if resolved against a compiled binary.
    pub function_id: Option<u32>,
    /// Optional boolean Solix expression guard evaluated in frame context.
    pub condition_expression: Option<String>,
    /// Optional hit-count condition expression (e.g. `== 5`, `>= 10`, `% 2 == 0`).
    pub hit_condition_expression: Option<String>,
    /// Optional logpoint template string with `{variable}` interpolation.
    pub log_message_template: Option<String>,
    /// Monotonically increasing hit counter incremented on every pass.
    pub current_hit_count: u64,
}

/// Function entry breakpoint descriptor.
#[derive(Debug, Clone)]
pub struct FunctionBreakpointEntry {
    /// Qualified or unqualified function name (e.g. `Point::distance_from_origin` or `main`).
    pub function_name: String,
    /// Optional boolean Solix expression guard.
    pub condition_expression: Option<String>,
    /// Optional hit-count condition expression.
    pub hit_condition_expression: Option<String>,
    /// Hit counter.
    pub current_hit_count: u64,
}

/// Active execution stepping mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SteppingState {
    /// Free execution until next breakpoint or trap.
    None,
    /// Step Over: advance until same or shallower frame depth and new source line.
    StepOver {
        start_call_depth: usize,
        start_file_path: String,
        start_line_number: u32,
    },
    /// Step In: advance until any new source line in a debug-enabled function.
    StepIn {
        start_file_path: String,
        start_line_number: u32,
    },
    /// Step Out: advance until call depth is strictly less than target depth.
    StepOut {
        target_call_depth: usize,
    },
}

/// Internal classification of variable references exposed to VS Code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableScopeType {
    /// Local variables and method parameters of a specific stack frame.
    FrameLocals(usize),
    /// Static and global variables partition.
    GlobalStatics,
    /// Raw virtual machine registers and execution telemetry.
    VmRegisters(usize),
    /// Fields of a heap-allocated class instance (`Object` or `ObjectPod`).
    ObjectFields(u32),
    /// Elements of an indexed dynamic heap array.
    ArrayElements(u32),
}

/// Allocator and tracker for hierarchical variable references in the VS Code Variables pane.
#[derive(Debug, Default)]
pub struct VariableReferenceManager {
    /// Next available reference identifier (positive 64-bit integer > 0).
    next_reference_id: i64,
    /// Map from external reference ID to internal scope descriptor.
    id_to_scope_map: HashMap<i64, VariableScopeType>,
}

impl VariableReferenceManager {
    /// Creates a new `VariableReferenceManager` with reference IDs beginning at 1000.
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_reference_id: 1000,
            id_to_scope_map: HashMap::new(),
        }
    }

    /// Clears all transient variable references between stepping or execution resumes.
    pub fn reset_transient_references(&mut self) {
        self.next_reference_id = 1000;
        self.id_to_scope_map.clear();
    }

    /// Allocates a unique reference ID for a variable scope or structured heap reference.
    pub fn allocate_reference(&mut self, scope_type: VariableScopeType) -> i64 {
        let reference_id = self.next_reference_id;
        self.next_reference_id += 1;
        self.id_to_scope_map.insert(reference_id, scope_type);
        reference_id
    }

    /// Resolves an external variable reference ID to its internal scope descriptor.
    #[must_use]
    pub fn resolve_reference(&self, reference_id: i64) -> Option<&VariableScopeType> {
        self.id_to_scope_map.get(&reference_id)
    }
}
