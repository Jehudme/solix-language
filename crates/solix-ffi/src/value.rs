//! Stable C-ABI tagged value representation and safe marshaling conversions.

#![forbid(unsafe_code)]

/// Type tag indicating a null or uninitialized reference value.
pub const TAG_NULL: u32 = 0;
/// Type tag indicating a boolean scalar value (0 for false, 1 for true).
pub const TAG_BOOL: u32 = 1;
/// Type tag indicating a 32-bit signed integer value.
pub const TAG_I32: u32 = 2;
/// Type tag indicating a 64-bit signed integer value.
pub const TAG_I64: u32 = 3;
/// Type tag indicating a 64-bit IEEE-754 floating-point value.
pub const TAG_F64: u32 = 4;
/// Type tag indicating a reference to an object, string, or array on the ARC heap.
pub const TAG_REF: u32 = 5;
/// Type tag indicating a first-class function pointer or closure address.
pub const TAG_FNPTR: u32 = 6;

/// Stable 16-byte aligned C-ABI tagged value passed across native shared library boundaries.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SolixCValue {
    /// 32-bit type discriminator tag (`TAG_NULL` .. `TAG_FNPTR`).
    pub tag: u32,
    /// Reserved flags and padding bits for memory alignment.
    pub flags: u32,
    /// 64-bit payload storing raw numeric bits or virtual heap reference address.
    pub payload: u64,
}

impl SolixCValue {
    /// Creates a null reference value.
    #[must_use]
    #[inline]
    pub const fn null() -> Self {
        Self {
            tag: TAG_NULL,
            flags: 0,
            payload: 0,
        }
    }

    /// Creates a boolean value.
    #[must_use]
    #[inline]
    pub const fn from_bool(boolean_value: bool) -> Self {
        Self {
            tag: TAG_BOOL,
            flags: 0,
            payload: if boolean_value { 1 } else { 0 },
        }
    }

    /// Creates a 32-bit signed integer value.
    #[must_use]
    #[inline]
    pub const fn from_i32(integer_value: i32) -> Self {
        Self {
            tag: TAG_I32,
            flags: 0,
            payload: (integer_value as i64) as u64,
        }
    }

    /// Creates a 64-bit signed integer value.
    #[must_use]
    #[inline]
    pub const fn from_i64(integer_value: i64) -> Self {
        Self {
            tag: TAG_I64,
            flags: 0,
            payload: integer_value as u64,
        }
    }

    /// Creates a 64-bit floating-point value.
    #[must_use]
    #[inline]
    pub const fn from_f64(float_value: f64) -> Self {
        Self {
            tag: TAG_F64,
            flags: 0,
            payload: float_value.to_bits(),
        }
    }

    /// Creates an object or array reference value at the specified heap address.
    #[must_use]
    #[inline]
    pub const fn from_ref(heap_address: u32) -> Self {
        Self {
            tag: TAG_REF,
            flags: 0,
            payload: heap_address as u64,
        }
    }

    /// Unpacks the value as a boolean. Returns `false` if tag does not match.
    #[must_use]
    #[inline]
    pub const fn as_bool(self) -> bool {
        self.tag == TAG_BOOL && self.payload != 0
    }

    /// Unpacks the value as a 32-bit signed integer.
    #[must_use]
    #[inline]
    pub const fn as_i32(self) -> i32 {
        self.payload as i64 as i32
    }

    /// Unpacks the value as a 64-bit signed integer.
    #[must_use]
    #[inline]
    pub const fn as_i64(self) -> i64 {
        self.payload as i64
    }

    /// Unpacks the value as a 64-bit IEEE-754 floating-point number.
    #[must_use]
    #[inline]
    pub fn as_f64(self) -> f64 {
        f64::from_bits(self.payload)
    }

    /// Unpacks the value as an object or array virtual heap reference address.
    #[must_use]
    #[inline]
    pub const fn as_ref(self) -> u32 {
        self.payload as u32
    }

    /// Checks if this value represents a null reference.
    #[must_use]
    #[inline]
    pub const fn is_null(self) -> bool {
        self.tag == TAG_NULL || (self.tag == TAG_REF && self.payload == 0)
    }
}

impl Default for SolixCValue {
    fn default() -> Self {
        Self::null()
    }
}
