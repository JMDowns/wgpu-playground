use crate::world_position::WorldPosition;
use winit::event::VirtualKeyCode;

pub const NUM_BLOCK_TYPES: u16 = 5;
pub const NUM_ADDITIONAL_THREADS: usize = 15;
pub const RENDER_DISTANCE: usize = 1;
pub const FOV_DISTANCE: usize = 32;
pub const CHUNK_DIMENSION: i32 = 32;
pub const CHUNK_PLANE_SIZE: i32 = 1024;
pub const CHUNK_SIZE: usize = 32768;
pub const CHUNK_DIMENSION_WRAPPED: usize = 34;
pub const CHUNK_PLANE_SIZE_WRAPPED: usize = 1156;
pub const CHUNK_SIZE_WRAPPED: usize = 39304;
pub const BITS_PER_POSITION: u32 = 6;
pub const TEXTURE_DIMENSION: u32 = 16;
pub const NUM_TEXTURES: usize = 5;
pub const TEX_MAX_X: u32 = 4;
pub const TEX_MAX_Y: u32 = 4;
pub const BITS_PER_TEX_COORD_X: u32 = 3;
pub const BITS_PER_TEX_COORD_Y: u32 = 3;
pub const BITS_PER_AMBIENT_OCCLUSION: u32 = 2;
pub const NUMBER_OF_CHUNKS_AROUND_PLAYER: u32 = 7;
pub const NUMBER_OF_CHUNKS_TO_RENDER: u32 = 1;
pub const BITS_PER_CHUNK_INDEX: u32 = 3;
pub const WORKGROUP_SIZE: u16 = 255;
pub const NUM_VERTICES_IN_BUCKET: u32 = 4096;
pub const NUM_BUCKETS_PER_CHUNK: usize = 64;
pub const NUM_BUCKETS_PER_SIDE: u32 = 10;
pub const NUM_BUCKETS: usize = 448;
pub const MESH_METHOD: &str = "greedy";
pub const CHUNK_GENERATION_METHOD: &str = "perlin";
pub const PERLIN_POSITIVE_THRESHOLD: f64 = 0.2;
pub const PERLIN_NEGATIVE_THRESHOLD: f64 = -0.2;
pub const PERLIN_SCALE_FACTOR: f64 = 0.02;

pub const MIP_LEVEL: u32 = 4;
pub const TEXTURE_LENGTH_WITH_MIPMAPS: usize = 341;

pub const UP_KEY: VirtualKeyCode = VirtualKeyCode::Space;
pub const DOWN_KEY: VirtualKeyCode = VirtualKeyCode::LShift;
pub const LEFT_KEY: VirtualKeyCode = VirtualKeyCode::A;
pub const RIGHT_KEY: VirtualKeyCode = VirtualKeyCode::D;
pub const FORWARD_KEY: VirtualKeyCode = VirtualKeyCode::W;
pub const BACKWARD_KEY: VirtualKeyCode = VirtualKeyCode::S;
pub const MOUSE_SENSITIVITY_THRESHOLD: f64 = 0.5;
pub const MOUSE_SENSITIVITY: f32 = 0.8;

pub const NUM_TASKS_PER_THREAD: usize = 15;
pub const GENERATE_CHUNK_PRIORITY: u32 = 1;
pub const UPDATE_CHUNK_PADDING_X_PRIORITY: u32 = 2;
pub const UPDATE_CHUNK_PADDING_Y_PRIORITY: u32 = 2;
pub const UPDATE_CHUNK_PADDING_Z_PRIORITY: u32 = 2;
pub const GENERATE_MESH_PRIORITY: u32 = 4;
pub const GENERATE_MESH_SIDE_PRIORITY: u32 = 3;

pub const MOVEMENT_SPEED: f32 = 10.0;
pub const MIN_MEMORY_USAGE_MB: u32 = 512;
pub const MAX_MEMORY_USAGE_MB: u32 = 10240;
pub fn get_positions_around_player(pos: WorldPosition) -> Vec<WorldPosition> {
    vec![
        WorldPosition::new(pos.x+(0), pos.y+(0), pos.z+(0)),
        WorldPosition::new(pos.x+(-1), pos.y+(0), pos.z+(0)),
        WorldPosition::new(pos.x+(1), pos.y+(0), pos.z+(0)),
        WorldPosition::new(pos.x+(0), pos.y+(1), pos.z+(0)),
        WorldPosition::new(pos.x+(0), pos.y+(-1), pos.z+(0)),
        WorldPosition::new(pos.x+(0), pos.y+(0), pos.z+(1)),
        WorldPosition::new(pos.x+(0), pos.y+(0), pos.z+(-1))
    ]
}
