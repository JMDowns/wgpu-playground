const CHUNKS_AROUND_PLAYER = 611;
const CHUNK_DIMENSION = 32;
const NUM_BUCKETS_PER_CHUNK = 64;
const NUM_BUCKETS_PER_SIDE = 10;
const SQRT_2_DIV_2 = .7071;
const NEG_SQRT_2_DIV_2 = -.7071;

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct DrawIndexedIndirect {
    vertex_count: u32,
    instance_count: u32,
    base_index: u32,
    vertex_offset: u32,
    base_instance: u32,
}

struct BucketData {
    buffer_index: i32,
    bucket_index: i32,
    side: u32,
};

struct ComputeData {
    world_position: array<i32,3>,
    bucket_data: array<BucketData, NUM_BUCKETS_PER_CHUNK>
};

@group(0) @binding(1)
var<storage> computeDataArray: array<ComputeData, CHUNKS_AROUND_PLAYER>;

@group(0) @binding(2)
var<storage> visibility_array: array<u32, CHUNKS_AROUND_PLAYER>;

@group(1) @binding(0)
var<storage, read_write> indirect_buffer_0: array<DrawIndexedIndirect, 32768>;
@group(1) @binding(1)
var<storage, read_write> indirect_buffer_1: array<DrawIndexedIndirect, 32768>;
@group(1) @binding(2)
var<storage, read_write> indirect_buffer_2: array<DrawIndexedIndirect, 32768>;
@group(1) @binding(3)
var<storage, read_write> indirect_buffer_3: array<DrawIndexedIndirect, 32768>;
@group(1) @binding(4)
var<storage, read_write> indirect_buffer_4: array<DrawIndexedIndirect, 32768>;
@group(1) @binding(5)
var<storage, read_write> indirect_buffer_5: array<DrawIndexedIndirect, 32768>;
@group(1) @binding(6)
var<storage, read_write> indirect_buffer_6: array<DrawIndexedIndirect, 32768>;
@group(1) @binding(7)
var<storage, read_write> indirect_buffer_7: array<DrawIndexedIndirect, 32768>;
@group(2) @binding(0)
var<storage, read_write> indirect_buffer_8: array<DrawIndexedIndirect, 32768>;
@group(2) @binding(1)
var<storage, read_write> indirect_buffer_9: array<DrawIndexedIndirect, 32768>;
@group(2) @binding(2)
var<storage, read_write> indirect_buffer_10: array<DrawIndexedIndirect, 32768>;
@group(2) @binding(3)
var<storage, read_write> indirect_buffer_11: array<DrawIndexedIndirect, 32768>;
@group(2) @binding(4)
var<storage, read_write> indirect_buffer_12: array<DrawIndexedIndirect, 32768>;
@group(2) @binding(5)
var<storage, read_write> indirect_buffer_13: array<DrawIndexedIndirect, 32768>;
@group(2) @binding(6)
var<storage, read_write> indirect_buffer_14: array<DrawIndexedIndirect, 32768>;
@group(2) @binding(7)
var<storage, read_write> indirect_buffer_15: array<DrawIndexedIndirect, 32768>;
@group(3) @binding(0)
var<storage, read_write> indirect_buffer_16: array<DrawIndexedIndirect, 32768>;
@group(3) @binding(1)
var<storage, read_write> indirect_buffer_17: array<DrawIndexedIndirect, 32768>;
@group(3) @binding(2)
var<storage, read_write> indirect_buffer_18: array<DrawIndexedIndirect, 32768>;
@group(3) @binding(3)
var<storage, read_write> indirect_buffer_19: array<DrawIndexedIndirect, 32768>;

fn is_not_in_frustum_via_plane(center_point: vec3<f32>, plane_normal: vec3<f32>, plane_distance: f32) -> bool {
    var r = abs(plane_normal.x * f32(CHUNK_DIMENSION / 2)) 
                        + abs(plane_normal.y * f32(CHUNK_DIMENSION / 2))
                        + abs(plane_normal.z * f32(CHUNK_DIMENSION / 2));

    var d = dot(plane_normal,center_point) + plane_distance;

    if (abs(d) < r) {
        return false;
    } else if (d < 0.0) {
        return d + r < 0.0;
    }
    return d - r < 0.0;
}

struct InFrustumResult {
    in_frustum: bool,
    normal_x: f32,
    normal_y: f32,
    normal_z: f32,
}

