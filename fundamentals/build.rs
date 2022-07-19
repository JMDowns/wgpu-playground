use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use formats::formats;
use serde_json;

fn main() {
    let blocks_json = std::fs::read_to_string("../data/blocks.json").unwrap();
    let vec_block_format: Vec<formats::block_format::BlockFormat> = serde_json::from_str(&blocks_json).unwrap();
    let block_type_reference_strings = vec_block_format.iter().map(|ef| format!("BlockType::{}", ef.block_type.to_uppercase())).collect::<Vec<String>>();
    let block_type_enum_values = vec_block_format.iter().map(|ef| format!("\t{},", ef.block_type.to_uppercase())).collect::<Vec<String>>();
    let num_block_types = block_type_enum_values.len()+1;
    let path = Path::new("src/enums/block_type.rs");
    let mut block_types_file = BufWriter::new(File::create(&path).unwrap());

    let consts_path = Path::new("src/consts.rs");
    let mut consts_file = BufWriter::new(File::create(&consts_path).unwrap());

    let string_to_block_type_path = Path::new("../string_to_type_dictionaries/build.rs");
    let mut string_to_block_type_file = BufWriter::new(File::create(&string_to_block_type_path).unwrap());

    writeln!(
        &mut consts_file,
        "pub const NUM_BLOCK_TYPES: u16 = {};",
        num_block_types
    ).unwrap();

    writeln!(
        &mut block_types_file,
         "{}\n{}\n{}",
         build_block_type_imports(),
         build_enum(&block_type_enum_values),
         build_traits()
    ).unwrap();

    let string_to_block_types_file_name = String::from("string_to_block_types_file");
    let lib_file_name = String::from("lib_file");

    writeln!(
        &mut string_to_block_type_file,
         "{}\nfn main() {{\n{}\n{}\n{}\n}}",
         build_string_to_block_type_imports(),
         build_file_creates(&string_to_block_types_file_name),
         build_lib_file(&lib_file_name),
         build_string_to_block_type_dictionary_builder(&string_to_block_types_file_name, vec_block_format, &block_type_reference_strings)
    ).unwrap();
}

fn build_block_type_imports() -> String {
    String::from([
        "use num_derive::FromPrimitive;",
        "use phf_shared;",
        "use std::fmt;",
        "use core::hash::{Hash, Hasher};"
    ].join("\n"))
}

fn build_file_creates(string_to_block_types_file_name: &String) -> String {
    String::from([
        "let lib_path = Path::new(\"src/lib.rs\");",
        "let string_to_block_types_path = Path::new(\"src/string_to_block_type.rs\");",
        "let mut lib_file = BufWriter::new(File::create(&lib_path).unwrap());",
        &format!("let mut {} = BufWriter::new(File::create(&string_to_block_types_path).unwrap());", string_to_block_types_file_name),
    ].join("\n"))
}

fn build_string_to_block_type_imports() -> String {
    String::from([
        "use std::fs::File;",
        "use std::io::{BufWriter, Write};",
        "use std::path::Path;",
    ].join("\n"))
}

fn build_string_to_block_type_dictionary_builder(string_to_block_type_dictionary_file_name: &String, block_values: Vec<formats::block_format::BlockFormat>, block_type_reference_strings: &Vec<String>) -> String {
    let block_names = block_values.iter().map(|b| &b.block_type);
    let map_entries = block_names.zip(block_type_reference_strings).collect::<Vec<(&String, &String)>>();
    String::from(
        format!("writeln!(\n\t&mut {},\n{},\n{}\n\t).unwrap();", 
        string_to_block_type_dictionary_file_name,
        "\t\"use fundamentals::enums::block_type::BlockType;\\npub static STRING_TO_BLOCK_TYPE: phf::Map<&str, BlockType> = \\n{};\\n\"",
        generate_string_to_block_type_map(map_entries)),
        )
}

fn build_lib_file(lib_file_name: &String) -> String {
    String::from(
        format!("writeln!(\n\t&mut {},\n{}\n\t).unwrap();",
        lib_file_name,
        "\"pub mod string_to_block_type;\""
        ))
}

// writeln!(
//     &mut file,
//      "{}\npub static BLOCK_TYPE_TO_COLOR: phf::Map<BlockType, wgpu::Color> = \n{};\n",
//      get_imports(),
//      phf_codegen::Map::new()
//      .entry(BlockType::STONE, "wgpu::Color { r: 0.411, g: 0.411, b: 0.411, a: 0.0}" )
//     .build()
// ).unwrap();

fn generate_string_to_block_type_map(entries: Vec<(&String, &String)>) -> String {
    let mut lines = Vec::new();
    lines.push("\tphf_codegen::Map::new()".to_string());
    for (string_name, block_type_name) in entries {
        let new_line = format!(".entry(\"{}\", \"{}\")", string_name, block_type_name);
        lines.push(new_line);
    }
    lines.push(".build()".to_string());
    lines.join("\n\t\t")
}

fn build_enum(block_type_enum_values: &Vec<String>) -> String {
    format!("#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, FromPrimitive)]\npub enum BlockType\n{{\n\tAIR = 0,\n{}\n}}\n",
        block_type_enum_values.join("\n"))
}

fn build_traits() -> String {
    String::from(
["impl phf_shared::FmtConst for BlockType {
    fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, \"BlockType::{:?}\", self)
    }
}",
"impl phf_shared::PhfHash for BlockType {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        (*self as u8).hash(state);
    }
}",
"impl phf_shared::PhfBorrow<BlockType> for BlockType {
    fn borrow(&self) -> &BlockType {
        self
    }
}"].join("\n")
    )
}