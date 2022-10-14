pub fn build_string_to_texture_coords_dictionary_writeln(block_string_to_texture_indices: &Vec<(String, [usize;6])>) -> String {
    String::from(
        format!("writeln!(\n\t&mut string_to_texture_indices_file,\n{},\n{}\n\t).unwrap();",
        "\t\"pub static STRING_TO_TEXTURE_INDICES: phf::Map<&str, [usize; 6]> = \\n{};\\n\"",
        generate_string_to_texture_coords_map(block_string_to_texture_indices)),
        )
}

fn generate_string_to_texture_coords_map(block_string_to_texture_indices: &Vec<(String, [usize;6])>) -> String {
    let mut lines = Vec::new();
    lines.push("\tphf_codegen::Map::new()".to_string());
    for (string_name, texture_index) in block_string_to_texture_indices {
        let new_line = format!(".entry(\"{}\", \"[{}]\")", string_name, 
            [
                format!("{}", texture_index[0]),
                format!(" {}", texture_index[1]),
                format!(" {}", texture_index[2]),
                format!(" {}", texture_index[3]),
                format!(" {}", texture_index[4]),
                format!(" {}", texture_index[5]),
            ].join(",")
            );
        lines.push(new_line);
    }
    lines.push(".build()".to_string());
    lines.join("\n\t\t")
}