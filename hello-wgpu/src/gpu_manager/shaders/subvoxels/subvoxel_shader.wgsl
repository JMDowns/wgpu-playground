const MAX_SUBVOXEL_OBJECTS : i32 = 500;
const MAX_SUBVOXEL_U32S : i32 = 313;
const MAX_COLORS : i32 = 32;
const MAX_AMBIENT_OCCLUSION_U32S : i32 = 782;
struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_proj_inverse: mat4x4<f32>,
    position: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position:vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) @interpolate(flat) side: i32,
    @location(2) @interpolate(flat) sv_id: u32,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) side: i32,
    @location(2) sv_id: u32,
};

struct SubvoxelObject {
    rx1: f32, rx2: f32, rx3: f32,
    ry1: f32, ry2: f32, ry3: f32,
    rz1: f32, rz2: f32, rz3: f32,
    size_x: f32,
    size_y: f32,
    size_z: f32,
    center_x: f32,
    center_y: f32,
    center_z: f32,
    model_offset: u32
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(1) @binding(0)
var<storage> SV_OBJECTS: array<SubvoxelObject, MAX_SUBVOXEL_OBJECTS>;
@group(1) @binding(1)
var<storage> SV_VOXELS: array<u32, MAX_SUBVOXEL_U32S>;
@group(1) @binding(2)
var<storage> SV_PALETTE: array<vec4<f32>, MAX_COLORS>;
@group(1) @binding(3)
var<storage> AMBIENT_OCCLUSION_ARRAY: array<u32, MAX_AMBIENT_OCCLUSION_U32S>;
const BITS_PER_SUBVOXEL_PALETTE : u32 = 8u;
const FRONT = 0;
const BACK = 1;
const LEFT = 2;
const RIGHT = 3;
const TOP = 4;
const BOTTOM = 5;

fn ao_calc(subvoxel_step: vec3<f32>, current_position_fract: vec3<f32>, block_index: u32, current_side: i32, color: vec4<f32>, ao_offset: u32) -> vec4<f32> {
    let block_index_bits_start = block_index * 20u;
    let block_index_bits_end = block_index_bits_start + 20u;
    let ambient_occlusion_voxel_start_bits_index = block_index_bits_start / 32u;
    let ambient_occlusion_voxel_end_bits_index = block_index_bits_end / 32u;
    var ao_bits = 0u;
    if (ambient_occlusion_voxel_start_bits_index != ambient_occlusion_voxel_end_bits_index) {
        let start_bits = AMBIENT_OCCLUSION_ARRAY[ambient_occlusion_voxel_start_bits_index + ao_offset] << (block_index_bits_start % 32u);
        let end_bits = AMBIENT_OCCLUSION_ARRAY[ambient_occlusion_voxel_end_bits_index + ao_offset] >> (32u - (block_index_bits_start % 32u));
        ao_bits = (start_bits | end_bits) >> 12u;
    } else {
        ao_bits = AMBIENT_OCCLUSION_ARRAY[ambient_occlusion_voxel_start_bits_index + ao_offset] >> (32u - (block_index_bits_end % 32u));
    }

    var FRONT_TOP_LEFT = f32((ao_bits & 524288u) >> 19u);
    var FRONT_TOP = f32((ao_bits & 262144u) >> 18u);
    var FRONT_TOP_RIGHT = f32((ao_bits & 131072u) >> 17u);
    var FRONT_RIGHT = f32((ao_bits & 65536u) >> 16u);
    var FRONT_BOTTOM_RIGHT = f32((ao_bits & 32768u) >> 15u);
    var FRONT_BOTTOM = f32((ao_bits & 16384u) >> 14u);
    var FRONT_BOTTOM_LEFT = f32((ao_bits & 8192u) >> 13u);
    var FRONT_LEFT = f32((ao_bits & 4096u) >> 12u);
    var LEFT_TOP = f32((ao_bits & 2048u) >> 11u);
    var LEFT_BOTTOM = f32((ao_bits & 1024u) >> 10u);
    var RIGHT_BOTTOM = f32((ao_bits & 512u) >> 9u);
    var RIGHT_TOP = f32((ao_bits & 256u) >> 8u);
    var BACK_TOP_LEFT = f32((ao_bits & 128u) >> 7u);
    var BACK_TOP = f32((ao_bits & 64u) >> 6u);
    var BACK_TOP_RIGHT = f32((ao_bits & 32u) >> 5u);
    var BACK_RIGHT = f32((ao_bits & 16u) >> 4u);
    var BACK_BOTTOM_RIGHT = f32((ao_bits & 8u) >> 3u);
    var BACK_BOTTOM = f32((ao_bits & 4u) >> 2u);
    var BACK_BOTTOM_LEFT = f32((ao_bits & 2u) >> 1u);
    var BACK_LEFT = f32((ao_bits & 1u) >> 0u);

    var SMALLER_BOUNDS = .3;
    var BOUND_MULTIPLICATION = 1. / SMALLER_BOUNDS;
    var LARGER_BOUNDS = 1. - SMALLER_BOUNDS;

    let SMALLER = max(vec3<f32>(0.), SMALLER_BOUNDS - current_position_fract) * BOUND_MULTIPLICATION;

    let LARGER = max(vec3<f32>(0.), current_position_fract - LARGER_BOUNDS) * BOUND_MULTIPLICATION;
    
    var ao_coefficient_sides = 0.;
    var ao_coefficient_corner = 0.;
    var ao_coefficient_surrounded = 0.;

    if (current_side == FRONT) {
        ao_coefficient_sides += SMALLER.y * FRONT_BOTTOM;
        ao_coefficient_sides += LARGER.y * FRONT_TOP;
        ao_coefficient_sides += SMALLER.z * FRONT_LEFT;
        ao_coefficient_sides += LARGER.z * FRONT_RIGHT;
        ao_coefficient_corner += SMALLER.y * SMALLER.z * FRONT_BOTTOM_LEFT;
        ao_coefficient_corner += SMALLER.y * LARGER.z * FRONT_BOTTOM_RIGHT;
        ao_coefficient_corner += LARGER.y * SMALLER.z * FRONT_TOP_LEFT;
        ao_coefficient_corner += LARGER.y * LARGER.z * FRONT_TOP_RIGHT;
        ao_coefficient_surrounded += SMALLER.y * SMALLER.z * FRONT_BOTTOM * FRONT_LEFT;
        ao_coefficient_surrounded += SMALLER.y * LARGER.z * FRONT_BOTTOM * FRONT_RIGHT;
        ao_coefficient_surrounded += LARGER.y * SMALLER.z * FRONT_TOP * FRONT_LEFT;
        ao_coefficient_surrounded += LARGER.y * LARGER.z * FRONT_TOP * FRONT_RIGHT;
    }
    else if (current_side == BACK) {
        ao_coefficient_sides += SMALLER.y * BACK_BOTTOM;
        ao_coefficient_sides += LARGER.y * BACK_TOP;
        ao_coefficient_sides += SMALLER.z * BACK_LEFT;
        ao_coefficient_sides += LARGER.z * BACK_RIGHT;
        ao_coefficient_corner += SMALLER.y * SMALLER.z * BACK_BOTTOM_LEFT;
        ao_coefficient_corner += SMALLER.y * LARGER.z * BACK_BOTTOM_RIGHT;
        ao_coefficient_corner += LARGER.y * SMALLER.z * BACK_TOP_LEFT;
        ao_coefficient_corner += LARGER.y * LARGER.z * BACK_TOP_RIGHT;
        ao_coefficient_surrounded += SMALLER.y * SMALLER.z * BACK_BOTTOM * BACK_LEFT;
        ao_coefficient_surrounded += SMALLER.y * LARGER.z * BACK_BOTTOM * BACK_RIGHT;
        ao_coefficient_surrounded += LARGER.y * SMALLER.z * BACK_TOP * BACK_LEFT;
        ao_coefficient_surrounded += LARGER.y * LARGER.z * BACK_TOP * BACK_RIGHT;
    }
    else if (current_side == LEFT) {
        ao_coefficient_sides += SMALLER.x * FRONT_LEFT;
        ao_coefficient_sides += LARGER.x * BACK_LEFT;
        ao_coefficient_sides += SMALLER.y * LEFT_BOTTOM;
        ao_coefficient_sides += LARGER.y * LEFT_TOP;
        ao_coefficient_corner += SMALLER.x * SMALLER.y * FRONT_BOTTOM_LEFT;
        ao_coefficient_corner += SMALLER.x * LARGER.y * FRONT_TOP_LEFT;
        ao_coefficient_corner += LARGER.x * SMALLER.y * BACK_BOTTOM_LEFT;
        ao_coefficient_corner += LARGER.x * LARGER.y * BACK_TOP_LEFT;
        ao_coefficient_surrounded += SMALLER.x * SMALLER.y * FRONT_LEFT * LEFT_BOTTOM;
        ao_coefficient_surrounded += SMALLER.x * LARGER.y * FRONT_LEFT * LEFT_TOP;
        ao_coefficient_surrounded += LARGER.x * SMALLER.y * BACK_LEFT * LEFT_BOTTOM;
        ao_coefficient_surrounded += LARGER.x * LARGER.y * BACK_LEFT * LEFT_TOP;
    }
    else if (current_side == RIGHT) {
        ao_coefficient_sides += SMALLER.x * FRONT_RIGHT;
        ao_coefficient_sides += LARGER.x * BACK_RIGHT;
        ao_coefficient_sides += SMALLER.y * RIGHT_BOTTOM;
        ao_coefficient_sides += LARGER.y * RIGHT_TOP;
        ao_coefficient_corner += SMALLER.x * SMALLER.y * FRONT_BOTTOM_RIGHT;
        ao_coefficient_corner += SMALLER.x * LARGER.y * FRONT_TOP_RIGHT;
        ao_coefficient_corner += LARGER.x * SMALLER.y * BACK_BOTTOM_RIGHT;
        ao_coefficient_corner += LARGER.x * LARGER.y * BACK_TOP_RIGHT;
        ao_coefficient_surrounded += SMALLER.x * SMALLER.y * FRONT_RIGHT * RIGHT_BOTTOM;
        ao_coefficient_surrounded += SMALLER.x * LARGER.y * FRONT_RIGHT * RIGHT_TOP;
        ao_coefficient_surrounded += LARGER.x * SMALLER.y * BACK_RIGHT * RIGHT_BOTTOM;
        ao_coefficient_surrounded += LARGER.x * LARGER.y * BACK_RIGHT * RIGHT_TOP;
    }
    else if (current_side == TOP) {
        ao_coefficient_sides += SMALLER.x * FRONT_TOP;
        ao_coefficient_sides += LARGER.x * BACK_TOP;
        ao_coefficient_sides += SMALLER.z * LEFT_TOP; 
        ao_coefficient_sides += LARGER.z * RIGHT_TOP;
        ao_coefficient_corner += SMALLER.x * LARGER.z * FRONT_TOP_RIGHT;
        ao_coefficient_corner += LARGER.x * LARGER.z * BACK_TOP_RIGHT;
        ao_coefficient_corner += SMALLER.x * SMALLER.z * FRONT_TOP_LEFT;
        ao_coefficient_corner += LARGER.x * SMALLER.z * BACK_TOP_LEFT;
        ao_coefficient_surrounded += SMALLER.x * LARGER.z * FRONT_TOP * RIGHT_TOP;
        ao_coefficient_surrounded += LARGER.x * LARGER.z * BACK_TOP * RIGHT_TOP;
        ao_coefficient_surrounded += SMALLER.x * SMALLER.z * FRONT_TOP * LEFT_TOP;
        ao_coefficient_surrounded += LARGER.x * SMALLER.z * BACK_TOP * LEFT_TOP;
    }
    else if (current_side == BOTTOM) {
        ao_coefficient_sides += SMALLER.x * FRONT_BOTTOM;
        ao_coefficient_sides += LARGER.x * BACK_BOTTOM;
        ao_coefficient_sides += SMALLER.z * LEFT_BOTTOM; 
        ao_coefficient_sides += LARGER.z * RIGHT_BOTTOM;
        ao_coefficient_corner += SMALLER.x * LARGER.z * FRONT_BOTTOM_RIGHT;
        ao_coefficient_corner += LARGER.x * LARGER.z * BACK_BOTTOM_RIGHT;
        ao_coefficient_corner += SMALLER.x * SMALLER.z * FRONT_BOTTOM_LEFT;
        ao_coefficient_corner += LARGER.x * SMALLER.z * BACK_BOTTOM_LEFT;
        ao_coefficient_surrounded += SMALLER.x * LARGER.z * FRONT_BOTTOM * RIGHT_BOTTOM;
        ao_coefficient_surrounded += LARGER.x * LARGER.z * BACK_BOTTOM * RIGHT_BOTTOM;
        ao_coefficient_surrounded += SMALLER.x * SMALLER.z * FRONT_BOTTOM * LEFT_BOTTOM;
        ao_coefficient_surrounded += LARGER.x * SMALLER.z * BACK_BOTTOM * LEFT_BOTTOM;
    }
    
    var color_coefficient = 0.;

    if (ao_coefficient_surrounded > 0.) {
        color_coefficient = ao_coefficient_sides + ao_coefficient_surrounded;
    } else if (ao_coefficient_sides < 0.01) {
        color_coefficient = ao_coefficient_corner;
    } else {
        color_coefficient = ao_coefficient_sides;
    }

    var new_color = mix(color, vec4<f32>(0.05, 0.05, 0.05, 0.0), .2 * color_coefficient);

    return new_color;
}
fn get_subvoxel_block_index(dimension: vec3<u32>, grid_coordinates: vec3<u32>) -> u32 {
    return grid_coordinates.x + grid_coordinates.y * dimension.x + grid_coordinates.z * dimension.x * dimension.y;
}
fn get_subvoxel_at_index(sv_offset_in_u32s: u32, block_index: u32) -> u32 {
    let u32_offset = sv_offset_in_u32s + (block_index * BITS_PER_SUBVOXEL_PALETTE) / 32u;
    let bit_offset = (block_index * BITS_PER_SUBVOXEL_PALETTE) % 32u;
    let subvoxel_palette_value = (SV_VOXELS[u32_offset] >> bit_offset) & 255u;
    return subvoxel_palette_value;
}
fn raycast(world_position: vec3<f32>, model_offset: u32, rotation_matrix: mat3x3<f32>, center: vec3<f32>, size: vec3<f32>, dimension: vec3<u32>) -> vec4<f32> {
    var subvoxel_step = size / vec3<f32>(dimension);
    var relative_position = (transpose(rotation_matrix)*(world_position - center));

    var camera_position_model = (transpose(rotation_matrix)*(camera.position.xyz - center));
    var direction_vector = normalize(relative_position - camera_position_model);

    var direction_vector_offset = .00001 * sign(direction_vector);
    
    var model_position = relative_position + size / 2. + direction_vector_offset;

    var model_position_subvoxel_f32 = model_position / subvoxel_step;
    var model_grid_coordinates = vec3<i32>(model_position_subvoxel_f32);
    var grid_correction = vec3<i32>(model_grid_coordinates >= vec3<i32>(dimension));
    model_grid_coordinates -= grid_correction;

    var step_directions = sign(direction_vector);
    var step_directions_i32 = vec3<i32>(step_directions);

    var current_position = model_position;

    var step_faces = vec3<i32>(0, 0, 0);
    var step_axis = vec3<i32>(0);
    if (step_directions.x < 0.) {
        step_faces.x = BACK;
    } else if (step_directions.x > 0.) {
        step_faces.x = FRONT;
    }

    if (step_directions.y < 0.) {
        step_faces.y = TOP;
    } else if (step_directions.y > 0.) {
        step_faces.y = BOTTOM;
    }

    if (step_directions.z < 0.) {
        step_faces.z = RIGHT;
    } else if (step_directions.z > 0.) {
        step_faces.z = LEFT;
    }

    let block_index = get_subvoxel_block_index(dimension, vec3<u32>(model_grid_coordinates));
    let subvoxel_palette = get_subvoxel_at_index(model_offset+4u, block_index);

    if (subvoxel_palette != 0u) {
        return SV_PALETTE[subvoxel_palette];
    }

    let max_step_size = i32(dimension.x << 1u) + i32(dimension.y << 1u) + i32(dimension.z << 1u);

    var step_sizes = subvoxel_step / abs(direction_vector);
    var next_distance = (step_directions * 0.5 + 0.5 - (model_position_subvoxel_f32-vec3<f32>(model_grid_coordinates))) / direction_vector * subvoxel_step;

    for(var i: i32 = 1; i <= max_step_size; i++) {
        var closest_distance = min(min(next_distance.x, next_distance.y), next_distance.z);
        current_position = current_position + direction_vector * closest_distance;
        step_axis = vec3<i32>(next_distance <= vec3<f32>(closest_distance));
        model_grid_coordinates = model_grid_coordinates + step_axis * step_directions_i32;
        next_distance = next_distance - closest_distance;
        next_distance = next_distance + step_sizes * vec3<f32>(step_axis);

        if (any(model_grid_coordinates < vec3<i32>(0))) {
            break;
        }

        if (any(vec3<u32>(model_grid_coordinates) >= dimension)) {
            break;
        }

        let block_index = get_subvoxel_block_index(dimension, vec3<u32>(model_grid_coordinates));
        let subvoxel_palette = get_subvoxel_at_index(model_offset+4u, block_index);
        if (subvoxel_palette != 0u) {
            let ao_offset = SV_VOXELS[model_offset+3u];
            return ao_calc(subvoxel_step, fract(current_position / subvoxel_step), block_index, dot(step_axis, step_faces), SV_PALETTE[subvoxel_palette], ao_offset);
        }
    }

    if (true) {
        discard;
    }
    return vec4<f32>(0.0, 0., 0., 0.);
}





@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position.x, model.position.y, model.position.z, 1.0);
    out.world_position = model.position;
    out.side = model.side;
    out.sv_id = model.sv_id;
    return out;
}







@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var sv_object = SV_OBJECTS[in.sv_id];
    var model_offset = sv_object.model_offset;

    var size = vec3<f32>(sv_object.size_x, sv_object.size_y, sv_object.size_z);
    var center = vec3<f32>(sv_object.center_x, sv_object.center_y, sv_object.center_z);
    var dimension = vec3<u32>(SV_VOXELS[model_offset], SV_VOXELS[model_offset+1u], SV_VOXELS[model_offset+2u]);
    var rotation_matrix = mat3x3<f32>(
        vec3<f32>(sv_object.rx1, sv_object.rx2, sv_object.rx3),
        vec3<f32>(sv_object.ry1, sv_object.ry2, sv_object.ry3),
        vec3<f32>(sv_object.rz1, sv_object.rz2, sv_object.rz3)
    );

    return raycast(in.world_position, model_offset, rotation_matrix, center, size, dimension);
}

