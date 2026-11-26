//! Binary program container (`SLXB`), constant pool, and bit-level serialization.

#![forbid(unsafe_code)]

use crate::bytecode::class::CompiledClass;
use crate::bytecode::function::{
    CompiledFunction, ExceptionHandlerEntry, LocalVariableDebugInfo, SourceMapEntry,
};
use crate::bytecode::instruction::Instruction;
use crate::bytecode::manifest::ExportManifest;
use crate::diagnostics::EmissionError;
use crate::span::Span;
use serde::{Deserialize, Serialize};

/// Magic header identifier for Solix compiled binary files.
pub const MAGIC: &[u8; 4] = b"SLXB";

/// Container format major version number.
pub const VERSION_MAJOR: u16 = 1;

/// Container format minor version number.
pub const VERSION_MINOR: u16 = 0;

/// Typed constant pool entry representing an immutable literal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstantPoolValue {
    /// 64-bit signed integer constant.
    Int64(i64),
    /// 64-bit floating point constant (stored as raw bits for bit-exact reproducibility).
    Float64(u64),
    /// UTF-8 string literal constant.
    String(String),
}

impl ConstantPoolValue {
    /// Constructs a Float64 entry from an `f64`.
    #[must_use]
    pub fn from_f64(value: f64) -> Self {
        Self::Float64(value.to_bits())
    }

    /// Converts this entry to an `f64` if it is a Float64 variant.
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float64(bits) => Some(f64::from_bits(*bits)),
            _ => None,
        }
    }
}

impl std::fmt::Display for ConstantPoolValue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int64(int_value) => write!(formatter, "{int_value}"),
            Self::Float64(raw_bits) => write!(formatter, "{}", f64::from_bits(*raw_bits)),
            Self::String(string_value) => write!(formatter, "\"{string_value}\""),
        }
    }
}

/// Deduplicating constant pool managing numeric and string literals.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ConstantPool {
    /// Array of pool values indexed by position.
    pub values: Vec<ConstantPoolValue>,
}

impl ConstantPool {
    /// Creates a new empty constant pool.
    #[must_use]
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    /// Adds or deduplicates a 64-bit signed integer, returning its index.
    pub fn add_i64(&mut self, value: i64) -> Result<u16, EmissionError> {
        let entry = ConstantPoolValue::Int64(value);
        if let Some(pos) = self.values.iter().position(|existing| *existing == entry) {
            return Ok(pos as u16);
        }
        self.push_value(entry)
    }

    /// Adds or deduplicates a 64-bit float, returning its index.
    pub fn add_f64(&mut self, value: f64) -> Result<u16, EmissionError> {
        let entry = ConstantPoolValue::from_f64(value);
        if let Some(pos) = self.values.iter().position(|existing| *existing == entry) {
            return Ok(pos as u16);
        }
        self.push_value(entry)
    }

    /// Adds or deduplicates a string literal, returning its index.
    pub fn add_string(&mut self, value: impl Into<String>) -> Result<u16, EmissionError> {
        let text = value.into();
        let entry = ConstantPoolValue::String(text);
        if let Some(pos) = self.values.iter().position(|existing| *existing == entry) {
            return Ok(pos as u16);
        }
        self.push_value(entry)
    }

    fn push_value(&mut self, value: ConstantPoolValue) -> Result<u16, EmissionError> {
        if self.values.len() >= u16::MAX as usize {
            return Err(EmissionError::ConstantPoolOverflow {
                count: self.values.len(),
                max: u16::MAX as usize,
            });
        }
        let index = self.values.len() as u16;
        self.values.push(value);
        Ok(index)
    }

    /// Retrieves the constant value at `index`.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&ConstantPoolValue> {
        self.values.get(index)
    }

    /// Number of constants in the pool.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` if the constant pool contains no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// Standalone executable or library binary container (`SLXB`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryProgram {
    /// Format magic header (`SLXB`).
    pub magic: [u8; 4],
    /// Major container version.
    pub version_major: u16,
    /// Minor container version.
    pub version_minor: u16,
    /// Binary flags (bitmask for debug info, optimizations, etc.).
    pub flags: u32,
    /// Function table index of the `main` entry point, if an executable.
    pub entry_point_id: Option<u32>,
    /// Fully qualified symbol of the entry point, if present.
    pub entry_point_symbol: Option<String>,
    /// Total static variable memory slots required by this unit.
    pub static_space_size: u32,
    /// Exported static symbols and their module-local slot indices.
    pub static_symbols_map: Vec<(String, u32)>,
    /// Deduplicated constant pool.
    pub constant_pool: ConstantPool,
    /// Object-oriented class layouts and virtual method tables.
    pub classes: Vec<CompiledClass>,
    /// Vector of compiled functions and instruction chunks.
    pub functions: Vec<CompiledFunction>,
    /// Optional default entrypoint symbol embedded in the binary container.
    #[serde(default)]
    pub entry_point: Option<String>,
    /// Package name for modular libraries.
    pub package_name: String,
    /// Semantic version string.
    pub package_version: String,
    /// Human-readable package description.
    pub package_description: String,
    /// Build profile recorded during compilation ("debug", "release", or "test").
    pub build_profile: String,
    /// Cryptographic SHA-256 digests of bundled native shared libraries.
    pub native_library_checksums: std::collections::HashMap<String, String>,
    /// Declared dynamic shared library dependencies required by this binary.
    #[serde(default)]
    pub dynamic_libraries: Vec<String>,
    /// Pre-parsed AST binary bytes for instant dependency ingestion.
    pub exported_ast_bytes: Vec<u8>,
    /// Full export manifest for dynamic linking and API reflection.
    pub export_manifest: ExportManifest,
    /// Canonical source file paths indexed by `source_id`.
    #[serde(default)]
    pub source_files: Vec<String>,
}

