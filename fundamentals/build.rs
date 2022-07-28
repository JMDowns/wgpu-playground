use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use formats::formats;
use ::formats::formats::config_format::ConfigFormat;
use serde_json;
use image::GenericImage;
use std::collections::HashMap;

fn main() {
    let blocks_json = std::fs::read_to_string("../data/blocks.json").unwrap();
    let vec_block_format: Vec<formats::block_format::BlockFormat> = serde_json::from_str(&blocks_json).unwrap();

    let config_json = std::fs::read_to_string("../data/config.json").unwrap();
    let config_format: formats::config_format::ConfigFormat = serde_json::from_str(&config_json).unwrap();

    let block_type_names = vec_block_format.iter().map(|ef| format!("{},", ef.block_type.replace(" ", "_").to_uppercase())).collect::<Vec<String>>();
    let block_type_reference_strings = block_type_names.iter().map(|n| format!("BlockType::{}", n)).collect::<Vec<String>>();
    let num_block_types = block_type_names.len()+1;
    let path = Path::new("src/enums/block_type.rs");
    let mut block_types_file = BufWriter::new(File::create(&path).unwrap());

    let consts_path = Path::new("src/consts.rs");
    let mut consts_file = BufWriter::new(File::create(&consts_path).unwrap());

    let string_to_block_type_path = Path::new("../string_to_type_dictionaries/build.rs");
    let mut string_to_block_type_file = BufWriter::new(File::create(&string_to_block_type_path).unwrap());

    

    writeln!(
        &mut block_types_file,
         "{}\n{}\n{}",
         build_block_type_imports(),
         build_enum(&block_type_names),
         build_traits()
    ).unwrap();

    let string_to_block_types_file_name = String::from("string_to_block_types_file");
    let string_to_texture_indices_name = String::from("string_to_texture_indices_file");
    let lib_file_name = String::from("lib_file");

    let atlas_path = Path::new("../hello-wgpu/src/atlas.png");

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
                }
            } else {
                block_texture_indices[i] = *texture_string_to_texture_indices.get(&block_texture).unwrap();
            }
            
            i += 1;
        }
        block_string_to_texture_indices.push((block_name.clone(), block_texture_indices));
        
    }

    let atlas_index_width = match image_coord_y {
        0 => image_coord_x,
        _ => 64
    };

    let atlas_index_height = image_coord_y + 1;

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

    let block_string_to_texture_coords = block_string_to_texture_indices.iter().map(|(s, t_arr)| (s, t_arr.map(|(tix, tiy)| (tix as f32 / atlas_index_width as f32, tiy as f32 / atlas_index_height as f32)))).collect();

    writeln!(
        &mut consts_file,
        "{}",
        [
            format!("pub const NUM_BLOCK_TYPES: u16 = {};", num_block_types),
            format!("pub const NUM_THREADS: usize = {};", generate_num_threads(&config_format)),
            format!("pub const RENDER_DISTANCE: usize = {};", config_format.render_radius),
            format!("pub const TEXTURE_WIDTH: f32 = {};", texture_width_str),
            format!("pub const TEXTURE_HEIGHT: f32 = {};", texture_height_str),
        ].join("\n")
    ).unwrap();

    writeln!(
        &mut string_to_block_type_file,
         "{}\nfn main() {{\n{}\n}}",
         build_string_to_block_type_imports(),
         [
            build_file_creates(&string_to_block_types_file_name, &string_to_texture_indices_name),
            build_lib_file(&lib_file_name),
            build_string_to_block_type_dictionary_builder(&string_to_block_types_file_name, &vec_block_format, &block_type_reference_strings),
            build_string_to_texture_indices_dictionary_builder(&string_to_texture_indices_name, &block_string_to_texture_coords)
         ].join("\n")
    ).unwrap();
}

fn generate_num_threads(cf: &ConfigFormat) -> usize {
    if cf.use_all_system_threads {
        num_cpus::get()
    } else {
        cf.num_threads_specified
    }
}

fn build_block_type_imports() -> String {
    String::from([
        "use num_derive::FromPrimitive;",
        "use phf_shared;",
        "use std::fmt;",
        "use core::hash::{Hash, Hasher};"
    ].join("\n"))
}

