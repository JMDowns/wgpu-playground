//FUNCTION IMPORT REQ ao_calc START
//FILE REQ subvoxels/sides.template.wgsl

//BUFFER REQ BEGIN
//NAME AMBIENT_OCCLUSION_ARRAY
//BUFFER_TYPE STORAGE
//TYPE ARRAY_U32
//BUFFER REQ END

//FUNCTION IMPORT REQ END
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