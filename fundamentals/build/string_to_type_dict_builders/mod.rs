use std::path::Path;
use std::fs::File;
use std::io::{BufWriter, Write};
use ::formats::formats::block_format::BlockFormat;

mod string_to_block_type;
mod string_to_texture_coords;

pub fn build_string_to_type_dictionaries(vec_block_format: &Vec<BlockFormat>, block_string_to_texture_indices: &Vec<(String, [usize;6])>) {
    let string_to_block_type_path = Path::new("../string_to_type_dictionaries/build.rs");
    let mut string_to_dict_build_file = BufWriter::new(File::create(string_to_block_type_path).unwrap());
    writeln!(
        &mut string_to_dict_build_file,
         "{}\nfn main() {{\n{}\n}}",
         build_string_to_dict_imports(),
         [
            build_file_creates(),
            build_lib_file(),
            string_to_block_type::build_string_to_block_type_dictionary_writeln(&vec_block_format),
            string_to_texture_coords::build_string_to_texture_coords_dictionary_writeln(&block_string_to_texture_indices)
         ].join("\n")
    ).unwrap();
}

fn build_file_creates() -> String {
    String::from([
        "let lib_path = Path::new(\"src/lib.rs\");",
        "let string_to_block_types_path = Path::new(\"src/string_to_block_type.rs\");",
        "let string_to_texture_indices_path = Path::new(\"src/string_to_texture_indices.rs\");",
        "let mut lib_file = BufWriter::new(File::create(&lib_path).unwrap());",
        "let mut string_to_block_types_file = BufWriter::new(File::create(&string_to_block_types_path).unwrap());",
        "let mut string_to_texture_indices_file = BufWriter::new(File::create(&string_to_texture_indices_path).unwrap());",
    ].join("\n"))
}

fn build_string_to_dict_imports() -> String {
    String::from([
        "use std::fs::File;",
        "use std::io::{BufWriter, Write};",
        "use std::path::Path;",
    ].join("\n"))
}

fn build_lib_file() -> String {
    String::from(
        format!("writeln!(\n\t&mut {},\n{}\n\t).unwrap();",
        "lib_file",
        [
        "\"{}{}\",",
        "\"pub mod string_to_block_type;\n\",",
        "\"pub mod string_to_texture_indices;\"",
        ].join("\n")
    ))
}