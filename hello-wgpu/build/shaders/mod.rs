use std::fmt::Display;
use std::path::Path;
use std::io;
use std::fs::File;
use std::collections::HashMap;
use fundamentals::bitpacking_spec::BitpackingSpec;
use std::io::BufRead;
use std::io::Read;
use std::cmp::Ordering;

use regex::Regex;

pub mod new_parser;
pub mod extract_functions;
pub mod build_model;
pub mod translate_artifact;

pub fn preprocess_inline_const_u32(template_file: &str, name: &str, input: u32) -> String {
    let type_name = "u32";
    return preprocess_inline_const(template_file, name, type_name, input);
}

pub fn preprocess_inline_const_i32(template_file: &str, name: &str, input: i32) -> String {
    let type_name = "i32";
    return preprocess_inline_const(template_file, name, type_name, input);
}

fn preprocess_inline_const<T: Display>(template_file: &str, name: &str, type_name: &str, input: T) -> String {
    // Define the placeholders
    let start_placeholder = format!("//INLINE CONST {type_name} {name}");
    println!("{}", start_placeholder);
    let end_placeholder = "//END INLINE CONST";

    // Find the positions of the placeholders
    if let Some(start_pos) = template_file.find(start_placeholder.as_str()) {
        if let Some(end_pos) = template_file.find(end_placeholder) {
            // Create the replacement string
            let replacement = format!("let {}: {} = {}{};", name, type_name, input, if (type_name == "u32") { "u" } else {""});

            // Replace the placeholder in the input string
            let mut result = String::new();
            result.push_str(&template_file[..start_pos]);
            result.push_str(&replacement);
            result.push_str(&template_file[end_pos + end_placeholder.len()..]);

            return result;
        }
    }

    // If placeholders are not found, return the original string
    panic!("String not found in preprocessing!");
}

pub fn read_bitpacking_specs<P: AsRef<Path>>(file_path: P) -> io::Result<Vec<BitpackingSpec>> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);
    
    let mut specs = Vec::new();
    
    for line in reader.lines() {
        let line = line?;
        if let Ok(spec) = line.parse::<BitpackingSpec>() {
            specs.push(spec);
        }
    }
    
    Ok(specs)
}

pub fn preprocess_inline_model_extract<P: AsRef<Path>>(template_file: &str, name: &str, spec_file_name: P) -> io::Result<String> {
    let specs = match read_bitpacking_specs(spec_file_name)
    {
        Ok(specs) => specs,
        Err(_) => panic!("Reading the bitpacking specs didn't go according to plan")
    };
    let mut template = String::new();
    let mut in_model_extract = false;
    let mut generated_code = String::new();

    // Group specs by data_name
    let mut spec_map: HashMap<String, Vec<BitpackingSpec>> = HashMap::new();
    for spec in specs {
        spec_map.entry(spec.data_name.clone()).or_default().push(spec);
    }

    let mut num_bits = 0;

    for line in template_file.lines() {
        
        if line.trim() == format!("// INLINE MODEL EXTRACT {}", name) {
            in_model_extract = true;
            continue;
        }
        
        if line.trim() == "// END INLINE MODEL EXTRACT" {
            in_model_extract = false;
            template.push_str(&generated_code);
            continue;
        }
        
        if in_model_extract {
            if line.trim().starts_with("// U32") || line.trim().starts_with("// I32") {
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.len() == 4 {
                    let shader_type = parts[1];
                    let shader_name = parts[2];
                    let spec_name = parts[3];
                    
                    if let Some(specs) = spec_map.get(spec_name) {
                        // Sort specs by bit_subslice_start
                        let mut sorted_specs = specs.clone();
                        sorted_specs.sort_by_key(|s| s.bit_subslice_start.unwrap());

                        let mut combined_code = String::new();
                        for spec in sorted_specs {
                            let bit_mask = (1u32 << spec.bit_length) - 1;
                            let bit_mask = bit_mask << spec.bit_start;
                            let bit_mask = if shader_type == "U32" {
                                format!("{}u", bit_mask)
                            } else {
                                bit_mask.to_string()
                            };

                            let shift_amount = spec.bit_start;
                            let shift_code = if shift_amount > 0 {
                                format!(" >> {}{}", shift_amount, if shader_type == "U32" { "u" } else { "" })
                            } else {
                                String::new()
                            };

                            if !combined_code.is_empty() {
                                combined_code.push_str(" | ");
                            }
                            combined_code.push_str(&format!(
                                "(model.data{} & {}){}",
                                num_bits / 32,
                                bit_mask,
                                shift_code
                            ));

                            num_bits += spec.bit_length;
                        }

                        let var_declaration = format!(
                            "\tvar {} = {};",
                            shader_name,
                            combined_code
                        );

                        generated_code.push_str(&var_declaration);
                        generated_code.push('\n');
                    }
                }
            }
        } else {
            template.push_str(&line);
            template.push('\n');
        }
    }
    
    Ok(template)
}

