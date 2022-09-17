use formats::formats::texture_format::TextureFormat;
use ::formats::formats::{block_format::BlockFormat, config_format::ConfigFormat};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use image::GenericImage;

pub struct AtlasBuilder {
    pub atlas_index_width: u32,
    pub atlas_index_height: u32,
    pub texture_width_str: String,
    pub texture_height_str: String,
    pub block_string_to_texture_coords: Vec<(String, [(u32, u32);6])>,
}

impl AtlasBuilder {
    pub fn build_and_save_atlas(vec_block_format: &Vec<BlockFormat>, config_format: &ConfigFormat, atlas_path: &Path) -> Self {
        let mut image_coord_x = 0;
        let mut image_coord_y = 0;
    
        let blocks = vec_block_format.iter().map(|bf| (bf.block_type.to_string(), &bf.texture)).collect::<Vec<(String, &TextureFormat)>>();
        let mut block_string_to_texture_indices = Vec::new();
        let mut texture_string_to_texture_indices = HashMap::new();
    
        let mut set_of_textures = HashSet::new();

        for (_, block_textures) in blocks.iter() {
            for block_texture in block_textures.to_vec() {
                set_of_textures.insert(block_texture);
            }
        }

        let atlas_num_images_width_max = 2_u32.pow((set_of_textures.len() as f32).log(4.0).ceil() as u32); // Calculates the minimum power of 2 square that can fit
        let altas_num_images_height_max = atlas_num_images_width_max;
    
        let mut texture_vec = Vec::new();
        
        for (block_name, block_textures) in blocks {
            let mut block_texture_indices = [(0,0);6];
            let mut i = 0;
            for block_texture in block_textures.to_vec() {
                if !texture_string_to_texture_indices.contains_key(&block_texture) {
                    texture_string_to_texture_indices.insert(block_texture.clone(), (image_coord_x, image_coord_y));
                    let block_texture = format!("../resources/{}", block_texture);
                    let texture_path = Path::new(&block_texture);
                    let texture = image::io::Reader::open(texture_path).unwrap().decode().unwrap();
                    block_texture_indices[i] = (image_coord_x, image_coord_y);
            
                    texture_vec.push(((image_coord_x, image_coord_y), texture));

                    image_coord_x += 1;

                    if image_coord_x % atlas_num_images_width_max == 0 {
                        image_coord_y += 1;
                        image_coord_x = 0;
                    }
                } else {
                    block_texture_indices[i] = *texture_string_to_texture_indices.get(&block_texture).unwrap();
                }
                
                i += 1;
            }
            block_string_to_texture_indices.push((block_name.clone(), block_texture_indices));
            
        }

        let mut atlas_buf = <image::ImageBuffer<image::Rgba<u8>, _>>::new(atlas_num_images_width_max*config_format.texture_dimension, altas_num_images_height_max*config_format.texture_dimension);
    
        for ((tix,tiy), texture) in texture_vec {
            atlas_buf.copy_from(&texture, tix*config_format.texture_dimension, tiy*config_format.texture_dimension).unwrap();
        }
    
        atlas_buf.save_with_format(atlas_path, image::ImageFormat::Png).unwrap();
    
        let texture_width = 1.0 / atlas_num_images_width_max as f32;
        let mut texture_width_str = texture_width.to_string();
        if texture_width == 1.0 {
            texture_width_str = String::from("1.0");
        }
    
        let texture_height = 1.0 / altas_num_images_height_max as f32;
        let mut texture_height_str = texture_height.to_string();
        if texture_height == 1.0 {
            texture_height_str = String::from("1.0");
        }
    
        let block_string_to_texture_coords = block_string_to_texture_indices;

        AtlasBuilder { 
            atlas_index_height: altas_num_images_height_max,
            atlas_index_width: atlas_num_images_width_max, 
            texture_width_str, 
            texture_height_str, 
            block_string_to_texture_coords 
        }
    }
}

