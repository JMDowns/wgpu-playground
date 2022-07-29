use ::formats::formats::block_format::BlockFormat;

pub fn build_string_to_block_type_dictionary_writeln(vec_block_format: &Vec<BlockFormat>) -> String {
    let block_type_reference_strings = vec_block_format.iter().map(|ef| format!("BlockType::{},", ef.block_type.replace(" ", "_").to_uppercase())).collect::<Vec<String>>();
    let block_names = vec_block_format.iter().map(|b| &b.block_type);
    let map_entries = block_names.zip(block_type_reference_strings).collect::<Vec<(&String, String)>>();
    String::from(
        format!("writeln!(\n\t&mut string_to_block_types_file,\n{},\n{}\n\t).unwrap();",
        "\t\"use fundamentals::enums::block_type::BlockType;\\npub static STRING_TO_BLOCK_TYPE: phf::Map<&str, BlockType> = \\n{};\\n\"",
        generate_string_to_block_type_map(map_entries)),
        )
}

fn generate_string_to_block_type_map(entries: Vec<(&String, String)>) -> String {
    let mut lines = Vec::new();
    lines.push("\tphf_codegen::Map::new()".to_string());
    for (string_name, block_type_name) in entries {
        let new_line = format!(".entry(\"{}\", \"{}\")", string_name, block_type_name);
        lines.push(new_line);
    }
    lines.push(".build()".to_string());
    lines.join("\n\t\t")
}