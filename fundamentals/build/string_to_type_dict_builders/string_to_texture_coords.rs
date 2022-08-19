pub fn build_string_to_texture_coords_dictionary_writeln(block_string_to_texture_coords: &Vec<(String, [(u32, u32);6])>) -> String {
    String::from(
        format!("writeln!(\n\t&mut string_to_texture_coords_file,\n{},\n{}\n\t).unwrap();",
        "\t\"use fundamentals::texture_coords::TextureCoordinates;\\npub static STRING_TO_TEXTURE_COORDINATES: phf::Map<&str, [TextureCoordinates; 6]> = \\n{};\\n\"",
        generate_string_to_texture_coords_map(block_string_to_texture_coords)),
        )
}

fn generate_string_to_texture_coords_map(block_string_to_texture_coords: &Vec<(String, [(u32, u32);6])>) -> String {
    let mut lines = Vec::new();
    lines.push("\tphf_codegen::Map::new()".to_string());
    for (string_name, texture_coords) in block_string_to_texture_coords {
        let mut texture_coord_strings = [(String::new(), String::new()), (String::new(), String::new()),(String::new(), String::new()),(String::new(), String::new()),(String::new(), String::new()),(String::new(), String::new())];
        for i in 0..6 {
            let tx = texture_coords[i].0;
            let ty = texture_coords[i].1;
            texture_coord_strings[i] = (tx.to_string().clone(), ty.to_string().clone());
        }
        let new_line = format!(".entry(\"{}\", \"[{}]\")", string_name, 
            [
                format!("TextureCoordinates {{tx: {}, ty: {}}}", texture_coord_strings[0].0, texture_coord_strings[0].1),
                format!("TextureCoordinates {{tx: {}, ty: {}}}", texture_coord_strings[1].0, texture_coord_strings[1].1),
                format!("TextureCoordinates {{tx: {}, ty: {}}}", texture_coord_strings[2].0, texture_coord_strings[2].1),
                format!("TextureCoordinates {{tx: {}, ty: {}}}", texture_coord_strings[3].0, texture_coord_strings[3].1),
                format!("TextureCoordinates {{tx: {}, ty: {}}}", texture_coord_strings[4].0, texture_coord_strings[4].1),
                format!("TextureCoordinates {{tx: {}, ty: {}}}", texture_coord_strings[5].0, texture_coord_strings[5].1)
            ].join(",")
            );
        lines.push(new_line);
    }
    lines.push(".build()".to_string());
    lines.join("\n\t\t")
}