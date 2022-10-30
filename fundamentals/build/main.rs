use ::formats::formats::{config_format::ConfigFormat, block_format::BlockFormat, controls_format::ControlsFormat};
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

    let controls_json = std::fs::read_to_string("../data/controls.json").unwrap();
    let controls_format: ControlsFormat = serde_json::from_str(&controls_json).unwrap();

    let atlas_builder = atlas_builder::AtlasBuilder::build_and_save_atlas(&vec_block_format, &config_format);

    enums::build_enums(&vec_block_format);

    string_to_type_dict_builders::build_string_to_type_dictionaries(&vec_block_format, &atlas_builder.block_string_to_texture_indices);

    let num_block_types = (vec_block_format.len()+1) as u16;
    let consts_model = consts::ConstsModel {
        num_block_types,
        atlas_max_num_images_height: atlas_builder.atlas_index_height,
        atlas_max_num_images_width: atlas_builder.atlas_index_width,
        num_textures: atlas_builder.num_textures,
    };
    consts::generate_consts(&config_format, &consts_model, &controls_format);
}