pub fn process_inline_vertex_struct(template: &str, name: &str, specs: &[BitpackingSpec]) -> io::Result<String> {
    let mut output = String::new();
    let mut in_vertex_struct = false;
    let mut shader_name = String::new();

    for line in template.lines() {
        if line.trim().starts_with(&format!("//INLINE VERTEX_STRUCT")) {
            let parts: Vec<&str> = line.trim().split_whitespace().collect();
            if parts.len() == 4 && parts[3] == name {
                shader_name = parts[2].to_string();
                in_vertex_struct = true;
                continue;
            }
        }
        
        if line.trim() == "//END INLINE VERTEX_STRUCT" && in_vertex_struct {
            in_vertex_struct = false;

            let mut field_map: HashMap<usize, Vec<&BitpackingSpec>> = HashMap::new();
            
            for spec in specs {
                let start = spec.bit_start / 32;
                field_map.entry(start).or_default().push(spec);
            }
            
            let mut fields = Vec::new();
            let mut locations = field_map.keys().cloned().collect::<Vec<usize>>();
            locations.sort();
            
            for loc in locations {
                fields.push(format!("@location({}) data{}: u32", loc, loc));
            }
            
            output.push_str(&format!("struct {} {{\n", shader_name));
            for field in fields {
                output.push_str(&format!("    {},\n", field));
            }
            output.push_str("};\n");
            output.push('\n');
            continue;
        }
        
        if !in_vertex_struct {
            output.push_str(line);
            output.push('\n');
        }
    }
    
    Ok(output)
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileReq {
    pub filename: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShaderReq {
    name: String,
    buffer_type: String,
    shader_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFunctionReq {
    pub function_name: String,
    pub filename: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Requirement {
    LocalFunction(String),
    LocalConst(String),
    CameraReq,
    FileReq(FileReq),
    GlobalFunctionReq(GlobalFunctionReq),
    ShaderReq(ShaderReq),
}

pub fn get_requirement_number(req: Requirement) -> usize {
    match req {
        Requirement::FileReq(_) => 0,
        Requirement::LocalConst(_) => 1,
        Requirement::LocalFunction(_) => 2,
        _ => 3
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionImportReq {
    pub function_name: String,
    pub function_filename: String,
    pub requirements: Vec<Requirement>,
}

impl PartialOrd for FunctionImportReq {
    fn partial_cmp(&self, other: &FunctionImportReq) -> Option<Ordering> {
        if self == other {
            return Some(Ordering::Equal);
        }
        for req in &self.requirements {
            match req {
                Requirement::LocalFunction(function_name) => {
                    if *function_name == other.function_name && self.function_filename == other.function_filename {
                        return Some(Ordering::Less);
                    }
                }
                Requirement::GlobalFunctionReq(greq) => {
                    if *greq.function_name == other.function_name && greq.filename == other.function_filename {
                        return Some(Ordering::Less);
                    }
                }
                _ => {}
            }
        }
        for req in &other.requirements {
            match req {
                Requirement::LocalFunction(function_name) => {
                    if *function_name == self.function_name && other.function_filename == self.function_filename {
                        return Some(Ordering::Greater);
                    }
                }
                Requirement::GlobalFunctionReq(greq) => {
                    if *greq.function_name == self.function_name && greq.filename == self.function_filename {
                        return Some(Ordering::Greater);
                    }
                }
                _ => {}
            }
        }

        return None;
    }
}

// Function to parse the file and extract the requirements
pub fn parse_requirements(file_path: &str) -> io::Result<Vec<FunctionImportReq>> {
    let path = Path::new(file_path);
    let file = File::open(&path)?;
    let reader = io::BufReader::new(file);

    let mut function_import_reqs = Vec::new();
    let mut current_function_import_req: Option<FunctionImportReq> = None;
    let mut current_shader_req = None;
    let mut current_global_function_name: Option<String> = None;
    let mut global_function_filename: Option<String> = None;

    for line in reader.lines() {
        let line = line?;

        

        if line.starts_with("//FUNCTION IMPORT REQ ") && line.ends_with(" START") {
            let function_name = line[21..line.len()-6].trim().to_string();
            current_function_import_req = Some(FunctionImportReq {
                function_name,
                function_filename: file_path.split('/').last().unwrap().to_string().replace("template.", ""),
                requirements: Vec::new(),
            });
        } else if line.starts_with("//FUNCTION IMPORT REQ END") {
            if let Some(req) = current_function_import_req.take() {
                function_import_reqs.push(req);
            }
        } else if let Some(req) = current_function_import_req.as_mut() {
            if let Some(ref function_name) = current_global_function_name {
                if line.starts_with("//FILENAME ") {
                    global_function_filename = Some(line[11..].trim().to_string());
                } else if line.starts_with("//GLOBAL FUNCTION END") {
                    req.requirements.push(Requirement::GlobalFunctionReq(GlobalFunctionReq {
                        function_name: function_name.to_string(),
                        filename: global_function_filename.clone().unwrap(),
                    }));
                    current_global_function_name = None;
                    global_function_filename = None;
                }
            }
            if line.starts_with("//LOCAL_FUNCTION ") {
                let function_name = line[17..].trim().to_string();
                req.requirements.push(Requirement::LocalFunction(function_name));
            } else if line.starts_with("//LOCAL_CONST ") {
                let const_name = line[14..].trim().to_string();
                req.requirements.push(Requirement::LocalConst(const_name));
            } else if line.starts_with("//CAMERA REQ") {
                req.requirements.push(Requirement::CameraReq);
            } else if line.starts_with("//FILE REQ ") {
                let filename = line[11..].trim().to_string();
                req.requirements.push(Requirement::FileReq(FileReq { filename }));
            } else if line.starts_with("//GLOBAL FUNCTION ") && line.ends_with(" START") {
                current_global_function_name = Some(line[18..line.len()-6].trim().to_string());
            } else if line.starts_with("//SHADER REQ BEGIN") {
                current_shader_req = Some(ShaderReq {
                    name: String::new(),
                    buffer_type: String::new(),
                    shader_type: String::new(),
                });
            } else if let Some(shader) = current_shader_req.as_mut() {
                if line.starts_with("//NAME ") {
                    shader.name = line[7..].trim().to_string();
                } else if line.starts_with("//BUFFER_TYPE ") {
                    shader.buffer_type = line[14..].trim().to_string();
                } else if line.starts_with("//TYPE ") {
                    shader.shader_type = line[7..].trim().to_string();
                } else if line.starts_with("//SHADER REQ END") {
                    req.requirements.push(Requirement::ShaderReq(shader.clone()));
                    current_shader_req = None;
                }
            }
        }
    }

    Ok(function_import_reqs)
}

pub fn remove_annotations(wgsl_content: &str) -> String {
    let mut result = Vec::new();
    let mut skip_lines = false;

    for line in wgsl_content.lines() {
        if line.starts_with("//FUNCTION IMPORT REQ ") && line.ends_with(" START") {
            skip_lines = true;
        } else if line.starts_with("//FUNCTION IMPORT REQ END") {
            skip_lines = false;
        } else if line.starts_with("//GLOBAL FUNCTION ") && line.ends_with(" START") {
            skip_lines = true;
        } else if line.starts_with("//GLOBAL FUNCTION END") {
            skip_lines = false;
        } else if skip_lines {
            continue;
        } else if line.starts_with("//LOCAL_FUNCTION ") || 
                  line.starts_with("//CAMERA REQ") || 
                  line.starts_with("//FILE REQ ") || 
                  line.starts_with("//NAME ") || 
                  line.starts_with("//BUFFER_TYPE ") || 
                  line.starts_with("//TYPE ") || 
                  line.starts_with("//SHADER REQ BEGIN") || 
                  line.starts_with("//SHADER REQ END") || 
                  line.starts_with("//FILENAME ") {
            continue;
        } else {
            result.push(line);
        }
    }

    result.join("\n")
}

pub fn get_artifact_filepath(filepath: &str) -> Result<String, &str> {
    // Ensure the filename has the correct prefix and suffix
    if filepath.starts_with("build/shaders/subvoxels/") && filepath.ends_with(".template.wgsl") {
        // Extract the {A} part
        let start_index = "build/shaders/subvoxels/".len();
        let end_index = filepath.len() - ".template.wgsl".len();
        let a_part = &filepath[start_index..end_index];
        
        // Construct the new filename
        let new_filename = format!("build/shaders/artifacts/{}.wgsl", a_part);
        Ok(new_filename)
    } else {
        Err("Filename format is incorrect")
    }
}

pub fn extract_functions(wgsl_code: &str) -> HashMap<String, String> {
    let mut functions = HashMap::new();
    let lines: Vec<&str> = wgsl_code.lines().collect();
    let mut inside_function = false;
    let mut function_name = String::new();
    let mut function_body = String::new();
    let mut brace_count = 0;

    for line in lines {
        if !inside_function {
            // Try to find the start of a function
            if let Some(start) = line.find("fn ") {
                if let Some(open_paren) = line[start..].find('(') {
                    if let Some(name) = line[start + 3..start + open_paren].trim().split_whitespace().next() {
                        inside_function = true;
                        function_name = name.to_string();
                        function_body = line.to_string();
                        brace_count = line.chars().filter(|&c| c == '{').count() - line.chars().filter(|&c| c == '}').count();
                        continue;
                    }
                }
            }
        } else {
            // We are inside a function body
            function_body.push('\n');
            function_body.push_str(line);
            brace_count += line.chars().filter(|&c| c == '{').count();
            brace_count -= line.chars().filter(|&c| c == '}').count();

            // Check if we have closed all braces
            if brace_count == 0 {
                functions.insert(function_name.clone(), function_body.clone());
                inside_function = false;
            }
        }
    }

    functions
}

pub fn extract_global_declarations(wgsl_code: &str) -> HashMap<String, String> {
    let re = Regex::new(r"(const|let|var)\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*:\s*[a-zA-Z_][a-zA-Z0-9_<>,\s]*\s*=\s*[^;]+;").unwrap();
    let mut declarations = HashMap::new();
    let mut brace_count = 0;
    let mut inside_function = false;

    for line in wgsl_code.lines() {
        let trimmed_line = line.trim();

        // Update brace count
        brace_count += trimmed_line.matches('{').count();
        brace_count -= trimmed_line.matches('}').count();

        // Detect the start of a function
        if trimmed_line.starts_with("fn ") {
            inside_function = true;
        }

        // Detect the end of a function
        if inside_function && brace_count == 0 {
            inside_function = false;
        }

        // Process line only if it's not inside a function
        if !inside_function && brace_count == 0 {
            if let Some(cap) = re.captures(trimmed_line) {
                let name = cap[2].to_string(); // variable name
                let full_statement = cap[0].to_string(); // entire statement
                
                declarations.insert(name, full_statement);
            }
        }
    }

    declarations
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub function_name: String,
    pub filename: String,
}

pub fn extract_inline_functions(input: &str) -> Vec<FunctionInfo> {
    let mut functions = Vec::new();

    // Define the regular expressions
    let inline_function_re = Regex::new(r"//INLINE FUNCTION ([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    let filename_re = Regex::new(r"//FILENAME ([^\s]+)").unwrap();
    let end_inline_function_re = Regex::new(r"//END INLINE FUNCTION").unwrap();

    let mut current_function_name = String::new();
    let mut current_filename = String::new();
    let mut inside_function = false;

    for line in input.lines() {
        if let Some(caps) = inline_function_re.captures(line) {
            current_function_name = caps[1].to_string();
            inside_function = true;
        } else if let Some(caps) = filename_re.captures(line) {
            if inside_function {
                current_filename = caps[1].to_string();
            }
        } else if end_inline_function_re.is_match(line) {
            if inside_function {
                functions.push(FunctionInfo {
                    function_name: current_function_name.clone(),
                    filename: current_filename.clone(),
                });
                inside_function = false;
            }
        }
    }

    functions
}

pub fn replace_inline_function(
    file_string: &str,
    function_name: &str,
    replacements: &HashMap<String, String>,
) -> String {
    let start_marker = format!("//INLINE FUNCTION {}", function_name);
    let end_marker = "//END INLINE FUNCTION";

    if let Some(start_index) = file_string.find(&start_marker) {
        if let Some(end_index) = file_string.find(&end_marker) {
            if end_index > start_index {
                // Extract the part before the inline function
                let before = &file_string[..start_index];
                // Extract the part after the inline function
                let after = &file_string[end_index + end_marker.len()..];
                // Join the replacement values with newlines
                let replacement = replacements.values().map(|s| s.as_str()).collect::<Vec<&str>>().join("\n");
                // Concatenate the parts with the replacement in between
                return format!("{}{}\n{}", before, replacement, after);
            }
        }
    }

    // If no replacement was made, return the original string
    file_string.to_string()
}