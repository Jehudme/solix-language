//! String conversion and display formatting for runtime values.

#![forbid(unsafe_code)]

use crate::value::model::Value;
use std::fmt;

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(formatter, "null"),
            Self::Bool(flag) => write!(formatter, "{flag}"),
            Self::I32(int_val) => write!(formatter, "{int_val}"),
            Self::I64(int_val) => write!(formatter, "{int_val}"),
            Self::F64(float_val) => {
                if float_val.fract() == 0.0 && !float_val.is_infinite() && !float_val.is_nan() {
                    write!(formatter, "{float_val:.1}")
                } else {
                    write!(formatter, "{float_val}")
                }
            }
            Self::Ref(addr) => write!(formatter, "<ref 0x{addr:08X}>"),
            Self::FnPtr(id) => write!(formatter, "<fn_ptr #{id}>"),
        }
    }
}
