use crate::world_position::WorldPosition;
use winit::event::VirtualKeyCode;

pub const NUM_BLOCK_TYPES: u16 = 3;
pub const NUM_ADDITIONAL_THREADS: usize = 1;
pub const RENDER_DISTANCE: usize = 2;
pub const FOV_DISTANCE: usize = 64;
pub const CHUNK_DIMENSION: i32 = 32;
pub const CHUNK_PLANE_SIZE: i32 = 1024;
pub const CHUNK_SIZE: usize = 32768;
pub const CHUNK_DIMENSION_WRAPPED: usize = 34;
pub const CHUNK_PLANE_SIZE_WRAPPED: usize = 1156;
pub const CHUNK_SIZE_WRAPPED: usize = 39304;
pub const BITS_PER_POSITION: u32 = 6;
pub const TEXTURE_DIMENSION: u32 = 16;
pub const NUM_TEXTURES: usize = 2;
pub const TEX_MAX_X: u32 = 2;
pub const TEX_MAX_Y: u32 = 2;
pub const BITS_PER_TEX_COORD_X: u32 = 2;
pub const BITS_PER_TEX_COORD_Y: u32 = 2;
pub const BITS_PER_AMBIENT_OCCLUSION: u32 = 2;
pub const NUMBER_OF_CHUNKS_AROUND_PLAYER: u32 = 57;
pub const NUMBER_OF_CHUNKS_TO_RENDER: u32 = 8;
pub const BITS_PER_CHUNK_INDEX: u32 = 6;
pub const WORKGROUP_SIZE: u16 = 255;
pub const NUM_VERTICES_IN_BUCKET: u32 = 65536;
pub const NUM_BUCKETS_PER_CHUNK: usize = 12;
pub const NUM_BUCKETS_PER_SIDE: u32 = 2;
pub const NUM_BUCKETS: usize = 684;

pub const MIP_LEVEL: u32 = 4;
pub const TEXTURE_LENGTH_WITH_MIPMAPS: usize = 341;

pub const UP_KEY: VirtualKeyCode = VirtualKeyCode::Space;
pub const DOWN_KEY: VirtualKeyCode = VirtualKeyCode::LShift;
pub const LEFT_KEY: VirtualKeyCode = VirtualKeyCode::A;
pub const RIGHT_KEY: VirtualKeyCode = VirtualKeyCode::D;
pub const FORWARD_KEY: VirtualKeyCode = VirtualKeyCode::W;
pub const BACKWARD_KEY: VirtualKeyCode = VirtualKeyCode::S;
pub const MOUSE_SENSITIVITY: f64 = 1.0;

