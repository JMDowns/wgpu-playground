use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use string_to_type_dictionaries::string_to_block_type::STRING_TO_BLOCK_TYPE;
use string_to_type_dictionaries::string_to_texture_indices::STRING_TO_TEXTURE_INDICES;
use formats::formats;
use phf_codegen;
use serde_json;
mod vertex_builder;
mod shader_builder;
mod frustum_compute_builder;
mod block_builder;
mod compute_state_helper_builder;
mod occlusion_shader_builder;

fn main() {
    let block_type_to_texture_coordinates_path = Path::new("src/dictionaries/").join("block_type_to_texture_coordinates.rs");
    let mut block_type_to_texture_coordinates_file = BufWriter::new(File::create(&block_type_to_texture_coordinates_path).unwrap());

    let blocks_json = std::fs::read_to_string("../data/blocks.json").unwrap();
    let vec_block_format: Vec<formats::block_format::BlockFormat> = serde_json::from_str(&blocks_json).unwrap();

    writeln!(
        &mut block_type_to_texture_coordinates_file,
         "{}\npub static BLOCK_TYPE_TO_TEXTURE_INDICES: phf::Map<BlockType, [usize; 6]> = \n{};\n",
         get_imports(),
         get_map(&vec_block_format)
    ).unwrap();

    vertex_builder::build_vertex_file();
    shader_builder::build_shader_file();
    frustum_compute_builder::build_compute_file();
    compute_state_helper_builder::build_compute_helper_file();
    occlusion_shader_builder::build_occlusion_shader_file();

    block_builder::build_block_file();
}

fn get_imports() -> String {
    String::from([
        "use fundamentals::enums::block_type::BlockType;",
    ].join("\n"))
}

fn get_map(vec_block_format: &Vec<formats::block_format::BlockFormat>) -> String {
    let mut map = phf_codegen::Map::new();
    for block in vec_block_format {
        let texture_coords = STRING_TO_TEXTURE_INDICES.get(&block.block_type).unwrap();

        map.entry(STRING_TO_BLOCK_TYPE.get(&block.block_type).unwrap(), &format!("{:?}", texture_coords));
    }
    map.build().to_string()
}