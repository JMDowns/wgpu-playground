use ::formats::formats::{block_format::BlockFormat, config_format::ConfigFormat};
use std::collections::HashMap;
use std::path::Path;
use image::GenericImage;

pub struct AtlasBuilder {
    pub atlas_index_width: u32,
    pub atlas_index_height: u32,
    pub texture_width_str: String,
    pub texture_height_str: String,
    pub block_string_to_texture_coords: Vec<(String, [(f32, f32);6])>,
}

impl AtlasBuilder {
    pub fn build_and_save_atlas(vec_block_format: &Vec<BlockFormat>, config_format: &ConfigFormat, atlas_path: &Path) -> Self {
        let mut image_coord_x = 0;
        let mut image_coord_y = 0;
    
        let blocks = vec_block_format.iter().map(|bf| (bf.block_type.to_string(), &bf.texture));
        let mut block_string_to_texture_indices = Vec::new();
        let mut texture_string_to_texture_indices = HashMap::new();
    
        let atlas_num_images_width_max = config_format.atlas_max_images_on_a_row;
    
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

        println!("{:?}", block_string_to_texture_indices);
        
        let atlas_index_width = match image_coord_y {
            0 => image_coord_x,
            _ => config_format.atlas_max_images_on_a_row
        };
    
        let atlas_index_height = match image_coord_y {
            0 => 1,
            // This is done because image_coord_y will increase before a block is placed in a row, causing an extra row of empty textures
            // if the number of blocks is perfectly divided by the number of blocks per row
            h => match image_coord_x % atlas_index_width {
                0 => h,
                _ => h+1
            }
        };

        let mut atlas_buf = <image::ImageBuffer<image::Rgba<u8>, _>>::new(atlas_index_width*config_format.texture_dimension, atlas_index_height*config_format.texture_dimension);
    
        for ((tix,tiy), texture) in texture_vec {
            atlas_buf.copy_from(&texture, tix*config_format.texture_dimension, tiy*config_format.texture_dimension).unwrap();
        }
    
        atlas_buf.save_with_format(atlas_path, image::ImageFormat::Png).unwrap();
    
        let texture_width = 1.0 / atlas_index_width as f32;
        let mut texture_width_str = texture_width.to_string();
        if texture_width == 1.0 {
            texture_width_str = String::from("1.0");
        }
    
        let texture_height = 1.0 / atlas_index_height as f32;
        let mut texture_height_str = texture_height.to_string();
        if texture_height == 1.0 {
            texture_height_str = String::from("1.0");
        }
    
        let block_string_to_texture_coords = block_string_to_texture_indices.iter().map(|(s, t_arr)| (s.clone(), t_arr.map(|(tix, tiy)| (tix as f32 / atlas_index_width as f32, tiy as f32 / (atlas_index_height as f32))))).collect();

        AtlasBuilder { 
            atlas_index_width, 
            atlas_index_height, 
            texture_width_str, 
            texture_height_str, 
            block_string_to_texture_coords 
        }
    }
}

