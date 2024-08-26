use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use regex::Regex;

use crate::translate_artifact::translate_template_filepath_to_artifact_filepath;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Requirement {
    Const(String, String),
    Function(String, String),
    Buffer(BufferReq),
    Camera,
    File(String),
}

#[derive(Debug)]
pub struct FunctionImportRequirements {
    pub function_name: String,
    pub import_requirements: Vec<Requirement>
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BufferReq {
    name: String,
    buffer_type: String,
    value_type: String,
}

#[derive(Debug)]
pub struct Buffer {
    pub name: String,
    pub declaration: String,
}

#[derive(Debug)]
pub struct Const {
    pub name: String,
    pub declaration: String,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub declaration: String,
}

#[derive(Debug)]
pub enum Command {
    WriteString(String),
    WriteBuffer(Buffer),
    WriteConst(Const),
    WriteStruct(String),
    WriteFunction(Function),
    InlineFunction(String, String),
    InlineConst(DataType, String),
    InlineVertexStruct(String, String),
    InlineModelExtract(InlineModelExtractCommand),
}

#[derive(Debug)]
pub struct InlineModelExtractCommand {
    pub model_name: String,
    pub model_properties: Vec<ModelProperty>,
}

#[derive(Debug)]
pub struct ModelProperty {
    pub property_name: String,
    pub import_name: String,
    pub data_type: DataType,
}

#[derive(Debug)]
pub enum DataType {
    I32,
    U32,
    F32,
    F16,
    Bool,
    Vec2I32,
    Vec3I32,
    Vec4I32,
    Vec2U32,
    Vec3U32,
    Vec4U32,
    Vec2F32,
    Vec3F32,
    Vec4F32,
    Vec2F16,
    Vec3F16,
    Vec4F16,
    Mat2x2F32,
    Mat2x3F32,
    Mat2x4F32,
    Mat3x2F32,
    Mat3x3F32,
    Mat3x4F32,
    Mat4x2F32,
    Mat4x3F32,
    Mat4x4F32,
    Mat2x2F16,
    Mat2x3F16,
    Mat2x4F16,
    Mat3x2F16,
    Mat3x3F16,
    Mat3x4F16,
    Mat4x2F16,
    Mat4x3F16,
    Mat4x4F16,
}

pub fn parse_wgsl_file(file_name: &str) -> Result<(Vec<FunctionImportRequirements>, Vec<Command>), std::io::Error> {
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::path::Path;

    // Read the file into file_content
    let path = Path::new(file_name);
    let file = File::open(&path)?;
    let file_content: String = io::BufReader::new(file)
        .lines()
        .filter_map(Result::ok)
        .collect::<Vec<String>>()
        .join("\n");

    let mut requirements = Vec::new();
    let mut commands = Vec::new();
    let mut lines = file_content.lines().peekable();
    let current_file_name = file_name.to_string();
    let mut write_string = String::new();

    let mut in_function = false;
    let mut in_function_curly_braces = 0;

    while let Some(line) = lines.next() {
        if (in_function && line.contains("{")) {
            in_function_curly_braces += 1;
        }
        if (in_function && line.contains("}")) {
            in_function_curly_braces -= 1;
            if (in_function_curly_braces == 0) {
                in_function = false;
            }
        }
        if line.trim().starts_with("//") {
            let comment = line.trim().trim_start_matches("//").trim();
            if comment.starts_with("INLINE CONST") {
                flush_write_string(&mut write_string, &mut commands);
                match parse_inline_const(line, &mut lines) {
                    Ok((data_type, name)) => {
                        commands.push(Command::InlineConst(data_type, name));
                    }
                    Err(err) => {
                        eprintln!("Error: {}", err);
                        break;
                    }
                }
            } else if comment.starts_with("INLINE VERTEX STRUCT") {
                flush_write_string(&mut write_string, &mut commands);
                match parse_inline_vertex_struct(line, &mut lines) {
                    Ok((struct_name, import_name)) => {
                        commands.push(Command::InlineVertexStruct(struct_name, import_name));
                    }
                    Err(err) => {
                        eprintln!("Error: {}", err);
                        break;
                    }
                }
            } else if comment.starts_with("INLINE MODEL EXTRACT") {
                flush_write_string(&mut write_string, &mut commands);
                match parse_inline_model_extract(line, &mut lines) {
                    Ok(model_command) => {
                        commands.push(Command::InlineModelExtract(model_command));
                    }
                    Err(err) => {
                        eprintln!("Error: {}", err);
                        break;
                    }
                }
            } else if comment.starts_with("INLINE FUNCTION") {
                flush_write_string(&mut write_string, &mut commands);
                match parse_inline_function(line, &mut lines) {
                    Ok((function_name, file_name)) => {
                        commands.push(Command::InlineFunction(function_name, format!("build/shaders/{}", file_name)));
                    }
                    Err(err) => {
                        eprintln!("Error: {}", err);
                        break;
                    }
                }
            } else if comment.starts_with("FUNCTION IMPORT REQ") {
                flush_write_string(&mut write_string, &mut commands);
                match parse_function_import_req(line, &mut lines, current_file_name.clone()) {
                    Ok((function_name, reqs)) => {
                        requirements.push(FunctionImportRequirements {
                            function_name,
                            import_requirements: reqs
                        });
                    }
                    Err(err) => {
                        eprintln!("Error: {}", err);
                        break;
                    }
                }
            } else {
                // Append other comments to write_string
                write_string.push_str(line);
                write_string.push('\n');
            }
        } else if line.trim().starts_with("const") && !in_function {
            flush_write_string(&mut write_string, &mut commands);
            match parse_write_const(line) {
                Some(write_const) => commands.push(Command::WriteConst(write_const)),
                None => {
                    eprintln!("Error: Invalid write const format");
                    break;
                }
            }
        } else if line.trim().starts_with("@group") {
            flush_write_string(&mut write_string, &mut commands);
            match parse_write_buffer(line, &mut lines) {
                Some(write_buffer) => commands.push(Command::WriteBuffer(write_buffer)),
                None => {
                    eprintln!("Error: Invalid write buffer format");
                    break;
                }
            }
        } else if line.trim().starts_with("struct") {
            flush_write_string(&mut write_string, &mut commands);
            match parse_write_struct(line, &mut lines) {
                Some(struct_declaration) => commands.push(Command::WriteStruct(struct_declaration)),
                None => {
                    eprintln!("Error: Invalid struct format");
                    break;
                }
            }
        } else if line.trim().starts_with("fn") {
            in_function = true;
            in_function_curly_braces = 1;
            write_string.push_str(line);
            write_string.push('\n');
        } else {
            // Append all non-comment lines to write_string
            write_string.push_str(line);
            write_string.push('\n');
        }
    }

    // Flush any remaining write_string content
    flush_write_string(&mut write_string, &mut commands);

    Ok((requirements, commands))
}


fn flush_write_string(write_string: &mut String, commands: &mut Vec<Command>) {
    if !write_string.is_empty() {
        commands.push(Command::WriteString(write_string.clone()));
        write_string.clear();
    }
}

fn parse_inline_const(line: &str, lines: &mut std::iter::Peekable<std::str::Lines>) -> Result<(DataType, String), &'static str> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    // Ensure the line contains the necessary parts
    if parts.len() != 4 {
        return Err("Expected INLINE CONST format to have three parts");
    }

