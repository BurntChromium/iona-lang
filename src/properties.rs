//! Properties

/// Tagged function properties
///
/// Pure == no side effects
/// Public == visible within this module
/// Export == visible within this module AND visible to other modules
pub enum Properties {
    Pure,
    Public,
    Export,
}

/// For error messages
pub const PROPERTY_LIST: [&str; 3] = ["Pure", "Public", "Export"];
