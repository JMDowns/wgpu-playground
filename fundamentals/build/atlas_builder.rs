use formats::formats::texture_format::TextureFormat;
use ::formats::formats::{block_format::BlockFormat, config_format::ConfigFormat};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::Path;

pub struct AtlasBuilder {
    pub atlas_index_width: u32,
    pub atlas_index_height: u32,
    pub block_string_to_texture_indices: Vec<(String, [usize;6])>,
    pub num_textures: usize,
}

impl AtlasBuilder {
    pub fn build_and_save_atlas(vec_block_format: &Vec<BlockFormat>, config_format: &ConfigFormat) -> Self {
        let mut image_index = 0;
    
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
        
        let mip_level = (config_format.texture_dimension as f32).log2() as usize;

        for (block_name, block_textures) in blocks {
            let mut block_texture_indices = [0;6];
            let mut i = 0;
            for block_texture in block_textures.to_vec() {
                if !texture_string_to_texture_indices.contains_key(&block_texture) {
                    texture_string_to_texture_indices.insert(block_texture.clone(), image_index);
                    let block_texture = format!("../resources/{}", block_texture);
                    let texture_path = Path::new(&block_texture);
                    let texture = image::io::Reader::open(texture_path).unwrap().decode().unwrap();
                    let mut mip_texture_vec = vec![texture.clone()];
                    for i in 1..mip_level as u32+1 {
                        let texture_downsized = texture.resize(texture.width() / 2u32.pow(i), texture.height() / 2u32.pow(i), image::imageops::FilterType::Nearest);
                        mip_texture_vec.push(texture_downsized);
                    }
                    block_texture_indices[i] = image_index;
            
                    texture_vec.push((image_index, mip_texture_vec));

                    image_index += 1;
                } else {
                    block_texture_indices[i] = *texture_string_to_texture_indices.get(&block_texture).unwrap();
                }
                
                i += 1;
            }
            block_string_to_texture_indices.push((block_name.clone(), block_texture_indices));
            
        }
    
        let num_textures = texture_vec.len();

        let mut data_buf: Vec<u8> = Vec::new();

        for (_, mip_texture_vec) in texture_vec {
            for texture in mip_texture_vec {
                data_buf.extend_from_slice(&texture.to_rgba8().as_flat_samples().as_slice());
            }
        }

        let mut data_atl = std::fs::File::create("../hello-wgpu/src/data.atl").expect("Unable to create file");
        data_atl.write_all(&data_buf).expect("Unable to create file");

        AtlasBuilder { 
            atlas_index_height: altas_num_images_height_max,
            atlas_index_width: atlas_num_images_width_max,
            block_string_to_texture_indices,
            num_textures
        }
    }
}