    let data_type = parse_data_type(parts[2]);
    let name = parts[3].to_string();

    // Consume lines until the end line
    while let Some(next_line) = lines.next() {
        if next_line.trim() == "//END INLINE CONST" {
            return Ok((data_type, name));
        }
    }

    Err("Expected //END INLINE CONST, but found end of input")
}

fn parse_inline_vertex_struct(line: &str, lines: &mut std::iter::Peekable<std::str::Lines>) -> Result<(String, String), &'static str> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    // Ensure the line contains the necessary parts
    if parts.len() != 5 {
        return Err("Expected INLINE VERTEX_STRUCT format to have five parts");
    }

    let struct_name = parts[3].to_string();
    let import_name = parts[4].to_string();

    // Consume lines until the end line
    while let Some(next_line) = lines.next() {
        if next_line.trim() == "//END INLINE VERTEX STRUCT" {
            return Ok((struct_name, import_name));
        }
    }

    Err("Expected //END INLINE VERTEX_STRUCT, but found end of input")
}

fn parse_inline_model_extract(line: &str, lines: &mut std::iter::Peekable<std::str::Lines>) -> Result<InlineModelExtractCommand, &'static str> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    // Ensure the line contains the necessary parts
    if parts.len() < 4 {
        return Err("Expected INLINE MODEL EXTRACT format to have at least four parts");
    }

    let model_name = parts[4].to_string();
    let mut model_properties = Vec::new();

    // Consume lines until the end line
    while let Some(next_line) = lines.next() {
        let trimmed_line = next_line.trim();
        if trimmed_line == "// END INLINE MODEL EXTRACT" {
            return Ok(InlineModelExtractCommand {
                model_name,
                model_properties,
            });
        }
        let parts: Vec<&str> = trimmed_line.split_whitespace().collect();
        if parts.len() == 4 {
            model_properties.push(ModelProperty {
                property_name: parts[2].to_string(),
                import_name: parts[3].to_string(),
                data_type: parse_data_type(parts[1]),
            });
        } else {
            return Err("Invalid model property format");
        }
    }

    Err("Expected // END INLINE MODEL EXTRACT, but found end of input")
}

