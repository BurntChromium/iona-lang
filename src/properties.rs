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

pub const PROPERTY_LIST: [&'static str; 3] = ["Pure", "Public", "Export"];