impl Default for BinaryProgram {
    fn default() -> Self {
        Self::new()
    }
}

impl BinaryProgram {
    /// Creates a new empty `BinaryProgram` with current version and magic header.
    #[must_use]
    pub fn new() -> Self {
        Self {
            magic: *MAGIC,
            version_major: VERSION_MAJOR,
            version_minor: VERSION_MINOR,
            flags: 0,
            entry_point_id: None,
            entry_point_symbol: None,
            static_space_size: 0,
            static_symbols_map: Vec::new(),
            constant_pool: ConstantPool::new(),
            classes: Vec::new(),
            functions: Vec::new(),
            entry_point: None,
            package_name: String::new(),
            package_version: "0.1.0".to_string(),
            package_description: String::new(),
            build_profile: "debug".to_string(),
            native_library_checksums: std::collections::HashMap::new(),
            dynamic_libraries: Vec::new(),
            exported_ast_bytes: Vec::new(),
            export_manifest: ExportManifest::default(),
            source_files: Vec::new(),
        }
    }

    /// Computes the canonical 64-character lowercase hexadecimal SHA-256 digest of this binary container.
    #[must_use]
    pub fn compute_sha256(&self) -> String {
        crate::hash::sha256_hex(&self.to_bytes())
    }

    /// Serializes the binary program into raw bytes in the `SLXB` format.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.try_to_bytes().unwrap_or_default()
    }

    /// Serializes the binary program into raw bytes in the `SLXB` format with overflow checks.
    ///
    /// # Errors
    /// Returns `EmissionError::CorruptBinaryContainer` if any container section count, string length,
    /// or serialized JSON descriptor exceeds valid `SLXB` bounds or fails serialization.
    pub fn try_to_bytes(&self) -> Result<Vec<u8>, EmissionError> {
        let mut buffer = Vec::new();
        // 1. Header
        buffer.extend_from_slice(&self.magic);
        buffer.extend_from_slice(&self.version_major.to_le_bytes());
        buffer.extend_from_slice(&self.version_minor.to_le_bytes());
        buffer.extend_from_slice(&self.flags.to_le_bytes());

        // Entry point
        match self.entry_point_id {
            Some(id) => {
                buffer.push(1);
                buffer.extend_from_slice(&id.to_le_bytes());
            }
            None => buffer.push(0),
        }
        match &self.entry_point_symbol {
            Some(sym) => {
                buffer.push(1);
                write_checked_string(&mut buffer, sym, "Header")?;
            }
            None => buffer.push(0),
        }

        // Static reservation
        buffer.extend_from_slice(&self.static_space_size.to_le_bytes());

        // 2. Static Symbols Map
        let static_sym_len = checked_u32_len(self.static_symbols_map.len(), "StaticSymbols")?;
        buffer.extend_from_slice(&static_sym_len.to_le_bytes());
        for (symbol, slot) in &self.static_symbols_map {
            write_checked_string(&mut buffer, symbol, "StaticSymbols")?;
            buffer.extend_from_slice(&slot.to_le_bytes());
        }

        // 3. Constant Pool
        let pool_len = checked_u32_len(self.constant_pool.values.len(), "ConstantPool")?;
        buffer.extend_from_slice(&pool_len.to_le_bytes());
        for val in &self.constant_pool.values {
            match val {
                ConstantPoolValue::Int64(int_val) => {
                    buffer.push(1);
                    buffer.extend_from_slice(&int_val.to_le_bytes());
                }
                ConstantPoolValue::Float64(bits) => {
                    buffer.push(2);
                    buffer.extend_from_slice(&bits.to_le_bytes());
                }
                ConstantPoolValue::String(text) => {
                    buffer.push(3);
                    write_checked_string(&mut buffer, text, "ConstantPool")?;
                }
            }
        }

        // 4. Classes
        let class_len = checked_u32_len(self.classes.len(), "Classes")?;
        buffer.extend_from_slice(&class_len.to_le_bytes());
        for cls in &self.classes {
            write_checked_string(&mut buffer, &cls.symbol, "Classes")?;
            buffer.extend_from_slice(&cls.vtable_id.to_le_bytes());
            match cls.base_vtable_id {
                Some(base) => {
                    buffer.push(1);
                    buffer.extend_from_slice(&base.to_le_bytes());
                }
                None => buffer.push(0),
            }
            buffer.extend_from_slice(&(cls.instance_size as u64).to_le_bytes());

            // Reference field offsets
            let ref_len = checked_u32_len(cls.reference_field_offsets.len(), "Classes")?;
            buffer.extend_from_slice(&ref_len.to_le_bytes());
            for &offset in &cls.reference_field_offsets {
                buffer.extend_from_slice(&(offset as u64).to_le_bytes());
            }

            // Weak field offsets
            let weak_len = checked_u32_len(cls.weak_field_offsets.len(), "Classes")?;
            buffer.extend_from_slice(&weak_len.to_le_bytes());
            for &offset in &cls.weak_field_offsets {
                buffer.extend_from_slice(&(offset as u64).to_le_bytes());
            }

            // VTable
            let vtable_len = checked_u32_len(cls.vtable.len(), "Classes")?;
            buffer.extend_from_slice(&vtable_len.to_le_bytes());
            for &func_id in &cls.vtable {
                buffer.extend_from_slice(&func_id.to_le_bytes());
            }

            // ITable
            let itable_len = checked_u32_len(cls.itable.len(), "Classes")?;
            buffer.extend_from_slice(&itable_len.to_le_bytes());
            for (iface_id, methods) in &cls.itable {
                buffer.extend_from_slice(&iface_id.to_le_bytes());
                let method_len = checked_u32_len(methods.len(), "Classes")?;
                buffer.extend_from_slice(&method_len.to_le_bytes());
                for &method_id in methods {
                    buffer.extend_from_slice(&method_id.to_le_bytes());
                }
            }

            // Destructor
            match cls.destructor_id {
                Some(destructor_id) => {
                    buffer.push(1);
                    buffer.extend_from_slice(&destructor_id.to_le_bytes());
                }
                None => buffer.push(0),
            }

            // Field types
            let field_types_len = checked_u32_len(cls.field_types.len(), "Classes")?;
            buffer.extend_from_slice(&field_types_len.to_le_bytes());
            for &(offset, tag) in &cls.field_types {
                buffer.extend_from_slice(&(offset as u64).to_le_bytes());
                buffer.push(tag);
            }

            // Field names (offset, name, type_name)
            let field_names_len = checked_u32_len(cls.field_names.len(), "Classes")?;
            buffer.extend_from_slice(&field_names_len.to_le_bytes());
            for (field_offset, field_name, field_type_name) in &cls.field_names {
                buffer.extend_from_slice(&(*field_offset as u64).to_le_bytes());
                write_checked_string(&mut buffer, field_name, "Classes")?;
                write_checked_string(&mut buffer, field_type_name, "Classes")?;
            }
        }

        // 5. Functions
        let func_len = checked_u32_len(self.functions.len(), "Functions")?;
        buffer.extend_from_slice(&func_len.to_le_bytes());
        for func in &self.functions {
            write_checked_string(&mut buffer, &func.symbol, "Functions")?;
            buffer.extend_from_slice(&func.source_id.to_le_bytes());
            buffer.extend_from_slice(&func.param_count.to_le_bytes());
            buffer.extend_from_slice(&func.num_registers.to_le_bytes());
            buffer.extend_from_slice(&func.static_base_offset.to_le_bytes());

            // Instructions
            let inst_len = checked_u32_len(func.instructions.len(), "Functions")?;
            buffer.extend_from_slice(&inst_len.to_le_bytes());
            for inst in &func.instructions {
                buffer.extend_from_slice(&inst.to_le_bytes());
            }

            // Reference registers
            let ref_reg_len = checked_u32_len(func.reference_registers.len(), "Functions")?;
            buffer.extend_from_slice(&ref_reg_len.to_le_bytes());
            for &reg in &func.reference_registers {
                buffer.extend_from_slice(&reg.to_le_bytes());
            }

            // Exception handlers
            let handler_len = checked_u32_len(func.exception_handlers.len(), "Functions")?;
            buffer.extend_from_slice(&handler_len.to_le_bytes());
            for handler in &func.exception_handlers {
                buffer.extend_from_slice(&handler.try_start_pc.to_le_bytes());
                buffer.extend_from_slice(&handler.try_end_pc.to_le_bytes());
                match &handler.catch_type_symbol {
                    Some(symbol) => {
                        buffer.push(1);
                        write_checked_string(&mut buffer, symbol, "Functions")?;
                    }
                    None => buffer.push(0),
                }
                buffer.extend_from_slice(&handler.catch_reg.to_le_bytes());
                buffer.extend_from_slice(&handler.handler_pc.to_le_bytes());
                match handler.finally_pc {
                    Some(finally_offset) => {
                        buffer.push(1);
                        buffer.extend_from_slice(&finally_offset.to_le_bytes());
                    }
                    None => buffer.push(0),
                }
                let try_ref_len = checked_u32_len(handler.try_ref_regs.len(), "Functions")?;
                buffer.extend_from_slice(&try_ref_len.to_le_bytes());
                for &ref_register in &handler.try_ref_regs {
                    buffer.extend_from_slice(&ref_register.to_le_bytes());
                }
            }

            // Source map
            let source_map_len = checked_u32_len(func.source_map.len(), "Functions")?;
            buffer.extend_from_slice(&source_map_len.to_le_bytes());
            for entry in &func.source_map {
                buffer.extend_from_slice(&entry.instruction_pc.to_le_bytes());
                buffer.extend_from_slice(&entry.span.id.to_le_bytes());
                buffer.extend_from_slice(&entry.span.start.to_le_bytes());
                buffer.extend_from_slice(&entry.span.end.to_le_bytes());
                buffer.extend_from_slice(&entry.line.to_le_bytes());
                buffer.extend_from_slice(&entry.column.to_le_bytes());
            }

            // Local variables
            let local_vars_len = checked_u32_len(func.local_variables.len(), "Functions")?;
            buffer.extend_from_slice(&local_vars_len.to_le_bytes());
            for var_entry in &func.local_variables {
                buffer.extend_from_slice(&var_entry.register.to_le_bytes());
                write_checked_string(&mut buffer, &var_entry.name, "Functions")?;
                write_checked_string(&mut buffer, &var_entry.type_name, "Functions")?;
                buffer.extend_from_slice(&var_entry.start_pc.to_le_bytes());
                buffer.extend_from_slice(&var_entry.end_pc.to_le_bytes());
                buffer.push(if var_entry.is_parameter { 1 } else { 0 });
            }

            // Function visibility & signature
            buffer.push(if func.is_public { 1 } else { 0 });
            let return_type_json = serde_json::to_string(&func.return_type).map_err(|err| {
                EmissionError::CorruptBinaryContainer {
                    section: "Functions".to_string(),
                    details: format!("Failed to serialize return type descriptor: {err}"),
                }
            })?;
            write_checked_string(&mut buffer, &return_type_json, "Functions")?;
            let param_types_len = checked_u32_len(func.parameter_types.len(), "Functions")?;
            buffer.extend_from_slice(&param_types_len.to_le_bytes());
            for parameter_type in &func.parameter_types {
                let parameter_type_json = serde_json::to_string(parameter_type).map_err(|err| {
                    EmissionError::CorruptBinaryContainer {
                        section: "Functions".to_string(),
                        details: format!("Failed to serialize parameter type descriptor: {err}"),
                    }
                })?;
                write_checked_string(&mut buffer, &parameter_type_json, "Functions")?;
            }
        }

        // 6. Metadata and Export Manifest
        write_checked_string(&mut buffer, &self.package_name, "Metadata")?;
        write_checked_string(&mut buffer, &self.package_version, "Metadata")?;
        write_checked_string(&mut buffer, &self.package_description, "Metadata")?;

        let mut manifest_to_serialize = self.export_manifest.clone();
        if manifest_to_serialize.entry_point.is_none() {
            manifest_to_serialize.entry_point = self.entry_point.clone();
        }
        if manifest_to_serialize.build_profile.is_empty() {
            manifest_to_serialize.build_profile = self.build_profile.clone();
        }
        if manifest_to_serialize.native_library_checksums.is_empty() {
            manifest_to_serialize.native_library_checksums = self.native_library_checksums.clone();
        }
        if manifest_to_serialize.dynamic_libraries.is_empty() {
            manifest_to_serialize.dynamic_libraries = self.dynamic_libraries.clone();
        }

        let manifest_bytes = serde_json::to_vec(&manifest_to_serialize).map_err(|err| {
            EmissionError::CorruptBinaryContainer {
                section: "ExportManifest".to_string(),
                details: format!("Failed to serialize export manifest: {err}"),
            }
        })?;
        let manifest_len = checked_u32_len(manifest_bytes.len(), "ExportManifest")?;
        buffer.extend_from_slice(&manifest_len.to_le_bytes());
        buffer.extend_from_slice(&manifest_bytes);

        // 7. Section 7: Exported AST (raw binary bytes)
        let ast_slice = if !self.exported_ast_bytes.is_empty() {
            &self.exported_ast_bytes
        } else {
            &self.export_manifest.exported_ast_bytes
        };
        let ast_len = checked_u32_len(ast_slice.len(), "ExportedAst")?;
        buffer.extend_from_slice(&ast_len.to_le_bytes());
        buffer.extend_from_slice(ast_slice);

        Ok(buffer)
    }

    /// Deserializes a `BinaryProgram` from raw bytes.
    ///
    /// # Errors
    /// Returns `EmissionError` if the byte slice is truncated, corrupted, or incompatible.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, EmissionError> {
        let mut reader = ByteReader::new(bytes);

        // 1. Header
        let magic_bytes = reader.read_bytes(4, "Header")?;
        let mut magic = [0u8; 4];
        magic.copy_from_slice(magic_bytes);
        if &magic != MAGIC {
            return Err(EmissionError::InvalidMagic { found: magic });
        }

        let version_major = reader.read_u16("Header")?;
        let version_minor = reader.read_u16("Header")?;
        if version_major != VERSION_MAJOR {
            return Err(EmissionError::UnsupportedVersion {
                expected_major: VERSION_MAJOR,
                expected_minor: VERSION_MINOR,
                found_major: version_major,
                found_minor: version_minor,
            });
        }

        let flags = reader.read_u32("Header")?;

        let has_entry_point_id = reader.read_flag_bool("Header")?;
        let entry_point_id = if has_entry_point_id {
            Some(reader.read_u32("Header")?)
        } else {
            None
        };

        let has_entry_symbol = reader.read_flag_bool("Header")?;
        let entry_point_symbol = if has_entry_symbol {
            Some(reader.read_string("Header")?)
        } else {
            None
        };

        let static_space_size = reader.read_u32("Header")?;

        // 2. Static Symbols Map
        let static_sym_count = reader.read_u32("StaticSymbols")? as usize;
        let mut static_symbols_map =
            Vec::with_capacity(reader.checked_capacity(static_sym_count, 8, "StaticSymbols")?);
        for _ in 0..static_sym_count {
            let sym = reader.read_string("StaticSymbols")?;
            let slot = reader.read_u32("StaticSymbols")?;
            static_symbols_map.push((sym, slot));
        }

        // 3. Constant Pool
        let pool_count = reader.read_u32("ConstantPool")? as usize;
        let _ = reader.checked_capacity(pool_count, 1, "ConstantPool")?;
        let mut constant_pool = ConstantPool::new();
        for _ in 0..pool_count {
            let tag = reader.read_u8("ConstantPool")?;
            let val = match tag {
                1 => ConstantPoolValue::Int64(reader.read_i64("ConstantPool")?),
                2 => ConstantPoolValue::Float64(reader.read_u64("ConstantPool")?),
                3 => ConstantPoolValue::String(reader.read_string("ConstantPool")?),
                unknown => {
                    return Err(EmissionError::CorruptBinaryContainer {
                        section: "ConstantPool".to_string(),
                        details: format!("Unknown constant tag: {unknown}"),
                    });
                }
            };
            constant_pool.values.push(val);
        }

        // 4. Classes
        let class_count = reader.read_u32("Classes")? as usize;
        let mut classes =
            Vec::with_capacity(reader.checked_capacity(class_count, 32, "Classes")?);
        for _ in 0..class_count {
            let symbol = reader.read_string("Classes")?;
            let vtable_id = reader.read_u32("Classes")?;
            let has_base = reader.read_flag_bool("Classes")?;
            let base_vtable_id = if has_base {
                Some(reader.read_u32("Classes")?)
            } else {
                None
            };
            let instance_size = reader.read_u64_usize("Classes", MAX_INSTANCE_SIZE_BYTES)?;

            let ref_count = reader.read_u32("Classes")? as usize;
            let mut reference_field_offsets =
                Vec::with_capacity(reader.checked_capacity(ref_count, 8, "Classes")?);
            for _ in 0..ref_count {
                reference_field_offsets
                    .push(reader.read_u64_usize("Classes", MAX_INSTANCE_SIZE_BYTES)?);
            }

            let weak_count = reader.read_u32("Classes")? as usize;
            let mut weak_field_offsets =
                Vec::with_capacity(reader.checked_capacity(weak_count, 8, "Classes")?);
            for _ in 0..weak_count {
                weak_field_offsets.push(reader.read_u64_usize("Classes", MAX_INSTANCE_SIZE_BYTES)?);
            }

            let vtable_len = reader.read_u32("Classes")? as usize;
            let mut vtable = Vec::with_capacity(reader.checked_capacity(vtable_len, 4, "Classes")?);
            for _ in 0..vtable_len {
                vtable.push(reader.read_u32("Classes")?);
            }

            let itable_len = reader.read_u32("Classes")? as usize;
            let mut itable = Vec::with_capacity(reader.checked_capacity(itable_len, 8, "Classes")?);
            for _ in 0..itable_len {
                let iface_id = reader.read_u32("Classes")?;
                let method_len = reader.read_u32("Classes")? as usize;
                let mut methods =
                    Vec::with_capacity(reader.checked_capacity(method_len, 4, "Classes")?);
                for _ in 0..method_len {
                    methods.push(reader.read_u32("Classes")?);
                }
                itable.push((iface_id, methods));
            }

            let has_destructor = reader.read_flag_bool("Classes")?;
            let destructor_id = if has_destructor {
                Some(reader.read_u32("Classes")?)
            } else {
                None
            };

            let mut class = CompiledClass::new(symbol, vtable_id, base_vtable_id, instance_size);
            class.reference_field_offsets = reference_field_offsets;
            class.weak_field_offsets = weak_field_offsets;
            class.vtable = vtable;
            class.itable = itable;
            class.destructor_id = destructor_id;

            let field_types_len = reader.read_u32("Classes")? as usize;
            let mut field_types =
                Vec::with_capacity(reader.checked_capacity(field_types_len, 9, "Classes")?);
            for _ in 0..field_types_len {
                let offset = reader.read_u64_usize("Classes", MAX_INSTANCE_SIZE_BYTES)?;
                let tag = reader.read_u8("Classes")?;
                field_types.push((offset, tag));
            }
            class.field_types = field_types;

            let field_names_len = reader.read_u32("Classes")? as usize;
            let mut field_names =
                Vec::with_capacity(reader.checked_capacity(field_names_len, 16, "Classes")?);
            for _ in 0..field_names_len {
                let offset = reader.read_u64_usize("Classes", MAX_INSTANCE_SIZE_BYTES)?;
                let name = reader.read_string("Classes")?;
                let type_name = reader.read_string("Classes")?;
                field_names.push((offset, name, type_name));
            }
            class.field_names = field_names;

            classes.push(class);
        }

        // 5. Functions
        let func_count = reader.read_u32("Functions")? as usize;
        let mut functions =
            Vec::with_capacity(reader.checked_capacity(func_count, 32, "Functions")?);
        for _ in 0..func_count {
            let symbol = reader.read_string("Functions")?;
            let source_id = reader.read_u16("Functions")?;
            let param_count = reader.read_u16("Functions")?;
            let num_registers = reader.read_u16("Functions")?;
            let static_base_offset = reader.read_u32("Functions")?;

            let inst_count = reader.read_u32("Functions")? as usize;
            let mut instructions =
                Vec::with_capacity(reader.checked_capacity(inst_count, 8, "Functions")?);
            for _ in 0..inst_count {
                let bytes = reader.read_bytes(8, "Functions")?;
                let mut inst_bytes = [0u8; 8];
                inst_bytes.copy_from_slice(bytes);
                instructions.push(Instruction::from_le_bytes(inst_bytes));
            }

            let ref_reg_count = reader.read_u32("Functions")? as usize;
            let mut reference_registers =
                Vec::with_capacity(reader.checked_capacity(ref_reg_count, 2, "Functions")?);
            for _ in 0..ref_reg_count {
                reference_registers.push(reader.read_u16("Functions")?);
            }

            let handler_count = reader.read_u32("Functions")? as usize;
            let mut exception_handlers =
                Vec::with_capacity(reader.checked_capacity(handler_count, 20, "Functions")?);
            for _ in 0..handler_count {
                let try_start_pc = reader.read_u32("Functions")?;
                let try_end_pc = reader.read_u32("Functions")?;
                let has_type = reader.read_flag_bool("Functions")?;
                let catch_type_symbol = if has_type {
                    Some(reader.read_string("Functions")?)
                } else {
                    None
                };
                let catch_reg = reader.read_u16("Functions")?;
                let handler_pc = reader.read_u32("Functions")?;
                let has_finally = reader.read_flag_bool("Functions")?;
                let finally_pc = if has_finally {
                    Some(reader.read_u32("Functions")?)
                } else {
                    None
                };

                let try_ref_count = reader.read_u32("Functions")? as usize;
                let mut try_ref_regs =
                    Vec::with_capacity(reader.checked_capacity(try_ref_count, 2, "Functions")?);
                for _ in 0..try_ref_count {
                    try_ref_regs.push(reader.read_u16("Functions")?);
                }

                exception_handlers.push(ExceptionHandlerEntry {
                    try_start_pc,
                    try_end_pc,
                    catch_type_symbol,
                    catch_reg,
                    handler_pc,
                    finally_pc,
                    try_ref_regs,
                });
            }

            let span_count = reader.read_u32("Functions")? as usize;
            let mut source_map =
                Vec::with_capacity(reader.checked_capacity(span_count, 22, "Functions")?);
            for _ in 0..span_count {
                let instruction_pc = reader.read_u32("Functions")?;
                let id = reader.read_u16("Functions")?;
                let start = reader.read_u32("Functions")?;
                let end = reader.read_u32("Functions")?;
                let line = reader.read_u32("Functions")?;
                let column = reader.read_u32("Functions")?;
                source_map.push(SourceMapEntry {
                    instruction_pc,
                    span: Span { id, start, end },
                    line,
                    column,
                });
            }

            let local_vars_len = reader.read_u32("Functions")? as usize;
            let mut local_variables =
                Vec::with_capacity(reader.checked_capacity(local_vars_len, 19, "Functions")?);
            for _ in 0..local_vars_len {
                let register = reader.read_u16("Functions")?;
                let name = reader.read_string("Functions")?;
                let type_name = reader.read_string("Functions")?;
                let start_pc = reader.read_u32("Functions")?;
                let end_pc = reader.read_u32("Functions")?;
                let is_parameter = reader.read_flag_bool("Functions")?;
                local_variables.push(LocalVariableDebugInfo {
                    register,
                    name,
                    type_name,
                    var_type: crate::types::Type::Primitive(crate::types::PrimitiveKind::Void),
                    start_pc,
                    end_pc,
                    is_parameter,
                });
            }

            let is_public = reader.read_flag_bool("Functions")?;
            let return_type_str = reader.read_string("Functions")?;
            let return_type = serde_json::from_str(&return_type_str).map_err(|err| {
                EmissionError::CorruptBinaryContainer {
                    section: "Functions".to_string(),
                    details: format!(
                        "Malformed JSON return type descriptor '{return_type_str}': {err}"
                    ),
                }
            })?;
            let parameter_types_len = reader.read_u32("Functions")? as usize;
            let mut parameter_types =
                Vec::with_capacity(reader.checked_capacity(parameter_types_len, 4, "Functions")?);
            for _ in 0..parameter_types_len {
                let param_type_str = reader.read_string("Functions")?;
                let param_type = serde_json::from_str(&param_type_str).map_err(|err| {
                    EmissionError::CorruptBinaryContainer {
                        section: "Functions".to_string(),
                        details: format!(
                            "Malformed JSON parameter type descriptor '{param_type_str}': {err}"
                        ),
                    }
                })?;
                parameter_types.push(param_type);
            }

            let mut func = CompiledFunction::new(symbol, source_id, param_count, num_registers);
            func.static_base_offset = static_base_offset;
            func.instructions = instructions;
            func.reference_registers = reference_registers;
            func.exception_handlers = exception_handlers;
            func.source_map = source_map;
            func.local_variables = local_variables;
            func.is_public = is_public;
            func.return_type = return_type;
            func.parameter_types = parameter_types;
            functions.push(func);
        }

        // 6. Metadata and Export Manifest
        let (package_name, package_version, package_description, mut export_manifest) =
            if reader.has_remaining() {
                let pkg_name = reader.read_string("Metadata")?;
                let pkg_version = reader.read_string("Metadata")?;
                let pkg_desc = reader.read_string("Metadata")?;
                let manifest_len = reader.read_u32("Metadata")? as usize;
                let manifest_slice = reader.read_bytes(manifest_len, "ExportManifest")?;
                let manifest: ExportManifest =
                    serde_json::from_slice(manifest_slice).map_err(|error| {
                        EmissionError::MalformedExportManifest {
                            reason: error.to_string(),
                        }
                    })?;
                (pkg_name, pkg_version, pkg_desc, manifest)
            } else {
                (
                    String::new(),
                    "0.1.0".to_string(),
                    String::new(),
                    ExportManifest::default(),
                )
            };

        // 7. Section 7: Exported AST
        let exported_ast_bytes = if reader.has_remaining() {
            let ast_len = reader.read_u32("ExportedAst")? as usize;
            reader.read_bytes(ast_len, "ExportedAst")?.to_vec()
        } else {
            export_manifest.exported_ast_bytes.clone()
        };
        export_manifest.exported_ast_bytes = exported_ast_bytes.clone();

        let build_profile = if !export_manifest.build_profile.is_empty() {
            export_manifest.build_profile.clone()
        } else {
            "debug".to_string()
        };
        let native_library_checksums = export_manifest.native_library_checksums.clone();
        let dynamic_libraries = export_manifest.dynamic_libraries.clone();
        let source_files = export_manifest.source_files.clone();

        Ok(BinaryProgram {
            magic,
            version_major,
            version_minor,
            flags,
            entry_point_id,
            entry_point_symbol,
            static_space_size,
            static_symbols_map,
            constant_pool,
            classes,
            functions,
            entry_point: export_manifest.entry_point.clone(),
            package_name,
            package_version,
            package_description,
            build_profile,
            native_library_checksums,
            dynamic_libraries,
            exported_ast_bytes,
            export_manifest,
            source_files,
        })
    }
}