fn parse_inline_function(line: &str, lines: &mut std::iter::Peekable<std::str::Lines>) -> Result<(String, String), &'static str> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    // Ensure the line contains the necessary parts
    if parts.len() != 3 {
        return Err("Expected INLINE FUNCTION format to have two parts");
    }

    let function_name = parts[2].to_string();

    // Validate and read the filename line
    let file_name_line = lines.next().ok_or("Expected FILENAME line, but found end of input")?.trim();
    let file_name_parts: Vec<&str> = file_name_line.split_whitespace().collect();
    if file_name_parts.len() != 2 || file_name_parts[0] != "//FILENAME" {
        return Err("Expected //FILENAME followed by the file name");
    }

    let file_name = file_name_parts[1].to_string();

    // Consume lines until the end line
    while let Some(next_line) = lines.next() {
        if next_line.trim() == "//END INLINE FUNCTION" {
            return Ok((function_name, file_name));
        }
    }

    Err("Expected //END INLINE FUNCTION, but found end of input")
}

fn parse_function_import_req(line: &str, lines: &mut std::iter::Peekable<std::str::Lines>, current_file_name: String) -> Result<(String, Vec<Requirement>), &'static str> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    // Ensure the line contains the necessary parts
    if parts.len() != 5 {
        return Err("Expected FUNCTION IMPORT REQ format to have five parts");
    }

    let function_name = parts[3].to_string();
    let mut import_requirements = Vec::new();

    while let Some(next_line) = lines.next() {
        let trimmed_line = next_line.trim();
        if trimmed_line == "//FUNCTION IMPORT REQ END" {
            return Ok((function_name, import_requirements));
        }
        if trimmed_line.starts_with("//LOCAL_CONST") {
            let artifact_filename = translate_template_filepath_to_artifact_filepath(&current_file_name.clone());
            import_requirements.push(Requirement::Const(parse_local_const(trimmed_line)?, artifact_filename));
        } else if trimmed_line.starts_with("//BUFFER REQ BEGIN") {
            import_requirements.push(Requirement::Buffer(parse_buffer_req(lines)?));
        } else if trimmed_line.starts_with("//LOCAL_FUNCTION") {
            let artifact_filename = translate_template_filepath_to_artifact_filepath(&current_file_name.clone());
            import_requirements.push(Requirement::Function(parse_local_function(trimmed_line)?, artifact_filename));
        } else if trimmed_line.starts_with("//CAMERA") {
            import_requirements.push(Requirement::Camera);
        } else if trimmed_line.starts_with("//GLOBAL FUNCTION") {
            let (global_function_name, global_file_name) = parse_global_function(trimmed_line, lines)?;
            let global_artifact_filename = format!("build/shaders/artifacts/{}", global_file_name);
            import_requirements.push(Requirement::Function(global_function_name, global_artifact_filename));
        } else if trimmed_line.starts_with("//FILE REQ") {
            import_requirements.push(Requirement::File(parse_file_req(trimmed_line)?));
        } else if trimmed_line.is_empty() {
        } else {
            return Err("Unexpected requirement format in FUNCTION IMPORT REQ");
        }
    }

    Err("Expected //FUNCTION IMPORT REQ END, but found end of input")
}

