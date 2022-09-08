use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use string_to_type_dictionaries::string_to_block_type::STRING_TO_BLOCK_TYPE;
use string_to_type_dictionaries::string_to_texture_coords::STRING_TO_TEXTURE_COORDINATES;
use formats::formats;
use phf_codegen;
use serde_json;
mod vertex_builder;
mod shader_builder;
mod frustum_compute_builder;

fn main() {
    let block_type_to_texture_coordinates_path = Path::new("src/dictionaries/").join("block_type_to_texture_coordinates.rs");
    let mut block_type_to_texture_coordinates_file = BufWriter::new(File::create(&block_type_to_texture_coordinates_path).unwrap());

    let blocks_json = std::fs::read_to_string("../data/blocks.json").unwrap();
    let vec_block_format: Vec<formats::block_format::BlockFormat> = serde_json::from_str(&blocks_json).unwrap();

    writeln!(
        &mut block_type_to_texture_coordinates_file,
         "{}\npub static BLOCK_TYPE_TO_TEXTURE_COORDINATES: phf::Map<BlockType, [TextureCoordinates; 6]> = \n{};\n",
         get_imports(),
         get_map(&vec_block_format)
    ).unwrap();

    vertex_builder::build_vertex_file();
    shader_builder::build_shader_file();
    frustum_compute_builder::build_compute_file();
}

fn get_imports() -> String {
    String::from([
        "use fundamentals::enums::block_type::BlockType;",
        "use fundamentals::texture_coords::TextureCoordinates;"
    ].join("\n"))
}

fn get_map(vec_block_format: &Vec<formats::block_format::BlockFormat>) -> String {
    let mut map = phf_codegen::Map::new();
    for block in vec_block_format {
        let texture_coords = STRING_TO_TEXTURE_COORDINATES.get(&block.block_type).unwrap();

        let mut tex_arr = Vec::new();

        for i in 0..6 {
            tex_arr.push((texture_coords[i].tx.to_string(), texture_coords[i].ty.to_string()));
        }

        let entries = tex_arr.iter().map(|(tx,ty)| format!("TextureCoordinates {{ tx: {}, ty: {} }}", tx, ty)).collect::<Vec<String>>().join(",");

        map.entry(STRING_TO_BLOCK_TYPE.get(&block.block_type).unwrap(), &format!("[{}]", entries));
    }
    map.build().to_string()
}