const MAX_INSTANCE_SIZE_BYTES: usize = 64 * 1024 * 1024;

fn checked_u32_len(len: usize, section: &str) -> Result<u32, EmissionError> {
    u32::try_from(len).map_err(|_| EmissionError::CorruptBinaryContainer {
        section: section.to_string(),
        details: format!("Element count or byte length {len} exceeds u32 container limit"),
    })
}

fn write_checked_string(
    buffer: &mut Vec<u8>,
    text: &str,
    section: &str,
) -> Result<(), EmissionError> {
    let bytes = text.as_bytes();
    let len_u32 = checked_u32_len(bytes.len(), section)?;
    buffer.extend_from_slice(&len_u32.to_le_bytes());
    buffer.extend_from_slice(bytes);
    Ok(())
}

/// Safe bounds-checked binary reader preventing integer overflow and buffer over-reads.
struct ByteReader<'buf> {
    data: &'buf [u8],
    offset: usize,
}

impl<'buf> ByteReader<'buf> {
    fn new(data: &'buf [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn has_remaining(&self) -> bool {
        self.offset < self.data.len()
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.offset)
    }

    fn checked_capacity(
        &self,
        count: usize,
        min_bytes_per_elem: usize,
        section: &str,
    ) -> Result<usize, EmissionError> {
        let available = self.remaining();
        let required_min = count.saturating_mul(min_bytes_per_elem);
        if required_min > available {
            return Err(EmissionError::UnexpectedEof {
                section: section.to_string(),
                required: required_min,
                available,
            });
        }
        Ok(count.min(4096))
    }

    fn read_bytes(&mut self, count: usize, section: &str) -> Result<&'buf [u8], EmissionError> {
        let available = self.remaining();
        if count > available {
            return Err(EmissionError::UnexpectedEof {
                section: section.to_string(),
                required: count,
                available,
            });
        }
        let slice = &self.data[self.offset..self.offset + count];
        self.offset += count;
        Ok(slice)
    }

    fn read_u8(&mut self, section: &str) -> Result<u8, EmissionError> {
        let bytes = self.read_bytes(1, section)?;
        Ok(bytes[0])
    }

    fn read_flag_bool(&mut self, section: &str) -> Result<bool, EmissionError> {
        match self.read_u8(section)? {
            0 => Ok(false),
            1 => Ok(true),
            invalid_tag => Err(EmissionError::CorruptBinaryContainer {
                section: section.to_string(),
                details: format!(
                    "Invalid boolean/option flag byte 0x{invalid_tag:02X} (expected 0 or 1)"
                ),
            }),
        }
    }

    fn read_u16(&mut self, section: &str) -> Result<u16, EmissionError> {
        let bytes = self.read_bytes(2, section)?;
        let mut arr = [0u8; 2];
        arr.copy_from_slice(bytes);
        Ok(u16::from_le_bytes(arr))
    }

    fn read_u32(&mut self, section: &str) -> Result<u32, EmissionError> {
        let bytes = self.read_bytes(4, section)?;
        let mut arr = [0u8; 4];
        arr.copy_from_slice(bytes);
        Ok(u32::from_le_bytes(arr))
    }

    fn read_i64(&mut self, section: &str) -> Result<i64, EmissionError> {
        let bytes = self.read_bytes(8, section)?;
        let mut arr = [0u8; 8];
        arr.copy_from_slice(bytes);
        Ok(i64::from_le_bytes(arr))
    }

    fn read_u64(&mut self, section: &str) -> Result<u64, EmissionError> {
        let bytes = self.read_bytes(8, section)?;
        let mut arr = [0u8; 8];
        arr.copy_from_slice(bytes);
        Ok(u64::from_le_bytes(arr))
    }

    fn read_u64_usize(
        &mut self,
        section: &str,
        max_allowed: usize,
    ) -> Result<usize, EmissionError> {
        let raw = self.read_u64(section)?;
        let value = usize::try_from(raw).map_err(|_| EmissionError::CorruptBinaryContainer {
            section: section.to_string(),
            details: format!("Size value {raw} exceeds host pointer width"),
        })?;
        if value > max_allowed {
            return Err(EmissionError::CorruptBinaryContainer {
                section: section.to_string(),
                details: format!("Size value {value} exceeds maximum allowed limit {max_allowed}"),
            });
        }
        Ok(value)
    }

    fn read_string(&mut self, section: &str) -> Result<String, EmissionError> {
        let len = self.read_u32(section)? as usize;
        let bytes = self.read_bytes(len, section)?;
        String::from_utf8(bytes.to_vec()).map_err(|err| EmissionError::CorruptBinaryContainer {
            section: section.to_string(),
            details: format!("Invalid UTF-8 string encoding: {err}"),
        })
    }
}
