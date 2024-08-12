use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::collections::HashMap;

#[derive(Debug)]
pub struct FunctionDetails {
    pub name: String,
    pub body: String,
}

#[derive(Debug)]
pub struct ConstDetails {
    pub name: String,
    pub body: String,
}

pub fn extract_functions(file_name: &str) -> Result<Vec<(FunctionDetails)>, String> {
    // Read the file into file_content
    let path = Path::new(file_name);
    let file = File::open(&path).map_err(|e| e.to_string())?;
    let lines = io::BufReader::new(file).lines();
    
    let mut functions = Vec::new();
    let mut inside_function_import_req = false;
    let mut inside_inline_function = false;
    let mut inside_function = false;
    let mut current_function = FunctionDetails {
        name: String::new(),
        body: String::new(),
    };

    let mut nested_braces_count = 0;

    for line in lines {
        let line = line.map_err(|e| e.to_string())?;
        let trimmed_line = line.trim();

        if trimmed_line == "//FUNCTION IMPORT REQ END" {
            inside_function_import_req = false;
        } else if trimmed_line.starts_with("//FUNCTION IMPORT REQ") {
            inside_function_import_req = true;
        } else if trimmed_line.starts_with("//INLINE FUNCTION") {
            inside_inline_function = true;
        } else if trimmed_line == "//END INLINE FUNCTION" {
            inside_inline_function = false;
        } else if trimmed_line.starts_with("fn ") && !inside_function_import_req && !inside_inline_function {
            if inside_function {
                return Err(format!("Nested function detected: {}", trimmed_line));
            }

            // Extract the function name
            let name_end = trimmed_line.find('(').unwrap_or(trimmed_line.len());
            let function_name = trimmed_line[3..name_end].trim().to_string();

            current_function.name = function_name;
            current_function.body = line.clone();
            inside_function = true;
            nested_braces_count += 1;
        } else if inside_function {
            current_function.body.push('\n');
            current_function.body.push_str(&line);
            if trimmed_line.contains("{") {
                nested_braces_count += 1;
            }
            if trimmed_line.contains("}") {
                nested_braces_count -= 1;
                // Close the current function
                if nested_braces_count == 0 {
                    functions.push(current_function);
                    current_function = FunctionDetails {
                        name: String::new(),
                        body: String::new(),
                    };
                    inside_function = false;
                }
                
            }
        }
    }

    if inside_function {
        return Err("Non-terminated function at the end of the file".to_string());
    }

    Ok(functions)
}

pub fn extract_declarations(filepath: &str) -> io::Result<Vec<ConstDetails>> {
    let file = File::open(filepath)?;
    let reader = io::BufReader::new(file);
    let mut declarations = Vec::new();

    let mut inside_function = false;
    let mut inside_inline_const = false;
    let mut brace_level = 0;

    for line in reader.lines() {
        let line = line?;
        let trimmed_line = line.trim();

        // Update brace level
        for char in trimmed_line.chars() {
            match char {
                '{' => brace_level += 1,
                '}' => brace_level -= 1,
                _ => {}
            }
        }

        // Check for start and end of function
        if trimmed_line.starts_with("fn ") && brace_level == 1 {
            inside_function = true;
        } else if inside_function && brace_level == 0 {
            inside_function = false;
        }

        // Check for inline const block
        if trimmed_line.starts_with("//INLINE CONST") {
            inside_inline_const = true;
        } else if trimmed_line.starts_with("//END INLINE CONST") {
            inside_inline_const = false;
        }

        // If outside of function and inline const block, check for variable declaration
        if !inside_function && !inside_inline_const && trimmed_line.starts_with("let ") {
            if let Some((name, _)) = trimmed_line[4..].split_once('=') {
                let name = name.trim().to_string();
                declarations.push(ConstDetails {
                    name: name.clone(),
                    body: line.clone(),
                });
            }
        }
    }

    Ok(declarations)
}

pub fn extract_file(path: &str) -> String {
    let path = Path::new(path);
    let file = File::open(&path).unwrap();
    let reader = io::BufReader::new(file);
    let mut content = String::new();
    for line in reader.lines() {
        content.push_str(&line.unwrap());
        content.push('\n');
    }
    content
}