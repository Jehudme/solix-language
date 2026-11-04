//! Source coordinate span model for tracking byte positions across compilation phases.

#![forbid(unsafe_code)]

use miette::SourceSpan;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Source code coordinate span representing an origin file and byte offsets.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct Span {
    /// Identifier indexing the source file in the compiler compilation context.
    pub id: u16,
    /// Absolute start offset in bytes from the beginning of the file (inclusive).
    pub start: u32,
    /// Absolute end offset in bytes from the beginning of the file (exclusive).
    pub end: u32,
}

impl Span {
    /// Creates a new source coordinate span.
    #[must_use]
    pub const fn new(id: u16, start: u32, end: u32) -> Self {
        Self { id, start, end }
    }

    /// Returns the length of the span in bytes.
    #[must_use]
    pub const fn length(&self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    /// Returns whether this span has zero length.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Converts this span into a `miette::SourceSpan` for rich diagnostic reporting.
    #[must_use]
    pub fn to_source_span(&self) -> SourceSpan {
        let length_in_bytes = self.length() as usize;
        SourceSpan::new((self.start as usize).into(), length_in_bytes)
    }

    /// Combines two spans into a single span enclosing both ranges.
    #[must_use]
    pub const fn combine(self, other: Self) -> Self {
        Self {
            id: self.id,
            start: if self.start < other.start {
                self.start
            } else {
                other.start
            },
            end: if self.end > other.end {
                self.end
            } else {
                other.end
            },
        }
    }
}

impl From<Span> for SourceSpan {
    fn from(span: Span) -> Self {
        span.to_source_span()
    }
}

impl Display for Span {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "{}:{}-{}", self.id, self.start, self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_properties() {
        let span = Span::new(1, 10, 25);
        assert_eq!(span.id, 1);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 25);
        assert_eq!(span.length(), 15);
        assert!(!span.is_empty());
        assert_eq!(format!("{span}"), "1:10-25");
    }

    #[test]
    fn test_span_combine() {
        let span1 = Span::new(0, 5, 15);
        let span2 = Span::new(0, 10, 30);
        let combined = span1.combine(span2);
        assert_eq!(combined.start, 5);
        assert_eq!(combined.end, 30);
    }
}
