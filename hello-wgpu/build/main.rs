pub mod shaders;

use core::num;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::f32::consts::E;
use std::fmt::Display;
use std::io::{self, Read, Write};
use std::env;
use std::path::PathBuf;
use std::fs::{self, File};
use std::path::Path;

use derivables::grid_aligned_subvoxel_vertex;
use indexmap::IndexSet;
use new_parser::{parse_wgsl_file, DataType};
//use shaders::*;

use fundamentals::consts;
use new_parser::*;
use shaders::extract_functions::{self, extract_file, extract_functions, ConstDetails};
use extract_functions::extract_declarations;
use build_model::*;
use shaders::build_model::{self, Model};
use shaders::new_parser::{self, Command, FunctionImportRequirements, InlineModelExtractCommand, Requirement};
use shaders::translate_artifact;
use translate_artifact::translate_template_filepath_to_artifact_filepath;

fn read_file_to_string(file_path: &str) -> io::Result<String> {
    // Open the file
    let mut file = File::open(file_path)?;

    // Create a String buffer to hold the file contents
    let mut contents = String::new();

    // Read the file contents into the String buffer
    file.read_to_string(&mut contents)?;

    // Return the contents
    Ok(contents)
}

fn write_file_with_content(file_name: &str, content: &str) -> io::Result<()> {
    // Check if the file exists
    let path = Path::new(file_name);
    if path.exists() {
        // Delete the file if it exists
        fs::remove_file(path)?;
    }

    // Create and write to the file
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

#[derive(Debug)]
enum DataValue {
    u32(u32),
    i32(i32),
    f32(f32),
    bool(bool),
    String(String),
}

impl DataValue {
    fn get_display_string(&self) -> String {
        match self {
            DataValue::u32(value) => value.to_string(),
            DataValue::i32(value) => value.to_string(),
            DataValue::f32(value) => value.to_string(),
            DataValue::bool(value) => value.to_string(),
            DataValue::String(value) => value.to_string(),
        }
    }
}

fn main() {
    let model_text_filepaths = vec!["../derivables/output.txt"];

    let mut models = Vec::new();

    for filepath in model_text_filepaths {
        let file_models = build_model::parse_file(filepath).unwrap();
        models.extend(file_models);
    }

    let mut filename_to_requirements = HashMap::new();
    let mut filename_to_commands = HashMap::new();
    let mut filename_to_functions = HashMap::new();
    let mut filename_to_consts = HashMap::new();

    let mut value_provider: HashMap<String, DataValue> = HashMap::new();
    value_provider.insert("BITS_PER_SUBVOXEL_PALETTE".to_string(), DataValue::i32(consts::BITS_PER_SUBVOXEL_PALETTE as i32));
    value_provider.insert("MAX_SUBVOXEL_U32S".to_string(), DataValue::i32(consts::NUM_SUBVOXEL_U32s as i32));
    value_provider.insert("MAX_AMBIENT_OCCLUSION_U32S".to_string(), DataValue::i32(consts::MAX_AMBIENT_OCCLUSION_U32S as i32));
    value_provider.insert("NUM_GRID_ALIGNED_SUBVOXEL_OBJECTS".to_string(), DataValue::i32(consts::MAX_GRID_ALIGNED_SUBVOXEL_OBJECTS as i32));
    value_provider.insert("MAX_COLORS".to_string(), DataValue::i32(consts::MAX_SUBVOXEL_COLORS as i32));
    value_provider.insert("NUM_CHUNK_I32S".to_string(), DataValue::i32(consts::NUMBER_OF_CHUNKS_AROUND_PLAYER as i32 * 3));
    value_provider.insert("CHUNK_DIMENSION".to_string(), DataValue::u32(consts::CHUNK_DIMENSION as u32));
    value_provider.insert("SUBVOXEL_DIMENSION".to_string(), DataValue::u32(consts::GRID_ALIGNED_SUBVOXEL_PLACEMENT_DIMENSION as u32));


    let template_filepaths = vec![
        "build/shaders/subvoxels/raycast.template.wgsl",
        "build/shaders/subvoxels/ao_calc.template.wgsl",
        "build/shaders/subvoxels/sides.template.wgsl",
        "build/shaders/subvoxels/grid_aligned_subvoxel_shader.template.wgsl"
        ];

    for filepath in template_filepaths.iter() {
        let artifact_filepath = translate_template_filepath_to_artifact_filepath(filepath);

        let (requirements, commands) = parse_wgsl_file(&filepath).unwrap();
        let functions = extract_functions(&filepath).unwrap();
        let consts = extract_declarations(&filepath).unwrap();
    
        filename_to_requirements.insert(artifact_filepath.clone(), requirements);
        filename_to_commands.insert(artifact_filepath.clone(), commands);
        filename_to_functions.insert(artifact_filepath.clone(), functions);
        filename_to_consts.insert(artifact_filepath, consts);
    }

    let saturated_filename_to_requirements = saturate_requirements(filename_to_requirements);

    for filepath in template_filepaths.iter() {
        let artifact_filepath = translate_template_filepath_to_artifact_filepath(filepath);

        let mut write_commands = Vec::new();
        let mut write_consts = Vec::new();
        let mut write_structs = Vec::new();
        let mut write_buffers = Vec::new();
        let mut write_functions = Vec::new();
        let mut import_commands_set = IndexSet::new();

        let current_filename = artifact_filepath.clone();

        let commands = filename_to_commands.get(&current_filename).unwrap();

        for command in commands {
            match command {
                Command::WriteString(s) => write_commands.push(Command::WriteString(s.clone())),
                Command::WriteConst(c) => write_consts.push(Command::WriteString(c.declaration.clone())),
                Command::WriteBuffer(b) => write_buffers.push(Command::WriteString(b.declaration.clone())),
                Command::WriteStruct(s) => write_structs.push(Command::WriteString(s.clone())),
                Command::InlineConst(data_type, const_name) => {
                    let write_command = process_inline_const(data_type, const_name, &value_provider);
                    match &write_command {
                        Command::WriteString(s) => {
                            let const_name = s.clone().split(" ").collect::<Vec<&str>>()[1].to_string();
                            filename_to_consts.get_mut(&current_filename).unwrap().push(ConstDetails {
                                name: const_name,
                                body: s.clone(),
                            });
                        }
                        _ => {}
                    };
                    write_consts.push(write_command);
                }
                Command::InlineVertexStruct(struct_name, import_name) => {
                    let write_command = process_inline_vertex_struct(struct_name, import_name, &models);
                    write_structs.push(write_command);
                }
                Command::InlineModelExtract(command) => {
                    let wgsl_code = generate_wgsl_code(&models, &command);
                    write_commands.push(Command::WriteString(wgsl_code));
                }
                Command::InlineFunction(function_name, file_name) => {
                    let artifact_path = translate_template_filepath_to_artifact_filepath(&file_name);
                    println!("{:?} {:?}", file_name, artifact_path);
                    let requirements = saturated_filename_to_requirements.get(&artifact_path).unwrap();
                    let import_requirements = &requirements.into_iter().find(|&req| &req.function_name == function_name).unwrap().import_requirements;
                    for req in import_requirements.into_iter() {
                        import_commands_set.insert(req);
                    }
                }
                _ => {}
            }
        }

        for requirement in import_commands_set.into_iter().rev() {
            match requirement {
                Requirement::Function(function_name, file_name) => {
                    let functions = filename_to_functions.get(file_name).unwrap();
                    let function_details = functions.iter().find(|&func| &func.name == function_name).unwrap();
                    write_commands.insert(0, Command::WriteString(function_details.body.clone()));
                }
                Requirement::Const(const_name, file_name) => {
                    let const_details = filename_to_consts.get(file_name).unwrap().iter().find(|&c| &c.name == const_name).unwrap();
                    write_commands.insert(0, Command::WriteString(const_details.body.clone()));
                }
                Requirement::File(file_name) => {
                    let string  = format!("build/shaders/{}", file_name);
                    println!("{}", string);
                    let file_contents = extract_file(&string);
                    write_commands.insert(0, Command::WriteString(file_contents));
                }
                _ => {}
            }
        }

        println!("{:?}", artifact_filepath);
        let mut file = File::create(artifact_filepath).unwrap();
    
        for command in write_consts {
            if let Command::WriteString(ref value) = command {
                writeln!(file, "{}", value).unwrap();
            }
        }

        for command in write_structs {
            if let Command::WriteString(ref value) = command {
                writeln!(file, "{}", value).unwrap();
            }
        }

        for command in write_buffers {
            if let Command::WriteString(ref value) = command {
                writeln!(file, "{}", value).unwrap();
            }
        }

        for command in write_commands {
            if let Command::WriteString(ref value) = command {
                writeln!(file, "{}", value).unwrap();
            }
        }

        for command in write_functions {
            if let Command::WriteString(ref value) = command {
                writeln!(file, "{}", value).unwrap();
            }
        }
    }

    let publish_dir = "src/gpu_manager/shaders";
    let files_to_publish = vec![
        "subvoxels/grid_aligned_subvoxel_shader.wgsl"
    ];

    for file in files_to_publish {
        let artifact_filepath = &format!("build/shaders/artifacts/{}", file);
        let publish_filepath = format!("{}/{}", publish_dir, file);
        fs::copy(artifact_filepath, publish_filepath).unwrap();
    }
}

fn saturate_requirements(
    filename_to_requirements: HashMap<String, Vec<FunctionImportRequirements>>,
) -> HashMap<String, Vec<FunctionImportRequirements>> {
    let mut saturated_requirements: HashMap<String, Vec<FunctionImportRequirements>> = HashMap::new();
    let mut cache: HashMap<(String, String), Vec<Requirement>> = HashMap::new();

    for (file_name, functions) in filename_to_requirements.iter() {
        let mut saturated_function_imports: Vec<FunctionImportRequirements> = Vec::new();

        for function in functions {
            if let Some(cached_requirements) = cache.get(&(function.function_name.clone(), file_name.clone())) {
                // Use cached results if available
                saturated_function_imports.push(FunctionImportRequirements {
                    function_name: function.function_name.clone(),
                    import_requirements: cached_requirements.clone(),
                });
                continue;
            }

            let mut visited = HashSet::new();
            let mut all_requirements = vec![Requirement::Function(function.function_name.clone(), file_name.clone())];
            let mut stack = VecDeque::new();

            // Start processing with the initial function
            stack.push_back((function.function_name.clone(), file_name.clone()));

            while let Some((current_function, current_file)) = stack.pop_back() {
                // Skip if we've already visited this function
                if visited.contains(&(current_function.clone(), current_file.clone())) {
                    continue;
                }
                visited.insert((current_function.clone(), current_file.clone()));

                if let Some(functions) = filename_to_requirements.get(&current_file) {
                    for func in functions {
                        if &func.function_name == &current_function {
                            for requirement in &func.import_requirements {
                                // If the requirement is a function, push it onto the stack
                                if let Requirement::Function(ref dep_function_name, ref dep_file_name) = requirement {
                                    stack.push_back((dep_function_name.clone(), dep_file_name.clone()));
                                }
                                 // Add the requirement to the front of the list
                                 if let Some(pos) = all_requirements.iter().position(|r| r == requirement) {
                                    // Move existing requirement to the front
                                    let req = all_requirements.remove(pos);
                                    println!("Moving requirement to the front: {:?}", req);
                                    all_requirements.insert(0, req);
                                } else {
                                    // Insert new requirement at the front
                                    all_requirements.insert(0, requirement.clone());
                                }
                            }
                        }
                    }
                }
            }

            // Cache the results
            cache.insert((function.function_name.clone(), file_name.clone()), all_requirements.clone());

            // Add the saturated requirements for this function
            saturated_function_imports.push(FunctionImportRequirements {
                function_name: function.function_name.clone(),
                import_requirements: all_requirements,
            });
        }

        saturated_requirements.insert(file_name.clone(), saturated_function_imports);
    }

    saturated_requirements
}

fn process_inline_const(data_type: &DataType, const_name: &String, value_provider: &HashMap<String, DataValue>) -> Command {
    match value_provider.get(const_name) {
        Some(value) => {
            return match data_type {
                DataType::U32 => Command::WriteString(format!("let {} : u32 = {}u;", const_name, value.get_display_string())),
                DataType::I32 => Command::WriteString(format!("let {} : i32 = {};", const_name, value.get_display_string())),
                DataType::F32 => Command::WriteString(format!("let {} : f32 = {};", const_name, value.get_display_string())),
                DataType::Bool => Command::WriteString(format!("let {} : bool = {};", const_name, value.get_display_string())),
                _ => Command::WriteString("".to_string())
            };
        },
        None => println!("Value not found for const_name: {}", const_name)
    }

    Command::WriteString("".to_string())
}

fn process_inline_vertex_struct(struct_name: &String, import_name: &String, models: &Vec<Model>) -> Command {
    let model = models.iter().find(|&m| &m.name == import_name).unwrap();
    let num_data_properties = model.get_num_u32s_to_fit_model();
    let data_properties = (0..num_data_properties).map(|n| format!("\t@location({}) data{}: u32,", n, n)).collect::<Vec<String>>().join("\n");
    Command::WriteString(format!(
        "struct {} {{
            {}
        }};", struct_name, data_properties
    ))
}