fn is_in_frustum(index: u32) -> InFrustumResult {
    var col1 = vec3<f32>(camera.view_proj[0][0], camera.view_proj[1][0], camera.view_proj[2][0]);
    var col2 = vec3<f32>(camera.view_proj[0][1], camera.view_proj[1][1], camera.view_proj[2][1]);
    var col3 = vec3<f32>(camera.view_proj[0][2], camera.view_proj[1][2], camera.view_proj[2][2]);
    var col4 = vec3<f32>(camera.view_proj[0][3], camera.view_proj[1][3], camera.view_proj[2][3]);

    var left_normal = col4 + col1;
    var right_normal = col4 - col1;
    var bottom_normal = col4 + col2;
    var top_normal = col4-col2;
    var front_normal = col3;
    var back_normal = col4-col3;

    var left_distance = camera.view_proj[3][3] + camera.view_proj[3][0];
    var right_distance = camera.view_proj[3][3] - camera.view_proj[3][0];
    var bottom_distance = camera.view_proj[3][3] + camera.view_proj[3][1];
    var top_distance = camera.view_proj[3][3] - camera.view_proj[3][1];
    var front_distance = camera.view_proj[3][2];
    var back_distance = camera.view_proj[3][3] - camera.view_proj[3][2];

    left_distance = left_distance / length(left_normal);
    left_normal = normalize(left_normal);
    right_distance = right_distance / length(right_normal);
    right_normal = normalize(right_normal);
    bottom_distance = bottom_distance / length(bottom_normal);
    bottom_normal = normalize(bottom_normal);
    top_distance = top_distance / length(top_normal);
    top_normal = normalize(top_normal);
    front_distance = front_distance / length(front_normal);
    front_normal = normalize(front_normal);
    back_distance = back_distance / length(back_normal);
    back_normal = normalize(back_normal);

    var chunk_pos = vec3<i32>(computeDataArray[index].world_position[0], computeDataArray[index].world_position[1], computeDataArray[index].world_position[2]);
    var center_of_chunk = vec3<f32>(f32(chunk_pos.x) * f32(CHUNK_DIMENSION) + f32(CHUNK_DIMENSION / 2), f32(chunk_pos.y)  * f32(CHUNK_DIMENSION) + f32(CHUNK_DIMENSION / 2), f32(chunk_pos.z)  * f32(CHUNK_DIMENSION) + f32(CHUNK_DIMENSION / 2));

    var frustum_result: InFrustumResult;
    frustum_result.normal_x = front_normal.x;
    frustum_result.normal_y = front_normal.y;
    frustum_result.normal_z = front_normal.z;

    if (is_not_in_frustum_via_plane(center_of_chunk, left_normal, left_distance)) {
        frustum_result.in_frustum = false;
        return frustum_result;
    }
    if (is_not_in_frustum_via_plane(center_of_chunk, right_normal, right_distance)) {
        frustum_result.in_frustum = false;
        return frustum_result;
    }
    if (is_not_in_frustum_via_plane(center_of_chunk, bottom_normal, bottom_distance)) {
        frustum_result.in_frustum = false;
        return frustum_result;
    }
    if (is_not_in_frustum_via_plane(center_of_chunk, top_normal, top_distance)) {
        frustum_result.in_frustum = false;
        return frustum_result;
    }
    if (is_not_in_frustum_via_plane(center_of_chunk, front_normal, front_distance)) {
        frustum_result.in_frustum = false;
        return frustum_result;
    }
    if (is_not_in_frustum_via_plane(center_of_chunk, back_normal, back_distance)) {
        frustum_result.in_frustum = false;
        return frustum_result;
    }
    frustum_result.in_frustum = true;
    return frustum_result;
}

fn set_instance_count_in_bucket(buffer_number: i32, bucket_number: i32, instance_count: u32) {
    switch buffer_number {
        case 0: {
    indirect_buffer_0[bucket_number].instance_count = instance_count;
}
case 1: {
    indirect_buffer_1[bucket_number].instance_count = instance_count;
}
case 2: {
    indirect_buffer_2[bucket_number].instance_count = instance_count;
}
case 3: {
    indirect_buffer_3[bucket_number].instance_count = instance_count;
}
case 4: {
    indirect_buffer_4[bucket_number].instance_count = instance_count;
}
case 5: {
    indirect_buffer_5[bucket_number].instance_count = instance_count;
}
case 6: {
    indirect_buffer_6[bucket_number].instance_count = instance_count;
}
case 7: {
    indirect_buffer_7[bucket_number].instance_count = instance_count;
}
case 8: {
    indirect_buffer_8[bucket_number].instance_count = instance_count;
}
case 9: {
    indirect_buffer_9[bucket_number].instance_count = instance_count;
}
case 10: {
    indirect_buffer_10[bucket_number].instance_count = instance_count;
}
case 11: {
    indirect_buffer_11[bucket_number].instance_count = instance_count;
}
case 12: {
    indirect_buffer_12[bucket_number].instance_count = instance_count;
}
case 13: {
    indirect_buffer_13[bucket_number].instance_count = instance_count;
}
case 14: {
    indirect_buffer_14[bucket_number].instance_count = instance_count;
}
case 15: {
    indirect_buffer_15[bucket_number].instance_count = instance_count;
}
case 16: {
    indirect_buffer_16[bucket_number].instance_count = instance_count;
}
case 17: {
    indirect_buffer_17[bucket_number].instance_count = instance_count;
}
case 18: {
    indirect_buffer_18[bucket_number].instance_count = instance_count;
}
case 19: {
    indirect_buffer_19[bucket_number].instance_count = instance_count;
}
default: {{}}
    }
}

