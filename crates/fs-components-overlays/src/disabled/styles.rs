//! The style sheet of a disabled control.
//!
//! Pattern: none - only theme variables (constants, no behaviour).

/// CSS of a disabled control: the content gets weak, the size stays.
pub const STYLES: &str = r"
.fs-disabled { opacity: 0.6; cursor: default; }
";
