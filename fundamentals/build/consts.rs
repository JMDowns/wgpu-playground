use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use formats::formats::config_format::ConfigFormat;

pub fn generate_consts(config_format: &ConfigFormat, consts_model: &ConstsModel) {
    let consts_path = Path::new("src/consts.rs");
    let mut consts_file = BufWriter::new(File::create(&consts_path).unwrap());
    writeln!(
        &mut consts_file,
        "{}",
        [
            format!("pub const NUM_BLOCK_TYPES: u16 = {};", consts_model.num_block_types),
            format!("pub const NUM_THREADS: usize = {};", generate_num_threads(&config_format)),
            format!("pub const RENDER_DISTANCE: usize = {};", config_format.render_radius),
            format!("pub const CHUNK_DIMENSION: i32 = {};", config_format.chunk_dimension),
            format!("pub const CHUNK_PLANE_SIZE: i32 = {};", (config_format.chunk_dimension as u32)*(config_format.chunk_dimension as u32)),
            format!("pub const CHUNK_SIZE: usize = {};", (config_format.chunk_dimension as u32)*(config_format.chunk_dimension as u32)*(config_format.chunk_dimension as u32)),
            format!("pub const BITS_PER_POSITION: u8 = {};", ((config_format.chunk_dimension+1) as f32).log2().ceil() as u8),
            format!("pub const TEX_MAX_X: u8 = {};", config_format.atlas_max_images_on_a_row),
            format!("pub const TEX_MAX_Y: u8 = {};", config_format.atlas_max_images_on_a_column),
            format!("pub const BITS_PER_TEX_COORD_X: u8 = {};", (((config_format.atlas_max_images_on_a_row + 1) as f32).log2().ceil())),
            format!("pub const BITS_PER_TEX_COORD_Y: u8 = {};", (((config_format.atlas_max_images_on_a_column + 1) as f32).log2().ceil())),
            format!("pub const BITS_PER_AMBIENT_OCCLUSION: u8 = 2;"),
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

pub struct ConstsModel<'a> {
    pub num_block_types: u16,
    pub texture_width_str: &'a str,
    pub texture_height_str: &'a str,
}