fn is_chunk_mesh_empty(index: u32) -> bool {
    for (var side: i32 = 0; side < 6; side++) {
        for (var i: i32 = 0; i < NUM_BUCKETS_PER_SIDE; i++) {
            var frustum_bucket_data = computeDataArray[index].bucket_data[side * NUM_BUCKETS_PER_SIDE + i];
            if (frustum_bucket_data.buffer_index != -1) {
                return false;
            }
        }
    }
    return true;
}

@compute
@workgroup_size(255)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var index = global_id.x;
    if (index >= u32(CHUNKS_AROUND_PLAYER)) {
        return;
    } 
    if (visibility_array[index] == 0u) {
        for (var i: i32 = 0; i < NUM_BUCKETS_PER_CHUNK; i++) {
            var frustum_bucket_data = computeDataArray[index].bucket_data[i];
            set_instance_count_in_bucket(frustum_bucket_data.buffer_index, frustum_bucket_data.bucket_index, 0u);
        }
        return;
    }
    if (is_chunk_mesh_empty(index)) {
        return;
    }
    var frustum_result = is_in_frustum(index);
    if (frustum_result.in_frustum) {

        // //Front, Back, Left, Right, Top, Bottom
        var lhs_comp_arr: array<f32, 6>;
        lhs_comp_arr[0] = frustum_result.normal_x;
        lhs_comp_arr[1] = frustum_result.normal_x; 
        lhs_comp_arr[2] = frustum_result.normal_z;
        lhs_comp_arr[3] = frustum_result.normal_z;
        lhs_comp_arr[4] = frustum_result.normal_y;
        lhs_comp_arr[5] = frustum_result.normal_y;
        var is_comp_lt: array<bool, 6>;
        is_comp_lt[0] = false; 
        is_comp_lt[1] = true; 
        is_comp_lt[2] = false; 
        is_comp_lt[3] = true; 
        is_comp_lt[4] = true; 
        is_comp_lt[5] = false;
        var angles: array<f32, 6>;
        angles[0] = NEG_SQRT_2_DIV_2; 
        angles[1] = SQRT_2_DIV_2; 
        angles[2] = NEG_SQRT_2_DIV_2; 
        angles[3] = SQRT_2_DIV_2; 
        angles[4] = SQRT_2_DIV_2; 
        angles[5] = NEG_SQRT_2_DIV_2;
        for (var side: i32 = 0; side < 6; side++) {
            if (is_comp_lt[side]) {
                if (lhs_comp_arr[side] < angles[side]) {
                    for (var i: i32 = 0; i < NUM_BUCKETS_PER_SIDE; i++) {
                        var frustum_bucket_data = computeDataArray[index].bucket_data[side * NUM_BUCKETS_PER_SIDE + i];
                        if (frustum_bucket_data.buffer_index != -1) {
                            set_instance_count_in_bucket(frustum_bucket_data.buffer_index, frustum_bucket_data.bucket_index, 1u);
                        }
                        else {
                            set_instance_count_in_bucket(frustum_bucket_data.buffer_index, frustum_bucket_data.bucket_index, 0u);
                        }
                    }
                } else {
                    for (var i: i32 = 0; i < NUM_BUCKETS_PER_SIDE; i++) {
                        var frustum_bucket_data = computeDataArray[index].bucket_data[side * NUM_BUCKETS_PER_SIDE + i];
                        if (frustum_bucket_data.buffer_index != -1) {
                            set_instance_count_in_bucket(frustum_bucket_data.buffer_index, frustum_bucket_data.bucket_index, 0u);
                        }
                    }
                }
            } else {
                if (lhs_comp_arr[side] > angles[side]) {
                    for (var i: i32 = 0; i < NUM_BUCKETS_PER_SIDE; i++) {
                        var frustum_bucket_data = computeDataArray[index].bucket_data[side * NUM_BUCKETS_PER_SIDE + i];
                        if (frustum_bucket_data.buffer_index != -1) {
                            set_instance_count_in_bucket(frustum_bucket_data.buffer_index, frustum_bucket_data.bucket_index, 1u);
                        }
                        else {
                            set_instance_count_in_bucket(frustum_bucket_data.buffer_index, frustum_bucket_data.bucket_index, 0u);
                        }
                    }
                } else {
                    for (var i: i32 = 0; i < NUM_BUCKETS_PER_SIDE; i++) {
                        var frustum_bucket_data = computeDataArray[index].bucket_data[side * NUM_BUCKETS_PER_SIDE + i];
                        set_instance_count_in_bucket(frustum_bucket_data.buffer_index, frustum_bucket_data.bucket_index, 0u);
                    }
                }
            }
            
        }
    } else {
        for (var i: i32 = 0; i < NUM_BUCKETS_PER_CHUNK; i++) {
            var frustum_bucket_data = computeDataArray[index].bucket_data[i];
            set_instance_count_in_bucket(frustum_bucket_data.buffer_index, frustum_bucket_data.bucket_index, 0u);
        }
    }
}
