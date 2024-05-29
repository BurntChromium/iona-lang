//! Function Properties provide syntactic metadata and hints to the compiler

/// Tagged function properties
///
/// Pure == no side effects
/// Public == visible within this module
/// Export == visible within this module AND visible to other modules
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Properties {
    Pure,
    Public,
    Export,
}

/// For error messages
pub const PROPERTY_LIST: [&str; 3] = ["Pure", "Public", "Export"];
