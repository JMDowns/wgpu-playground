use std::path::Path;
use std::fs::File;
use std::io::{BufWriter, Write};

use fundamentals::consts::{NUMBER_OF_CHUNKS_AROUND_PLAYER, CHUNK_DIMENSION, WORKGROUP_SIZE};

pub fn build_compute_file() {
    let compute_path = Path::new("../hello-wgpu/src/frustum_compute.wgsl");
    let mut compute_file = BufWriter::new(File::create(&compute_path).unwrap());

    writeln!(
        &mut compute_file,
         "{}",
         build_compute_string()
    ).unwrap();
}

fn build_compute_string() -> String {
    let chunk_position_array_length = NUMBER_OF_CHUNKS_AROUND_PLAYER * 3;
    format!("let CHUNKS_AROUND_PLAYER = {NUMBER_OF_CHUNKS_AROUND_PLAYER};
let CHUNK_DIMENSION = {CHUNK_DIMENSION};

struct CameraUniform {{
    view_proj: mat4x4<f32>,
}};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct BucketData {{
    buffer_index: i32,
    bucket_index: i32,
    vertex_count: u32
}};

struct ComputeData {{
    world_position: array<i32,3>,
    bucket_data: array<BucketData, 12>
}};

@group(0) @binding(1)
var<storage> computeDataArray: array<ComputeData, {chunk_position_array_length}>;

@group(0) @binding(2)
var<storage, read_write> chunk_indexes_to_show: array<u32, CHUNKS_AROUND_PLAYER>;

fn is_not_in_frustum_via_plane(center_point: vec3<f32>, plane_normal: vec3<f32>, plane_distance: f32) -> bool {{
    var r = abs(plane_normal.x * f32(CHUNK_DIMENSION / 2)) 
                        + abs(plane_normal.y * f32(CHUNK_DIMENSION / 2))
                        + abs(plane_normal.z * f32(CHUNK_DIMENSION / 2));

    var d = dot(plane_normal,center_point) + plane_distance;

    if (abs(d) < r) {{
        return false;
    }} else if (d < 0.0) {{
        return d + r < 0.0;
    }}
    return d - r < 0.0;
}}

fn is_in_frustum(index: u32) -> bool {{
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

    if (is_not_in_frustum_via_plane(center_of_chunk, left_normal, left_distance)) {{
        return false;
    }}
    if (is_not_in_frustum_via_plane(center_of_chunk, right_normal, right_distance)) {{
        return false;
    }}
    if (is_not_in_frustum_via_plane(center_of_chunk, bottom_normal, bottom_distance)) {{
        return false;
    }}
    if (is_not_in_frustum_via_plane(center_of_chunk, top_normal, top_distance)) {{
        return false;
    }}
    if (is_not_in_frustum_via_plane(center_of_chunk, front_normal, front_distance)) {{
        return false;
    }}
    if (is_not_in_frustum_via_plane(center_of_chunk, back_normal, back_distance)) {{
        return false;
    }}
    return true;
}}

@compute
@workgroup_size({WORKGROUP_SIZE})
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    var index = global_id.x;
    if (index >= u32(CHUNKS_AROUND_PLAYER)) {{
        return;
    }} 
    if (is_in_frustum(index)) {{
        chunk_indexes_to_show[index] = 1u;
    }} else {{
        chunk_indexes_to_show[index] = 0u;
    }}
}}")
}