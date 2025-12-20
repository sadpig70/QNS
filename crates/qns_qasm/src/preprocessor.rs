use crate::QasmError;
use std::fs;
use std::path::Path;

/// Recursively resolves `include` statements in QASM code.
///
/// # Arguments
/// * `input` - The QASM code string.
/// * `base_path` - The base path to resolve relative includes from.
///
/// # Returns
/// * `Result<String, QasmError>` - The expanded QASM code.
pub fn resolve_includes(input: &str, base_path: &Path) -> Result<String, QasmError> {
    let mut output = String::new();

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("include") {
            // Parse include "filename";
            // Simple parsing: split by quotes
            let parts: Vec<&str> = trimmed.split('"').collect();
            if parts.len() >= 2 {
                let filename = parts[1];
                let include_path = base_path.join(filename);

                if include_path.exists() {
                    let content = fs::read_to_string(&include_path).map_err(|e| {
                        QasmError::ParseError(format!(
                            "Failed to read include file {}: {}",
                            filename, e
                        ))
                    })?;

                    // Recursive call to handle nested includes
                    // We use the parent of the included file as the new base path
                    let new_base = include_path.parent().unwrap_or(Path::new("."));
                    let expanded = resolve_includes(&content, new_base)?;
                    output.push_str(&expanded);
                    output.push('\n');
                } else {
                    return Err(QasmError::ParseError(format!(
                        "Include file not found: {}",
                        include_path.display()
                    )));
                }
            } else {
                // Malformed include, just keep it (or error)
                output.push_str(line);
                output.push('\n');
            }
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }

    Ok(output)
}