fn parse_local_const(line: &str) -> Result<String, &'static str> {
    line.split_whitespace().nth(1).map(|s| s.to_string()).ok_or("Expected LOCAL_CONST format to have two parts")
}


fn parse_buffer_req(lines: &mut std::iter::Peekable<std::str::Lines>) -> Result<BufferReq, &'static str> {
    let mut name = String::new();
    let mut buffer_type = String::new();
    let mut value_type = String::new();
    while let Some(line) = lines.next() {
        let trimmed_line = line.trim();
        if trimmed_line == "//BUFFER REQ END" {
            return Ok(BufferReq {
                name,
                buffer_type,
                value_type,
            });
        }
        let parts: Vec<&str> = trimmed_line.split_whitespace().collect();
        match parts[0] {
            "//NAME" => name = parts[1].to_string(),
            "//BUFFER_TYPE" => buffer_type = parts[1].to_string(),
            "//TYPE" => value_type = parts[1].to_string(),
            _ => return Err("Unexpected part in BUFFER REQ"),
        }
    }

    Err("Expected //BUFFER REQ END, but found end of input")
}

fn parse_local_function(line: &str) -> Result<String, &'static str> {
    line.split_whitespace().nth(1).map(|s| s.to_string()).ok_or("Expected LOCAL_FUNCTION format to have two parts")
}

fn parse_global_function(line: &str, lines: &mut std::iter::Peekable<std::str::Lines>) -> Result<(String, String), &'static str> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    // Ensure the line contains the necessary parts
    if parts.len() != 4 {
        return Err("Expected GLOBAL FUNCTION format to have four parts");
    }

    let function_name = parts[2].to_string();

    // Validate and read the filename line
    let file_name_line = lines.next().ok_or("Expected FILENAME line, but found end of input")?.trim();
    let file_name_parts: Vec<&str> = file_name_line.split_whitespace().collect();
    if file_name_parts.len() != 2 || file_name_parts[0] != "//FILENAME" {
        return Err("Expected //FILENAME followed by the file name");
    }

    let file_name = file_name_parts[1].to_string();

    // Consume lines until the end line
    while let Some(next_line) = lines.next() {
        if next_line.trim() == "//GLOBAL FUNCTION END" {
            return Ok((function_name, file_name));
        }
    }

    Err("Expected //GLOBAL FUNCTION END, but found end of input")
}

fn parse_file_req(line: &str) -> Result<String, &'static str> {
    line.split_whitespace().nth(2).map(|s| s.to_string()).ok_or("Expected FILE REQ format to have three parts")
}

