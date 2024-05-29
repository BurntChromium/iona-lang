//! Function Permissions increase program security

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permissions {
    ReadFile,
    WriteFile,
    ReadNetwork,
    WriteNetwork,
    Custom,
}

impl Permissions {
    pub fn from_str(input: &str) -> Self {
        match input {
            "ReadFile" => Self::ReadFile,
            "WriteFile" => Self::WriteFile,
            "ReadNetwork" => Self::ReadNetwork,
            "WriteNetwork" => Self::WriteNetwork,
            _ => Self::Custom,
        }
    }
}