fn build_file_creates(string_to_block_types_file_name: &String, string_to_texture_indices_file_name: &String) -> String {
    String::from([
        "let lib_path = Path::new(\"src/lib.rs\");",
        "let string_to_block_types_path = Path::new(\"src/string_to_block_type.rs\");",
        "let string_to_texture_indices_path = Path::new(\"src/string_to_texture_indices.rs\");",
        "let mut lib_file = BufWriter::new(File::create(&lib_path).unwrap());",
        &format!("let mut {} = BufWriter::new(File::create(&string_to_block_types_path).unwrap());", string_to_block_types_file_name),
        &format!("let mut {} = BufWriter::new(File::create(&string_to_texture_indices_path).unwrap());", string_to_texture_indices_file_name),
    ].join("\n"))
}

fn build_string_to_block_type_imports() -> String {
    String::from([
        "use std::fs::File;",
        "use std::io::{BufWriter, Write};",
        "use std::path::Path;",
    ].join("\n"))
}

fn build_string_to_block_type_dictionary_builder(string_to_block_type_dictionary_file_name: &String, block_values: &Vec<formats::block_format::BlockFormat>, block_type_reference_strings: &Vec<String>) -> String {
    let block_names = block_values.iter().map(|b| &b.block_type);
    let map_entries = block_names.zip(block_type_reference_strings).collect::<Vec<(&String, &String)>>();
    String::from(
        format!("writeln!(\n\t&mut {},\n{},\n{}\n\t).unwrap();", 
        string_to_block_type_dictionary_file_name,
        "\t\"use fundamentals::enums::block_type::BlockType;\\npub static STRING_TO_BLOCK_TYPE: phf::Map<&str, BlockType> = \\n{};\\n\"",
        generate_string_to_block_type_map(map_entries)),
        )
}

fn build_string_to_texture_indices_dictionary_builder(string_to_texture_indices_file_name: &String, block_string_to_texture_coords: &Vec<(&String, [(f32, f32);6])>) -> String {
    String::from(
        format!("writeln!(\n\t&mut {},\n{},\n{}\n\t).unwrap();", 
        string_to_texture_indices_file_name,
        "\t\"use fundamentals::enums::block_type::BlockType;\nuse fundamentals::texture_coords::TextureCoordinates;\\npub static STRING_TO_TEXTURE_COORDINATES: phf::Map<&str, [TextureCoordinates; 6]> = \\n{};\\n\"",
        generate_string_to_texture_indices_map(block_string_to_texture_coords)),
        )
}

fn generate_string_to_texture_indices_map(block_string_to_texture_coords: &Vec<(&String, [(f32, f32);6])>) -> String {
    let mut lines = Vec::new();
    lines.push("\tphf_codegen::Map::new()".to_string());
    for (string_name, texture_coords) in block_string_to_texture_coords {
        let mut texture_coord_strings = [(String::new(), String::new()), (String::new(), String::new()),(String::new(), String::new()),(String::new(), String::new()),(String::new(), String::new()),(String::new(), String::new())];
        for i in 0..6 {
            let tx = texture_coords[i].0;
            let ty = texture_coords[i].1;
            let mut string_tx = tx.to_string();
            if tx == 0.0 {
                string_tx = String::from("0.0");
            }
            let mut string_ty = ty.to_string();
            if ty == 0.0 {
                string_ty = String::from("0.0");
            }
            texture_coord_strings[i] = (string_tx.clone(), string_ty.clone());
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

fn build_lib_file(lib_file_name: &String) -> String {
    String::from(
        format!("writeln!(\n\t&mut {},\n{}\n\t).unwrap();",
        lib_file_name,
        [
            "\"{}{}\",",
        "\"pub mod string_to_block_type;\",",
        "\"pub mod string_to_texture_indices;\"",
        ].join("\n")
        ))
}

fn generate_string_to_block_type_map(entries: Vec<(&String, &String)>) -> String {
    let mut lines = Vec::new();
    lines.push("\tphf_codegen::Map::new()".to_string());
    for (string_name, block_type_name) in entries {
        let new_line = format!(".entry(\"{}\", \"{}\")", string_name, block_type_name);
        lines.push(new_line);
    }
    lines.push(".build()".to_string());
    lines.join("\n\t\t")
}

fn build_enum(block_type_enum_values: &Vec<String>) -> String {
    format!("#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, FromPrimitive)]\npub enum BlockType\n{{\n\tAIR = 0,\n\t{}\n}}\n",
        block_type_enum_values.join("\n\t"))
}

fn build_traits() -> String {
    String::from(
["impl phf_shared::FmtConst for BlockType {
    fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, \"BlockType::{:?}\", self)
    }
}",
"impl phf_shared::PhfHash for BlockType {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        (*self as u8).hash(state);
    }
}",
"impl phf_shared::PhfBorrow<BlockType> for BlockType {
    fn borrow(&self) -> &BlockType {
        self
    }
}"].join("\n")
    )
}