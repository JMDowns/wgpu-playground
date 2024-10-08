use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ConfigFormat {
    pub num_additional_threads_specified: usize,
    pub use_all_system_threads: bool,
    pub render_radius: usize,
    pub max_amount_of_blocktypes: u32,
    pub texture_dimension: u32,
    pub chunk_dimension: u8,
    pub num_tasks_per_thread: usize,
    pub task_priorities: TaskPriorities,
    pub movement_speed: f32,
    pub mesh_method: String,
    pub chunk_generation_method: String,
    pub chunk_generation_options: ChunkGenerationOptions,
    pub min_memory_mb: u32,
    pub max_memory_mb: u32,
    pub max_subvoxel_objects: u32,
    pub max_subvoxels_in_models: u32,
    pub max_subvoxel_colors: usize,
    pub max_grid_aligned_subvoxel_objects: u32,
    pub grid_aligned_subvoxel_placement_dimension: u32
}

#[derive(Serialize, Deserialize)]
pub struct TaskPriorities {
    pub chunk: usize,
    pub update_chunk_padding_x: usize,
    pub update_chunk_padding_y: usize,
    pub update_chunk_padding_z: usize,
    pub mesh: usize,
    pub mesh_side: usize
}

#[derive(Serialize, Deserialize)]
pub struct ChunkGenerationOptions {
    pub perlin_positive_threshold: f32,
    pub perlin_negative_threshold: f32,
    pub perlin_scale_factor: f32,
}