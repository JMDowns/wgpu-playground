use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use formats::formats::config_format::ConfigFormat;
use formats::formats::controls_format::ControlsFormat;

pub fn generate_consts(config_format: &ConfigFormat, consts_model: &ConstsModel, controls_format: &ControlsFormat) {
    let consts_path = Path::new("src/consts.rs");
    let mut consts_file = BufWriter::new(File::create(&consts_path).unwrap());

    let position_offset_vec = generate_chunk_pos_around_player_fn(config_format);
    let num_chunks_around_player = position_offset_vec.len();

    let num_vertices_in_bucket = config_format.vertices_per_bucket;
    let num_buckets_per_chunk = (config_format.chunk_dimension as u32)*(config_format.chunk_dimension as u32)*(config_format.chunk_dimension as u32) * 8 / num_vertices_in_bucket;

    let mip_level = (config_format.texture_dimension as f32).log2() as u32;
    let texture_length_with_mipmaps = generate_texture_length_with_mipmap_level(mip_level, config_format.texture_dimension);

    if config_format.chunk_dimension % 2 != 0 {
        panic!("Chunk dimension must be a multiple of 2, as index buffer writes must be a multiple of 4.")
    }
    writeln!(
        &mut consts_file,
        "{}",
        [
            String::from("use crate::world_position::WorldPosition;"),
            String::from("use winit::event::VirtualKeyCode;\n"),
            format!("pub const NUM_BLOCK_TYPES: u16 = {};", consts_model.num_block_types),
            format!("pub const NUM_ADDITIONAL_THREADS: usize = {};", generate_num_threads(&config_format)),
            format!("pub const RENDER_DISTANCE: usize = {};", config_format.render_radius),
            format!("pub const FOV_DISTANCE: usize = {};", config_format.render_radius*config_format.chunk_dimension as usize),
            format!("pub const CHUNK_DIMENSION: i32 = {};", config_format.chunk_dimension),
            format!("pub const CHUNK_PLANE_SIZE: i32 = {};", (config_format.chunk_dimension as u32)*(config_format.chunk_dimension as u32)),
            format!("pub const CHUNK_SIZE: usize = {};", (config_format.chunk_dimension as u32)*(config_format.chunk_dimension as u32)*(config_format.chunk_dimension as u32)),
            format!("pub const CHUNK_DIMENSION_WRAPPED: usize = {};", config_format.chunk_dimension+2),
            format!("pub const CHUNK_PLANE_SIZE_WRAPPED: usize = {};", (config_format.chunk_dimension as u32+2)*(config_format.chunk_dimension as u32+2)),
            format!("pub const CHUNK_SIZE_WRAPPED: usize = {};", (config_format.chunk_dimension as u32+2)*(config_format.chunk_dimension as u32+2)*(config_format.chunk_dimension as u32+2)),
            format!("pub const BITS_PER_POSITION: u32 = {};", ((config_format.chunk_dimension+1) as f32).log2().ceil() as u8),
            format!("pub const TEXTURE_DIMENSION: u32 = {};", config_format.texture_dimension),
            format!("pub const NUM_TEXTURES: usize = {};", consts_model.num_textures),
            format!("pub const TEX_MAX_X: u32 = {};", consts_model.atlas_max_num_images_width),
            format!("pub const TEX_MAX_Y: u32 = {};", consts_model.atlas_max_num_images_height),
            format!("pub const BITS_PER_TEX_COORD_X: u32 = {};", (((consts_model.atlas_max_num_images_width + 1) as f32).log2().ceil())),
            format!("pub const BITS_PER_TEX_COORD_Y: u32 = {};", (((consts_model.atlas_max_num_images_height + 1) as f32).log2().ceil())),
            format!("pub const BITS_PER_AMBIENT_OCCLUSION: u32 = 2;"),
            format!("pub const NUMBER_OF_CHUNKS_AROUND_PLAYER: u32 = {};", num_chunks_around_player),
            format!("pub const NUMBER_OF_CHUNKS_TO_RENDER: u32 = {};", (num_chunks_around_player as f32 / 8 as f32).ceil() as u32),
            format!("pub const BITS_PER_CHUNK_INDEX: u32 = {};", ((num_chunks_around_player) as f32).log2().ceil() as u32),
            format!("pub const WORKGROUP_SIZE: u16 = {};", 255),
            format!("pub const NUM_VERTICES_IN_BUCKET: u32 = {};", num_vertices_in_bucket), 
            format!("pub const NUM_BUCKETS_PER_CHUNK: usize = {};", num_buckets_per_chunk),
            format!("pub const NUM_BUCKETS_PER_SIDE: u32 = {};", num_buckets_per_chunk / 6),
            format!("pub const NUM_BUCKETS: usize = {};", num_buckets_per_chunk * num_chunks_around_player as u32),
            format!("pub const MESH_METHOD: &str = \"{}\";", config_format.mesh_method),
            format!("pub const CHUNK_GENERATION_METHOD: &str = \"{}\";", config_format.chunk_generation_method),
            format!("pub const PERLIN_POSITIVE_THRESHOLD: f64 = {};", config_format.chunk_generation_options.perlin_positive_threshold),
            format!("pub const PERLIN_NEGATIVE_THRESHOLD: f64 = {};", config_format.chunk_generation_options.perlin_negative_threshold),
            format!("pub const PERLIN_SCALE_FACTOR: f64 = {};", config_format.chunk_generation_options.perlin_scale_factor),
            String::new(),
            format!("pub const MIP_LEVEL: u32 = {};", mip_level),
            format!("pub const TEXTURE_LENGTH_WITH_MIPMAPS: usize = {};", texture_length_with_mipmaps),
            String::new(),
            format!("pub const UP_KEY: VirtualKeyCode = {};", controls_format.up),
            format!("pub const DOWN_KEY: VirtualKeyCode = {};", controls_format.down),
            format!("pub const LEFT_KEY: VirtualKeyCode = {};", controls_format.left),
            format!("pub const RIGHT_KEY: VirtualKeyCode = {};", controls_format.right),
            format!("pub const FORWARD_KEY: VirtualKeyCode = {};", controls_format.forward),
            format!("pub const BACKWARD_KEY: VirtualKeyCode = {};", controls_format.backward),
            format!("pub const MOUSE_SENSITIVITY_THRESHOLD: f64 = {:.1};", controls_format.mouse_sensitivity_threshold),
            format!("pub const MOUSE_SENSITIVITY: f32 = {:.1};", controls_format.mouse_sensitivity),
            String::new(),
            format!("pub const NUM_TASKS_PER_THREAD: usize = {};", config_format.num_tasks_per_thread),
            format!("pub const GENERATE_CHUNK_PRIORITY: u32 = {};", config_format.task_priorities.chunk),
            format!("pub const UPDATE_CHUNK_PADDING_X_PRIORITY: u32 = {};", config_format.task_priorities.update_chunk_padding_x),
            format!("pub const UPDATE_CHUNK_PADDING_Y_PRIORITY: u32 = {};", config_format.task_priorities.update_chunk_padding_y),
            format!("pub const UPDATE_CHUNK_PADDING_Z_PRIORITY: u32 = {};", config_format.task_priorities.update_chunk_padding_z),
            format!("pub const GENERATE_MESH_PRIORITY: u32 = {};", config_format.task_priorities.mesh),
            format!("pub const GENERATE_MESH_SIDE_PRIORITY: u32 = {};", config_format.task_priorities.mesh_side),
            String::new(),
            format!("pub const MOVEMENT_SPEED: f32 = {:.1};", config_format.movement_speed),
            format!("pub const MIN_MEMORY_USAGE_MB: u32 = {};", config_format.min_memory_mb),
            format!("pub const MAX_MEMORY_USAGE_MB: u32 = {};", config_format.max_memory_mb),
            generate_string_from_position_offsets(position_offset_vec),
        ].join("\n")
    ).unwrap();
}

