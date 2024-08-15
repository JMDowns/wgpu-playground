struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_proj_inverse: mat4x4<f32>,
    position: vec4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) clip_position:vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) side: i32,
    @location(2) sv_id: u32,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) side: i32,
    @location(2) sv_id: u32,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position.x, model.position.y, model.position.z, 1.0);
    out.world_position = model.position;
    out.side = model.side;
    out.sv_id = model.sv_id;
    return out;
}

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

//INLINE CONST i32 MAX_SUBVOXEL_OBJECTS
const MAX_SUBVOXEL_OBJECTS = 32;
//END INLINE CONST
@group(1) @binding(0)
var<storage> SV_OBJECTS: array<SubvoxelObject, MAX_SUBVOXEL_OBJECTS>;
//INLINE CONST i32 MAX_SUBVOXEL_U32S
const MAX_SUBVOXEL_U32S = 0;
//END INLINE CONST
@group(1) @binding(1)
var<storage> SV_VOXELS: array<u32, MAX_SUBVOXEL_U32S>;
//INLINE CONST i32 MAX_COLORS
const MAX_COLORS = 0;
//END INLINE CONST
@group(1) @binding(2)
var<storage> SV_PALETTE: array<vec4<f32>, MAX_COLORS>;
//INLINE CONST i32 MAX_AMBIENT_OCCLUSION_U32S
const MAX_AMBIENT_OCCLUSION_U32S = 0;
//END INLINE CONST
@group(1) @binding(3)
var<storage> AMBIENT_OCCLUSION_ARRAY: array<u32, MAX_AMBIENT_OCCLUSION_U32S>;

//INLINE FUNCTION raycast
//FILENAME subvoxels/raycast.template.wgsl
fn raycast(world_position: vec3<f32>, model_offset: u32, rotation_matrix: mat3x3<f32>, center: vec3<f32>, size: vec3<f32>, dimension: vec3<u32>) -> vec4<f32> {
    return vec4<f32>(0.0);
}
//END INLINE FUNCTION

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