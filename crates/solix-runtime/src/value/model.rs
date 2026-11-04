//! Strongly-typed virtual machine runtime values.

#![forbid(unsafe_code)]

use crate::exceptions::RuntimeError;

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

    /// Extracts a boolean value or returns a `TypeMismatch` error.
    pub fn as_bool(&self) -> Result<bool, RuntimeError> {
        match self {
            Self::Bool(b) => Ok(*b),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "bool".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    /// Extracts a 32-bit signed integer or returns a `TypeMismatch` error.
    pub fn as_i32(&self) -> Result<i32, RuntimeError> {
        match self {
            Self::I32(n) => Ok(*n),
            Self::I64(n) => Ok(*n as i32),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "int32".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    /// Extracts a 64-bit signed integer or returns a `TypeMismatch` error.
    pub fn as_i64(&self) -> Result<i64, RuntimeError> {
        match self {
            Self::I64(n) => Ok(*n),
            Self::I32(n) => Ok(*n as i64),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "int64".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    /// Extracts a 64-bit float or returns a `TypeMismatch` error.
    pub fn as_f64(&self) -> Result<f64, RuntimeError> {
        match self {
            Self::F64(f) => Ok(*f),
            Self::I32(n) => Ok(*n as f64),
            Self::I64(n) => Ok(*n as f64),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "float64".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    /// Extracts a heap reference address or returns a `TypeMismatch` error.
    pub fn as_ref(&self) -> Result<u32, RuntimeError> {
        match self {
            Self::Ref(addr) => Ok(*addr),
            Self::Null => Ok(0),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "ref".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    /// Extracts a function pointer or returns a `TypeMismatch` error.
    pub fn as_fn_ptr(&self) -> Result<u32, RuntimeError> {
        match self {
            Self::FnPtr(id) => Ok(*id),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "fn_ptr".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    /// Returns `true` if this value is `Value::Null`.
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Raw 64-bit word representation used for flat byte memory storage.
    #[must_use]
    pub fn to_raw_u64(&self) -> u64 {
        match self {
            Self::Null => 0,
            Self::Bool(b) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            Self::I32(n) => *n as u32 as u64,
            Self::I64(n) => *n as u64,
            Self::F64(f) => f.to_bits(),
            Self::Ref(addr) => *addr as u64,
            Self::FnPtr(id) => *id as u64,
        }
    }
}
