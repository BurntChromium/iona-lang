//! Handles code generation for the C language target

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Error, Write};

use crate::parse::FunctionData;

/// Emits a compact, function-signature-only header file
pub fn emit_c_header(function_table: &BTreeMap<String, FunctionData>) -> Result<(), Error> {
    // Construct the header file string
    let mut buffer_str: String = "#include <stdbool.h>\n\n".to_string();
    for (name, data) in function_table {
        let mut definition: String = "".to_string();
        // Start with return type
        definition += data.return_type.to_str();
        // Add fn name
        definition += &format!(" {name}(");
        // Add arguments
        if data.args.len() == 0 {
            definition += ");";
        }
        for (index, arg) in data.args.iter().enumerate() {
            definition += &format!("{} {}", arg.data_type.to_str(), arg.name);
            // Comma separate all but the last argument
            if index + 1 < data.args.len() {
                definition += ", ";
            } else {
                definition += ");";
            }
        }
        // Push this fn to the buffer
        buffer_str += &definition;
        buffer_str += "\n\n";
    }
    // Write to a file
    let path = "./codegen/iona_generated_header.h";
    println!("writing C header file to {path}");
    let mut output = File::create(path)?;
    write!(output, "{}", buffer_str)
}