fn parse_data_type(type_str: &str) -> DataType {
    match type_str {
        "i32" => DataType::I32,
        "u32" => DataType::U32,
        "f32" => DataType::F32,
        "f16" => DataType::F16,
        "bool" => DataType::Bool,
        "vec2<i32>" => DataType::Vec2I32,
        "vec3<i32>" => DataType::Vec3I32,
        "vec4<i32>" => DataType::Vec4I32,
        "vec2<u32>" => DataType::Vec2U32,
        "vec3<u32>" => DataType::Vec3U32,
        "vec4<u32>" => DataType::Vec4U32,
        "vec2<f32>" => DataType::Vec2F32,
        "vec3<f32>" => DataType::Vec3F32,
        "vec4<f32>" => DataType::Vec4F32,
        "vec2<f16>" => DataType::Vec2F16,
        "vec3<f16>" => DataType::Vec3F16,
        "vec4<f16>" => DataType::Vec4F16,
        "mat2x2<f32>" => DataType::Mat2x2F32,
        "mat2x3<f32>" => DataType::Mat2x3F32,
        "mat2x4<f32>" => DataType::Mat2x4F32,
        "mat3x2<f32>" => DataType::Mat3x2F32,
        "mat3x3<f32>" => DataType::Mat3x3F32,
        "mat3x4<f32>" => DataType::Mat3x4F32,
        "mat4x2<f32>" => DataType::Mat4x2F32,
        "mat4x3<f32>" => DataType::Mat4x3F32,
        "mat4x4<f32>" => DataType::Mat4x4F32,
        "mat2x2<f16>" => DataType::Mat2x2F16,
        "mat2x3<f16>" => DataType::Mat2x3F16,
        "mat2x4<f16>" => DataType::Mat2x4F16,
        "mat3x2<f16>" => DataType::Mat3x2F16,
        "mat3x3<f16>" => DataType::Mat3x3F16,
        "mat3x4<f16>" => DataType::Mat3x4F16,
        "mat4x2<f16>" => DataType::Mat4x2F16,
        "mat4x3<f16>" => DataType::Mat4x3F16,
        "mat4x4<f16>" => DataType::Mat4x4F16,
        _ => panic!("Unknown data type: {}", type_str),
    }
}

fn parse_write_const(line: &str) -> Option<Const> {
    let pattern = regex::Regex::new(r"^const\s+(\w+)\s*=\s*.*;$").unwrap();
    if let Some(captures) = pattern.captures(line.trim()) {
        let name = captures[1].to_string();
        return Some(Const {
            name,
            declaration: line.to_string(),
        });
    }
    None
}

fn parse_write_struct(line: &str, lines: &mut std::iter::Peekable<std::str::Lines>) -> Option<String> {
    let mut declaration = String::new();
    declaration.push_str(line);
    declaration.push('\n');

    while let Some(line) = lines.next() {
        declaration.push_str(line);
        declaration.push('\n');
        if (line.trim().contains("}")) {
            return Some(declaration);
        }
    }

    panic!("Got to end of file before parsing struct!");
}

fn parse_write_buffer(current_line: &str, lines: &mut std::iter::Peekable<std::str::Lines>) -> Option<Buffer> {
    let mut declaration = String::new();
    declaration.push_str(current_line);
    declaration.push('\n');

    if let Some(line) = lines.next() {
        let trimmed_line = line.trim();
        let re = Regex::new(r"var<[^>]+>\s+(\w+):\s*.*").unwrap();
        if let Some(captures) = re.captures(trimmed_line) {
            declaration.push_str(line);
            return Some(Buffer {
                name: captures[1].to_string(),
                declaration,
            });
        } else {
            panic!("Expected buffer declaration to be in the correct format! {}", trimmed_line);
        };
    }
    else {
        panic!("Expected buffer declaration to be on the next line");
    }
}

fn parse_write_function(current_line: &str, lines: &mut std::iter::Peekable<std::str::Lines>) -> Option<Function> {
    let mut declaration = String::new();
    declaration.push_str(current_line);
    declaration.push('\n');

    let trimmed_line = current_line.trim();
    let name_end = trimmed_line.find('(').unwrap_or(trimmed_line.len());
    let function_name = trimmed_line[3..name_end].trim().to_string();

    let mut nested_braces_count = 1;

    while let Some(line) = lines.next() {
        let trimmed_line = line.trim();
        declaration.push_str(&line);
        declaration.push('\n');
        if trimmed_line.contains("{") {
            nested_braces_count += 1;
        }
        if trimmed_line.contains("}") {
            nested_braces_count -= 1;
            // Close the current function
            if nested_braces_count == 0 {
                return Some(Function {
                    name: function_name,
                    declaration,
                });
            }
            
        }
    }

    panic!("Got to end of file before parsing function!");
}