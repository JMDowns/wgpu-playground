use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::vec;
use formats::formats::config_format::ConfigFormat;

pub fn generate_consts(config_format: &ConfigFormat, consts_model: &ConstsModel) {
    let consts_path = Path::new("src/consts.rs");
    let mut consts_file = BufWriter::new(File::create(&consts_path).unwrap());

    let position_offset_vec = generate_chunk_pos_around_player_fn(config_format);
    writeln!(
        &mut consts_file,
        "{}",
        [
            String::from("use crate::world_position::WorldPosition;\n"),
            format!("pub const NUM_BLOCK_TYPES: u16 = {};", consts_model.num_block_types),
            format!("pub const NUM_THREADS: usize = {};", generate_num_threads(&config_format)),
            format!("pub const RENDER_DISTANCE: usize = {};", config_format.render_radius),
            format!("pub const CHUNK_DIMENSION: i32 = {};", config_format.chunk_dimension),
            format!("pub const CHUNK_PLANE_SIZE: i32 = {};", (config_format.chunk_dimension as u32)*(config_format.chunk_dimension as u32)),
            format!("pub const CHUNK_SIZE: usize = {};", (config_format.chunk_dimension as u32)*(config_format.chunk_dimension as u32)*(config_format.chunk_dimension as u32)),
            format!("pub const BITS_PER_POSITION: u32 = {};", ((config_format.chunk_dimension+1) as f32).log2().ceil() as u8),
            format!("pub const TEX_MAX_X: u32 = {};", config_format.atlas_max_images_on_a_row),
            format!("pub const TEX_MAX_Y: u32 = {};", config_format.atlas_max_images_on_a_column),
            format!("pub const BITS_PER_TEX_COORD_X: u32 = {};", (((config_format.atlas_max_images_on_a_row + 1) as f32).log2().ceil())),
            format!("pub const BITS_PER_TEX_COORD_Y: u32 = {};", (((config_format.atlas_max_images_on_a_column + 1) as f32).log2().ceil())),
            format!("pub const BITS_PER_AMBIENT_OCCLUSION: u32 = 2;"),
            format!("pub const NUMBER_OF_CHUNKS_AROUND_PLAYER: u32 = {};", position_offset_vec.len()),
            format!("pub const BITS_PER_CHUNK_INDEX: u32 = {};", ((position_offset_vec.len()) as f32).log2().ceil() as u32),
            String::new(),
            generate_string_from_position_offsets(position_offset_vec),
        ].join("\n")
    ).unwrap();
}

fn generate_chunk_pos_around_player_fn(config_format: &ConfigFormat) -> Vec<(i32, i32, i32)> {
    let mut vec_of_position_offsets = Vec::new();
    let radius = config_format.render_radius as i32;
    for x in -radius..radius + 1 {
        for y in -radius..radius + 1 {
            for z in -radius..radius + 1 {
                let vertex_positions = [
                (x, y, z),
                (x, y, z+1),
                (x, y+1, z),
                (x, y+1, z+1),
                (x+1, y, z),
                (x+1, y, z+1),
                (x+1, y+1, z),
                (x+1, y+1, z+1)
                ];

                let mut is_in_radius = true;
                for (x,y,z) in vertex_positions {
                    if ((x.pow(3)+y.pow(3)+z.pow(3)) as f32).cbrt() > (radius + 2) as f32 {
                        is_in_radius = false;
                    }
                }

                if is_in_radius {
                    vec_of_position_offsets.push((x,y,z));
                }
            }
        }
    }

    vec_of_position_offsets
}

fn generate_string_from_position_offsets(vec_of_position_offsets: Vec<(i32,i32,i32)>) -> String {
    let mut pos_new_vec = Vec::new();
    for (x_off,y_off,z_off) in vec_of_position_offsets {
        pos_new_vec.push(format!("WorldPosition::new(pos.x+({x_off}), pos.y+({y_off}), pos.z+({z_off}))"));
    }

    [
    format!("pub fn get_positions_around_player(pos: WorldPosition) -> [WorldPosition; {}] {{", pos_new_vec.len()).as_str(),
    "    [",
    format!("        {}",pos_new_vec.join(",\n        ")).as_str(),
    "    ]",
    "}"
    ].join("\n")
}

fn generate_num_threads(cf: &ConfigFormat) -> usize {
    if cf.use_all_system_threads {
        num_cpus::get()
    } else {
        cf.num_threads_specified
    }
}

pub struct ConstsModel<'a> {
    pub num_block_types: u16,
    pub texture_width_str: &'a str,
    pub texture_height_str: &'a str,
}