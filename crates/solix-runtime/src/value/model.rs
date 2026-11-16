//! Strongly-typed virtual machine runtime values.

#![forbid(unsafe_code)]

use crate::exceptions::RuntimeError;

#[cold]
#[inline(never)]
fn type_mismatch_error(expected: &'static str, found_value: &Value) -> RuntimeError {
    RuntimeError::TypeMismatch {
        expected: expected.to_string(),
        found: found_value.type_name().to_string(),
    }
}

/// Strongly-typed 64-bit tagged value representation for register and heap variables.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Value {
    /// Null reference or uninitialized value.
    #[default]
    Null,
    /// Boolean truth value.
    Bool(bool),
    /// 32-bit signed two's complement integer.
    I32(i32),
    /// 64-bit signed two's complement integer.
    I64(i64),
    /// IEEE-754 64-bit double precision floating point number.
    F64(f64),
    /// 32-bit byte address of a reference-counted dynamic heap payload.
    Ref(u32),
    /// 32-bit global function index or function pointer.
    FnPtr(u32),
}

impl Value {
    /// Returns the static human-readable type name of this value variant.
    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Bool(_) => "bool",
            Self::I32(_) => "int32",
            Self::I64(_) => "int64",
            Self::F64(_) => "float64",
            Self::Ref(_) => "ref",
            Self::FnPtr(_) => "fn_ptr",
        }
    }

    /// Evaluates truthiness according to Solix semantic rules:
    /// - `Null` is false
    /// - `Bool(b)` is `b`
    /// - `I32(n)` is `n != 0`
    /// - `I64(n)` is `n != 0`
    /// - `F64(f)` is `!f.is_nan() && f != 0.0`
    /// - `Ref(addr)` is `addr != 0`
    /// - `FnPtr(id)` is true
    #[inline(always)]
    #[must_use]
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Bool(b) => *b,
            Self::I32(n) => *n != 0,
            Self::I64(n) => *n != 0,
            Self::F64(f) => !f.is_nan() && *f != 0.0,
            Self::Ref(addr) => *addr != 0,
            Self::FnPtr(_) => true,
        }
    }

    /// Evaluates equality between two values with support for mixed primitive/reference types.
    #[inline(always)]
    #[must_use]
    pub fn is_equal_to(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Ref(target_address), Self::Null) | (Self::Null, Self::Ref(target_address)) => {
                *target_address == 0
            }
            (Self::Ref(first_address), Self::Ref(second_address)) => {
                first_address == second_address
            }
            (Self::I32(first_int), Self::I32(second_int)) => first_int == second_int,
            (Self::I64(first_long), Self::I64(second_long)) => first_long == second_long,
            (Self::I32(first_int), Self::I64(second_long)) => {
                (*first_int as i64) == *second_long || (*first_int as u32 as i64) == *second_long
            }
            (Self::I64(first_long), Self::I32(second_int)) => {
                *first_long == (*second_int as i64) || *first_long == (*second_int as u32 as i64)
            }
            (Self::Bool(first_bool), Self::Bool(second_bool)) => first_bool == second_bool,
            (Self::F64(first_float), Self::F64(second_float)) => first_float == second_float,
            (Self::FnPtr(first_fn), Self::FnPtr(second_fn)) => first_fn == second_fn,
            _ => false,
        }
    }

    /// Extracts a boolean value or returns a `TypeMismatch` error.
    #[inline(always)]
    pub fn as_bool(&self) -> Result<bool, RuntimeError> {
        match self {
            Self::Bool(boolean_value) => Ok(*boolean_value),
            _ => Err(type_mismatch_error("bool", self)),
        }
    }

    /// Extracts a 32-bit signed integer or returns a `TypeMismatch` error.
    #[inline(always)]
    pub fn as_i32(&self) -> Result<i32, RuntimeError> {
        match self {
            Self::I32(primitive_number) => Ok(*primitive_number),
            Self::I64(primitive_number) => Ok(*primitive_number as i32),
            _ => Err(type_mismatch_error("int32", self)),
        }
    }

    /// Extracts a 64-bit signed integer or returns a `TypeMismatch` error.
    #[inline(always)]
    pub fn as_i64(&self) -> Result<i64, RuntimeError> {
        match self {
            Self::I64(number_val) => Ok(*number_val),
            Self::I32(number_val) => Ok(*number_val as i64),
            _ => Err(type_mismatch_error("int64", self)),
        }
    }

    /// Extracts a 64-bit float or returns a `TypeMismatch` error.
    #[inline(always)]
    pub fn as_f64(&self) -> Result<f64, RuntimeError> {
        match self {
            Self::F64(float_val) => Ok(*float_val),
            Self::I32(number_val) => Ok(*number_val as f64),
            Self::I64(number_val) => Ok(*number_val as f64),
            _ => Err(type_mismatch_error("float64", self)),
        }
    }

    /// Extracts a heap reference address or returns a `TypeMismatch` error.
    #[inline(always)]
    pub fn as_ref(&self) -> Result<u32, RuntimeError> {
        match self {
            Self::Ref(heap_address) => Ok(*heap_address),
            Self::Null => Ok(0),
            _ => Err(type_mismatch_error("ref", self)),
        }
    }

    /// Extracts a function pointer or returns a `TypeMismatch` error.
    #[inline(always)]
    pub fn as_fn_ptr(&self) -> Result<u32, RuntimeError> {
        match self {
            Self::FnPtr(func_id) => Ok(*func_id),
            _ => Err(type_mismatch_error("fn_ptr", self)),
        }
    }

    /// Returns `true` if this value is `Value::Null`.
    #[inline(always)]
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub const TAG_NULL: u64 = 0x7FFC_0000_0000_0000;
    pub const TAG_BOOL: u64 = 0x7FFD_0000_0000_0000;
    pub const TAG_I32: u64 = 0x7FFE_0000_0000_0000;
    pub const TAG_REF: u64 = 0x7FFF_0000_0000_0000;
    pub const TAG_FNPTR: u64 = 0x7FFB_0000_0000_0000;
    pub const TAG_MASK: u64 = 0xFFFF_0000_0000_0000;
    pub const CANONICAL_NAN: u64 = 0x7FF8_0000_0000_0000;

    /// Returns the raw unwrapped 64-bit integer or bit pattern for static memory storage.
    #[inline(always)]
    #[must_use]
    pub fn to_primitive_u64(&self) -> u64 {
        match self {
            Self::Null => 0,
            Self::Bool(is_true) => {
                if *is_true {
                    1
                } else {
                    0
                }
            }
            Self::I32(int_val) => *int_val as u32 as u64,
            Self::I64(int_val) => *int_val as u64,
            Self::F64(float_val) => float_val.to_bits(),
            Self::Ref(addr) => *addr as u64,
            Self::FnPtr(func_id) => *func_id as u64,
        }
    }

    /// Encodes this value into a 2-word (16-byte) tagged representation: `(tag, payload)`.
    #[must_use]
    pub fn to_tagged_parts(&self) -> (u64, u64) {
        match self {
            Self::Null => (0, 0),
            Self::Bool(is_boolean) => (1, if *is_boolean { 1 } else { 0 }),
            Self::I32(integer_val) => (2, *integer_val as u32 as u64),
            Self::I64(integer_val) => (3, *integer_val as u64),
            Self::F64(float_val) => (4, float_val.to_bits()),
            Self::Ref(address_val) => (5, *address_val as u64),
            Self::FnPtr(function_id) => (6, *function_id as u64),
        }
    }

    /// Decodes a value from a 2-word (16-byte) tagged representation.
    #[must_use]
    pub fn from_tagged_parts(tag_id: u64, payload_word: u64) -> Self {
        match tag_id {
            1 => Self::Bool((payload_word & 1) != 0),
            2 => Self::I32(payload_word as i32),
            3 => Self::I64(payload_word as i64),
            4 => Self::F64(f64::from_bits(payload_word)),
            5 => {
                let address_val = payload_word as u32;
                if address_val == 0 {
                    Self::Null
                } else {
                    Self::Ref(address_val)
                }
            }
            6 => Self::FnPtr(payload_word as u32),
            _ => Self::Null,
        }
    }

    /// Encodes this value into a raw 64-bit NaN-boxed word representation for memory storage.
    #[inline(always)]
    #[must_use]
    pub fn to_raw_u64(&self) -> u64 {
        match self {
            Self::Null => Self::TAG_NULL,
            Self::Bool(is_true) => Self::TAG_BOOL | (*is_true as u64),
            Self::I32(int_val) => Self::TAG_I32 | (*int_val as u32 as u64),
            Self::Ref(addr) => {
                if *addr == 0 {
                    Self::TAG_NULL
                } else {
                    Self::TAG_REF | (*addr as u64)
                }
            }
            Self::FnPtr(func_id) => Self::TAG_FNPTR | (*func_id as u64),
            Self::F64(float_val) => {
                if float_val.is_nan() {
                    Self::CANONICAL_NAN
                } else {
                    float_val.to_bits()
                }
            }
            Self::I64(int_val) => {
                if *int_val >= i32::MIN as i64 && *int_val <= i32::MAX as i64 {
                    Self::TAG_I32 | (*int_val as i32 as u32 as u64)
                } else {
                    *int_val as u64
                }
            }
        }
    }

    /// Decodes a value from its raw 64-bit NaN-boxed word representation.
    #[inline(always)]
    #[must_use]
    pub fn from_raw_u64(raw: u64) -> Self {
        if raw == 0 || raw == Self::TAG_NULL {
            Self::Null
        } else if (raw & Self::TAG_MASK) == Self::TAG_BOOL {
            Self::Bool((raw & 1) != 0)
        } else if (raw & Self::TAG_MASK) == Self::TAG_I32 {
            Self::I32(raw as i32)
        } else if (raw & Self::TAG_MASK) == Self::TAG_REF {
            let addr = raw as u32;
            if addr == 0 {
                Self::Null
            } else {
                Self::Ref(addr)
            }
        } else if (raw & Self::TAG_MASK) == Self::TAG_FNPTR {
            Self::FnPtr(raw as u32)
        } else if raw == Self::CANONICAL_NAN {
            Self::F64(f64::NAN)
        } else if (raw & 0x7FF8_0000_0000_0000) != 0x7FF8_0000_0000_0000 {
            Self::F64(f64::from_bits(raw))
        } else {
            Self::I64(raw as i64)
        }
    }
}
