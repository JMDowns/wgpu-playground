fn get_subvoxel_block_index(dimension: vec3<u32>, grid_coordinates: vec3<u32>) -> u32 {
    return grid_coordinates.x + grid_coordinates.y * dimension.x + grid_coordinates.z * dimension.x * dimension.y;
}

//INLINE CONST u32 BITS_PER_SUBVOXEL_PALETTE
const BITS_PER_SUBVOXEL_PALETTE = 8u;
//END INLINE CONST
//FUNCTION IMPORT REQ get_subvoxel_at_index START

//LOCAL_CONST BITS_PER_SUBVOXEL_PALETTE

//BUFFER REQ BEGIN
//NAME SV_VOXELS
//BUFFER_TYPE STORAGE
//TYPE ARRAY_U32
//BUFFER REQ END

//FUNCTION IMPORT REQ END
fn get_subvoxel_at_index(sv_offset_in_u32s: u32, block_index: u32) -> u32 {
    let u32_offset = sv_offset_in_u32s + (block_index * BITS_PER_SUBVOXEL_PALETTE) / 32u;
    let bit_offset = (block_index * BITS_PER_SUBVOXEL_PALETTE) % 32u;
    let subvoxel_palette_value = (SV_VOXELS[u32_offset] >> bit_offset) & 255u;
    return subvoxel_palette_value;
}

//FUNCTION IMPORT REQ raycast START

//LOCAL_FUNCTION get_subvoxel_at_index
//LOCAL_FUNCTION get_subvoxel_block_index

//CAMERA REQ
//FILE REQ subvoxels/sides.template.wgsl

//GLOBAL FUNCTION ao_calc START
//FILENAME subvoxels/ao_calc.wgsl
fn ao_calc(subvoxel_step: vec3<f32>, current_position_fract: vec3<f32>, block_index: u32, current_side: i32, color: vec4<f32>, ao_offset: u32) -> vec4<f32> {
    return vec4<f32>(0.0);
}
//GLOBAL FUNCTION END

//BUFFER REQ BEGIN
//NAME SV_PALETTE
//BUFFER_TYPE STORAGE
//TYPE ARRAY_VEC4_F32
//BUFFER REQ END

//BUFFER REQ BEGIN
//NAME SV_VOXELS
//BUFFER_TYPE STORAGE
//TYPE ARRAY_U32
//BUFFER REQ END

//FUNCTION IMPORT REQ END
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

    discard;
    //return vec4<f32>(0.0, 0., 0., 0.);
}