fn generate_texture_length_with_mipmap_level(mip_level: u32, texture_dimension: u32) -> usize {
    let mut length_sum = 0;
    for mip in 0..mip_level+1 {
        length_sum += texture_dimension * texture_dimension / 4_u32.pow(mip as u32);
    }

    length_sum as usize
}

fn generate_chunk_pos_around_player_fn(config_format: &ConfigFormat) -> Vec<(i32, i32, i32)> {
    let mut vec_of_position_offsets = Vec::new();
    let radius = config_format.render_radius as i32;
    for n in 0..radius + 1 {
        let positions_vec = generate_cube_shell_size_n_positions(0, 0, 0, n);
        for (x,y,z) in positions_vec {
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
                if (((x as f32 - 0.5).powf(2.0)+(y as f32 - 0.5).powf(2.0)+(z as f32 - 0.5).powf(2.0)) as f32).sqrt() > radius as f32 + 1.0 {
                    is_in_radius = false;
                }
            }

            if is_in_radius {
                vec_of_position_offsets.push((x,y,z));
            }
        }
    }

    vec_of_position_offsets
}

fn generate_cube_shell_size_n_positions(x: i32, y: i32, z: i32, n: i32) -> Vec<(i32, i32, i32)> {
    let mut positions_vec = Vec::new();
    if n == 0 {
        positions_vec.push((x,y,z));
    } else {
        for i in -n..n+1 {
            for j in -n..n+1 {
                positions_vec.push((x-n, y+i, z+j))
            }
        }
        for i in -n..n+1 {
            for j in -n..n+1 {
                positions_vec.push((x+n, y+i, z+j))
            }
        }
        for i in -n+1..n {
            for j in -n..n+1 {
                positions_vec.push((x+i, y+n, z+j))
            }
        }
        for i in -n+1..n {
            for j in -n..n+1 {
                positions_vec.push((x+i, y-n, z+j))
            }
        }
        for i in -n+1..n {
            for j in -n+1..n {
                positions_vec.push((x+i, y+j, z+n))
            }
        }
        for i in -n+1..n {
            for j in -n+1..n {
                positions_vec.push((x+i, y+j, z-n))
            }
        }
    }
    positions_vec
}

fn generate_string_from_position_offsets(vec_of_position_offsets: Vec<(i32,i32,i32)>) -> String {
    let mut pos_new_vec = Vec::new();
    for (x_off,y_off,z_off) in vec_of_position_offsets {
        pos_new_vec.push(format!("WorldPosition::new(pos.x+({x_off}), pos.y+({y_off}), pos.z+({z_off}))"));
    }

    [
    format!("pub fn get_positions_around_player(pos: WorldPosition) -> Vec<WorldPosition> {{").as_str(),
    "    vec![",
    format!("        {}",pos_new_vec.join(",\n        ")).as_str(),
    "    ]",
    "}"
    ].join("\n")
}

fn generate_num_threads(cf: &ConfigFormat) -> usize {
    if cf.use_all_system_threads {
        num_cpus::get() - 1
    } else {
        cf.num_additional_threads_specified
    }
}

pub struct ConstsModel {
    pub num_block_types: u16,
    pub atlas_max_num_images_width: u32,
    pub atlas_max_num_images_height: u32,
    pub num_textures: usize,
}