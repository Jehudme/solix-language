//! Memory block layouts, 8-byte header packing, and object tag metadata.

#![forbid(unsafe_code)]

use crate::exceptions::RuntimeError;

/// Size of the 8-byte allocation block header preceding all heap payloads.
pub const HEADER_SIZE: usize = 8;

/// Minimum allocation alignment in bytes (64-bit word alignment).
pub const ALIGNMENT: usize = 8;

/// Maximum allowable reference count (26-bit unsigned integer).
pub const MAX_REF_COUNT: u32 = 0x03FF_FFFF;

/// Four-color state tags for the Bacon & Rajan (2001) cycle collection algorithm.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum GcColor {
    /// In use or free block (Black).
    #[default]
    Black = 0,
    /// Possible member of cycle / trial deletion in progress (Gray).
    Gray = 1,
    /// Confirmed member of unreachable cycle (White).
    White = 2,
    /// Possible root of cycle (Purple).
    Purple = 3,
}

impl GcColor {
    /// Decodes 2 bits into a `GcColor`.
    #[must_use]
    pub const fn from_bits(bits: u8) -> Self {
        match bits & 0x03 {
            0 => Self::Black,
            1 => Self::Gray,
            2 => Self::White,
            3 => Self::Purple,
            _ => Self::Black,
        }
    }

    /// Converts this color tag into an 8-bit unsigned byte.
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Returns whether a block tag can contain outgoing reference pointers to other heap blocks.
#[must_use]
pub const fn can_contain_references(tag: BlockTag) -> bool {
    matches!(
        tag,
        BlockTag::Object | BlockTag::ArrayRef | BlockTag::Tuple | BlockTag::Closure
    )
}

/// Strongly-typed 4-bit heap allocation category tag.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BlockTag {
    /// Raw unformatted byte buffer.
    Raw = 0x00,
    /// Object instance with virtual method table and fields.
    Object = 0x01,
    /// Homogeneous byte array (`uint8[]`).
    ArrayU8 = 0x02,
    /// Homogeneous 32-bit integer array (`int32[]`).
    ArrayI32 = 0x03,
    /// Homogeneous 64-bit integer array (`int64[]`).
    ArrayI64 = 0x04,
    /// Homogeneous 64-bit float array (`float64[]`).
    ArrayF64 = 0x05,
    /// Dynamic reference array holding object or collection pointers.
    ArrayRef = 0x06,
    /// Heterogeneous tuple structure.
    Tuple = 0x07,
    /// Closure environment with function ID and captured local values.
    Closure = 0x08,
    /// Heap-allocated immutable UTF-8 string.
    String = 0x09,
    /// Object instance containing only Plain-Old-Data (POD) fields without destructors or reference fields.
    ObjectPod = 0x0A,
}

impl BlockTag {
    /// Decodes a 4-bit nibble into a `BlockTag`.
    #[must_use]
    pub fn from_nibble(nibble: u8) -> Self {
        match nibble & 0x0F {
            0x00 => Self::Raw,
            0x01 => Self::Object,
            0x02 => Self::ArrayU8,
            0x03 => Self::ArrayI32,
            0x04 => Self::ArrayI64,
            0x05 => Self::ArrayF64,
            0x06 => Self::ArrayRef,
            0x07 => Self::Tuple,
            0x08 => Self::Closure,
            0x09 => Self::String,
            0x0A => Self::ObjectPod,
            _ => Self::Raw,
        }
    }

    /// Converts this tag into an 8-bit unsigned byte.
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }
}

/// 8-byte metadata header positioned immediately prior to each dynamic payload.
///
/// Bit layout:
/// - Bytes 0..4: `size` (32-bit unsigned payload size or element count)
/// - Bytes 4..8: `tag_and_ref` (Bits 28..32: `BlockTag`, Bits 26..28: `GcColor`, Bits 0..26: `ref_count`)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BlockHeader {
    /// Payload capacity in bytes (or element count for arrays).
    pub size: u32,
    /// Memory block classification tag.
    pub tag: BlockTag,
    /// Bacon & Rajan 4-color state tag for cycle collection.
    pub color: GcColor,
    /// Automatic reference count (max 67,108,863).
    pub ref_count: u32,
}

impl BlockHeader {
    /// Creates a new `BlockHeader` initialized with color `GcColor::Black`.
    #[must_use]
    pub fn new(size: u32, tag: BlockTag, ref_count: u32) -> Self {
        Self {
            size,
            tag,
            color: GcColor::Black,
            ref_count: ref_count.min(MAX_REF_COUNT),
        }
    }

    /// Creates a new `BlockHeader` with an explicit `GcColor`.
    #[must_use]
    pub fn with_color(size: u32, tag: BlockTag, color: GcColor, ref_count: u32) -> Self {
        Self {
            size,
            tag,
            color,
            ref_count: ref_count.min(MAX_REF_COUNT),
        }
    }

    /// Serializes the header into an 8-byte array.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut bytes = [0u8; HEADER_SIZE];
        let tag_nibble = (self.tag.to_u8() & 0x0F) as u32;
        let color_bits = (self.color.to_u8() & 0x03) as u32;
        let tag_and_ref =
            (tag_nibble << 28) | (color_bits << 26) | (self.ref_count & MAX_REF_COUNT);

        bytes[0..4].copy_from_slice(&self.size.to_le_bytes());
        bytes[4..8].copy_from_slice(&tag_and_ref.to_le_bytes());
        bytes
    }

    /// Deserializes a `BlockHeader` from an 8-byte array.
    #[must_use]
    pub fn from_bytes(bytes: [u8; HEADER_SIZE]) -> Self {
        let size = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let tag_and_ref = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

        let tag_nibble = ((tag_and_ref >> 28) & 0x0F) as u8;
        let color_bits = ((tag_and_ref >> 26) & 0x03) as u8;
        let ref_count = tag_and_ref & MAX_REF_COUNT;
        let tag = BlockTag::from_nibble(tag_nibble);
        let color = GcColor::from_bits(color_bits);

        Self {
            size,
            tag,
            color,
            ref_count,
        }
    }

    /// Increments the reference count.
    ///
    /// If the block is marked immortal (`ref_count == MAX_REF_COUNT`), the reference count is unchanged.
    ///
    /// # Errors
    /// Returns `RuntimeError::TypeMismatch` if the reference counter overflows `MAX_REF_COUNT`.
    pub fn inc_ref(&mut self) -> Result<u32, RuntimeError> {
        if self.ref_count == MAX_REF_COUNT {
            return Ok(MAX_REF_COUNT);
        }
        if self.ref_count >= MAX_REF_COUNT - 1 {
            return Err(RuntimeError::TypeMismatch {
                expected: "ref count < MAX_REF_COUNT".to_string(),
                found: format!("ref count overflowed {}", self.ref_count),
            });
        }
        self.ref_count += 1;
        Ok(self.ref_count)
    }

    /// Decrements the reference count by 1.
    ///
    /// If the block is marked immortal (`ref_count == MAX_REF_COUNT`), the reference count is unchanged.
    #[must_use]
    pub fn dec_ref(&mut self) -> u32 {
        if self.ref_count == MAX_REF_COUNT {
            return MAX_REF_COUNT;
        }
        if self.ref_count > 0 {
            self.ref_count -= 1;
        }
        self.ref_count
    }
}

/// Aligns a byte size up to the nearest multiple of `ALIGNMENT` (8 bytes).
#[must_use]
pub const fn align_up(size: usize) -> usize {
    (size + ALIGNMENT - 1) & !(ALIGNMENT - 1)
}