fn generate_wgsl_code(models: &Vec<Model>, extract_command: &InlineModelExtractCommand) -> String {
    // Find the model based on the name in the extract command
    let model = models.iter().find(|&m| m.name == extract_command.model_name).unwrap();
    
    let mut wgsl_code = String::new();

    for model_property in &extract_command.model_properties {
        if let Some(property) = model.properties.get(&model_property.import_name) {
            let mut first_bit_slice = true;

            for bit_slice in &property.bit_slices {
                let start_index = bit_slice.bit_start / 32;
                let start_bit = bit_slice.bit_start % 32;
                let mask = (1u32 << bit_slice.bit_length) - 1;

                if first_bit_slice {
                    wgsl_code.push_str(&format!(
                        "var {}: u32 = ((model.data{} >> {}u) & 0x{:X}u);\n",
                        model_property.property_name, start_index, start_bit, mask
                    ));
                    first_bit_slice = false;
                } else {
                    match bit_slice.subbit_start {
                        Some(start) => {
                            wgsl_code.push_str(&format!(
                                "{} |= ((model.data{} >> {}u) & 0x{:X}u) << {}u;\n",
                                model_property.property_name, start_index, start_bit, mask, start
                            ));
                        },
                        None => panic!("Subbit start not found")
                    }
                }
            }
        }
    }

    wgsl_code
}