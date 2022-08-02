use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ConfigFormat {
    pub num_threads_specified: usize,
    pub use_all_system_threads: bool,
    pub render_radius: usize,
    pub atlas_max_images_on_a_row: u32,
    pub texture_dimension: u32,
    pub chunk_dimension: u8
}