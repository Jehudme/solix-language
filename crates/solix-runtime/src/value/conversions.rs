//! String conversion and display formatting for runtime values.

#![forbid(unsafe_code)]

use crate::value::model::Value;
use std::fmt;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::I32(n) => write!(f, "{n}"),
            Self::I64(n) => write!(f, "{n}"),
            Self::F64(v) => {
                if v.fract() == 0.0 && !v.is_infinite() && !v.is_nan() {
                    write!(f, "{v:.1}")
                } else {
                    write!(f, "{v}")
                }
            }
            Self::Ref(addr) => write!(f, "<ref 0x{addr:08X}>"),
            Self::FnPtr(id) => write!(f, "<fn_ptr #{id}>"),
        }
    }
}
