use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ConfigFormat {
    pub num_additional_threads_specified: usize,
    pub use_all_system_threads: bool,
    pub render_radius: usize,
    pub atlas_max_images_on_a_row: u32,
    pub atlas_max_images_on_a_column: u32,
    pub max_amount_of_blocktypes: u32,
    pub max_lighting_level: u8,
    pub number_of_light_colors: u32,
    pub number_of_alpha_values: u8,
    pub texture_dimension: u32,
    pub chunk_dimension: u8,
    pub mesh_bucket_size: u32,
    pub workgroup_size: u16,
}