//! Handles code generation for the C language target

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Error, Write};

use crate::parse::{FunctionData, PrimitiveDataType};

/// Emits a compact, function-signature-only header file
pub fn emit_c_header(function_table: BTreeMap<String, FunctionData>) -> Result<(), Error> {
    // Construct the header file string
    let mut buffer_str: String = String::new();
    for (name, data) in function_table {
        let mut definition: String = "".to_string();
        match data.return_type {
            PrimitiveDataType::Void => definition += "void",
            PrimitiveDataType::Bool => definition += "bool",
            PrimitiveDataType::Int => definition += "int",
            PrimitiveDataType::Float => definition += "float",
            PrimitiveDataType::Str => definition += "char",
        }
        buffer_str += &definition;
        buffer_str += "\n";
    }
    // Write to a file
    let path = "/codegen/iona_generated_header.h";
    let mut output = File::create(path)?;
    write!(output, "{}", buffer_str)
}
