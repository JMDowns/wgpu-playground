use std::path::Path;
use ::formats::formats::{config_format::ConfigFormat, block_format::BlockFormat};
use serde_json;
mod atlas_builder;
mod string_to_type_dict_builders;
mod enums;
mod consts;

fn main() {
    let blocks_json = std::fs::read_to_string("../data/blocks.json").unwrap();
    let vec_block_format: Vec<BlockFormat> = serde_json::from_str(&blocks_json).unwrap();

    let config_json = std::fs::read_to_string("../data/config.json").unwrap();
    let config_format: ConfigFormat = serde_json::from_str(&config_json).unwrap();

    let atlas_path = Path::new("../hello-wgpu/src/atlas.png");
    let atlas_builder = atlas_builder::AtlasBuilder::build_and_save_atlas(&vec_block_format, &config_format, atlas_path);

    enums::build_enums(&vec_block_format);

    string_to_type_dict_builders::build_string_to_type_dictionaries(&vec_block_format, &atlas_builder.block_string_to_texture_coords);

    let num_block_types = (vec_block_format.len()+1) as u16;
    let consts_model = consts::ConstsModel {
        num_block_types,
        texture_width_str: &atlas_builder.texture_width_str,
        texture_height_str: &atlas_builder.texture_height_str
    };
    consts::generate_consts(&config_format, &consts_model);
}