pub fn get_positions_around_player(pos: WorldPosition) -> Vec<WorldPosition> {
    vec![
        WorldPosition::new(pos.x+(0), pos.y+(0), pos.z+(0)),
        WorldPosition::new(pos.x+(-1), pos.y+(-1), pos.z+(-1)),
        WorldPosition::new(pos.x+(-1), pos.y+(-1), pos.z+(0)),
        WorldPosition::new(pos.x+(-1), pos.y+(-1), pos.z+(1)),
        WorldPosition::new(pos.x+(-1), pos.y+(0), pos.z+(-1)),
        WorldPosition::new(pos.x+(-1), pos.y+(0), pos.z+(0)),
        WorldPosition::new(pos.x+(-1), pos.y+(0), pos.z+(1)),
        WorldPosition::new(pos.x+(-1), pos.y+(1), pos.z+(-1)),
        WorldPosition::new(pos.x+(-1), pos.y+(1), pos.z+(0)),
        WorldPosition::new(pos.x+(-1), pos.y+(1), pos.z+(1)),
        WorldPosition::new(pos.x+(1), pos.y+(-1), pos.z+(-1)),
        WorldPosition::new(pos.x+(1), pos.y+(-1), pos.z+(0)),
        WorldPosition::new(pos.x+(1), pos.y+(-1), pos.z+(1)),
        WorldPosition::new(pos.x+(1), pos.y+(0), pos.z+(-1)),
        WorldPosition::new(pos.x+(1), pos.y+(0), pos.z+(0)),
        WorldPosition::new(pos.x+(1), pos.y+(0), pos.z+(1)),
        WorldPosition::new(pos.x+(1), pos.y+(1), pos.z+(-1)),
        WorldPosition::new(pos.x+(1), pos.y+(1), pos.z+(0)),
        WorldPosition::new(pos.x+(1), pos.y+(1), pos.z+(1)),
        WorldPosition::new(pos.x+(0), pos.y+(1), pos.z+(-1)),
        WorldPosition::new(pos.x+(0), pos.y+(1), pos.z+(0)),
        WorldPosition::new(pos.x+(0), pos.y+(1), pos.z+(1)),
        WorldPosition::new(pos.x+(0), pos.y+(-1), pos.z+(-1)),
        WorldPosition::new(pos.x+(0), pos.y+(-1), pos.z+(0)),
        WorldPosition::new(pos.x+(0), pos.y+(-1), pos.z+(1)),
        WorldPosition::new(pos.x+(0), pos.y+(0), pos.z+(1)),
        WorldPosition::new(pos.x+(0), pos.y+(0), pos.z+(-1)),
        WorldPosition::new(pos.x+(-2), pos.y+(-1), pos.z+(0)),
        WorldPosition::new(pos.x+(-2), pos.y+(0), pos.z+(-1)),
        WorldPosition::new(pos.x+(-2), pos.y+(0), pos.z+(0)),
        WorldPosition::new(pos.x+(-2), pos.y+(0), pos.z+(1)),
        WorldPosition::new(pos.x+(-2), pos.y+(1), pos.z+(0)),
        WorldPosition::new(pos.x+(2), pos.y+(-1), pos.z+(0)),
        WorldPosition::new(pos.x+(2), pos.y+(0), pos.z+(-1)),
        WorldPosition::new(pos.x+(2), pos.y+(0), pos.z+(0)),
        WorldPosition::new(pos.x+(2), pos.y+(0), pos.z+(1)),
        WorldPosition::new(pos.x+(2), pos.y+(1), pos.z+(0)),
        WorldPosition::new(pos.x+(-1), pos.y+(2), pos.z+(0)),
        WorldPosition::new(pos.x+(0), pos.y+(2), pos.z+(-1)),
        WorldPosition::new(pos.x+(0), pos.y+(2), pos.z+(0)),
        WorldPosition::new(pos.x+(0), pos.y+(2), pos.z+(1)),
        WorldPosition::new(pos.x+(1), pos.y+(2), pos.z+(0)),
        WorldPosition::new(pos.x+(-1), pos.y+(-2), pos.z+(0)),
        WorldPosition::new(pos.x+(0), pos.y+(-2), pos.z+(-1)),
        WorldPosition::new(pos.x+(0), pos.y+(-2), pos.z+(0)),
        WorldPosition::new(pos.x+(0), pos.y+(-2), pos.z+(1)),
        WorldPosition::new(pos.x+(1), pos.y+(-2), pos.z+(0)),
        WorldPosition::new(pos.x+(-1), pos.y+(0), pos.z+(2)),
        WorldPosition::new(pos.x+(0), pos.y+(-1), pos.z+(2)),
        WorldPosition::new(pos.x+(0), pos.y+(0), pos.z+(2)),
        WorldPosition::new(pos.x+(0), pos.y+(1), pos.z+(2)),
        WorldPosition::new(pos.x+(1), pos.y+(0), pos.z+(2)),
        WorldPosition::new(pos.x+(-1), pos.y+(0), pos.z+(-2)),
        WorldPosition::new(pos.x+(0), pos.y+(-1), pos.z+(-2)),
        WorldPosition::new(pos.x+(0), pos.y+(0), pos.z+(-2)),
        WorldPosition::new(pos.x+(0), pos.y+(1), pos.z+(-2)),
        WorldPosition::new(pos.x+(1), pos.y+(0), pos.z+(-2))
    ]
}
