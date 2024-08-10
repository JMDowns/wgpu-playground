struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_proj_inverse: mat4x4<f32>,
    position: vec4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct GridAlignedSubvoxelGpuData {
    size_x: u32,
    size_y: u32,
    size_z: u32,
    maximal_chunk_index: u32,
    maximal_block_x: u32,
    maximal_block_y: u32,
    maximal_block_z: u32,
    maximal_subvoxel_x: u32,
    maximal_subvoxel_y: u32,
    maximal_subvoxel_z: u32,
    model_offset: u32,
    rotation: u32
}

//region BUFFER DECLARATIONS
//INLINE CONST i32 MAX_SUBVOXEL_U32S
let MAX_SUBVOXEL_U32S = 0;
//END INLINE CONST
@group(1) @binding(0)
var<storage> SV_VOXELS: array<u32, MAX_SUBVOXEL_U32S>;

//INLINE CONST i32 MAX_AMBIENT_OCCLUSION_U32S
let MAX_AMBIENT_OCCLUSION_U32S = 0;
//END INLINE CONST
@group(1) @binding(1)
var<storage> AMBIENT_OCCLUSION_ARRAY: array<u32, MAX_AMBIENT_OCCLUSION_U32S>;

//INLINE CONST i32 NUM_GRID_ALIGNED_SUBVOXEL_OBJECTS
let NUM_GRID_ALIGNED_SUBVOXEL_OBJECTS = 0;
//END INLINE CONST
@group(1) @binding(2)
var<storage> GAS_ARRAY: array<GridAlignedSubvoxelGpuData, NUM_GRID_ALIGNED_SUBVOXEL_OBJECTS>;

//INLINE CONST i32 MAX_COLORS
let MAX_COLORS = 0;
//END INLINE CONST
@group(1) @binding(3)
var<storage> SV_PALETTE: array<vec4<f32>, MAX_COLORS>;

//INLINE CONST i32 NUM_CHUNK_I32S
let NUM_CHUNK_I32S = 21;
//END INLINE CONST
@group(2) @binding(0)
var<storage> CHUNK_POSITIONS: array<i32,NUM_CHUNK_I32S>;

//endregion

//region CONST_DECLARATIONS

//INLINE CONST u32 CHUNK_DIMENSION
let CHUNK_DIMENSION = 32u;
//END INLINE CONST
//INLINE CONST u32 SUBVOXEL_DIMENSION
let SUBVOXEL_DIMENSION = 16u;
//END INLINE CONST

//endregion

fn get_initial_world_position(grid_aligned_subvoxel_object: GridAlignedSubvoxelGpuData) -> vec3<f32> {
    let chunk_index = grid_aligned_subvoxel_object.maximal_chunk_index;
    
    var chunk_position = vec3<i32>(CHUNK_POSITIONS[3u*chunk_index], CHUNK_POSITIONS[3u*chunk_index+1u], CHUNK_POSITIONS[3u*chunk_index+2u]);
    let chunk_offset = vec3<f32>(chunk_position) * f32(CHUNK_DIMENSION);
    
    let block_offset = vec3<f32>(f32(grid_aligned_subvoxel_object.maximal_block_x), f32(grid_aligned_subvoxel_object.maximal_block_y), f32(grid_aligned_subvoxel_object.maximal_block_z));

    let subvoxel_offset_multiplier = (1. / f32(SUBVOXEL_DIMENSION));
    let subvoxel_offset = subvoxel_offset_multiplier * vec3<f32>(f32(grid_aligned_subvoxel_object.maximal_subvoxel_x), f32(grid_aligned_subvoxel_object.maximal_subvoxel_y), f32(grid_aligned_subvoxel_object.maximal_subvoxel_z));
    
    var world_position = chunk_offset + block_offset + subvoxel_offset;
    return world_position;
}

fn get_vertex_world_position(grid_aligned_subvoxel_object: GridAlignedSubvoxelGpuData, vertex_corner: u32) -> vec3<f32> {
    var world_position = get_initial_world_position(grid_aligned_subvoxel_object);
    switch vertex_corner {
        case 0u: {
            world_position -= vec3<f32>(0.0, 0.0, 0.0);
        }
        case 1u: {
            world_position -= vec3<f32>(f32(grid_aligned_subvoxel_object.size_x), 0.0, 0.0);
        }
        case 2u: {
            world_position -= vec3<f32>(f32(grid_aligned_subvoxel_object.size_x), f32(grid_aligned_subvoxel_object.size_y), 0.0);
        }
        case 3u: {
            world_position -= vec3<f32>(0., f32(grid_aligned_subvoxel_object.size_y), 0.0);
        }
        case 4u: {
            world_position -= vec3<f32>(0., 0., f32(grid_aligned_subvoxel_object.size_z));
        }
        case 5u: {
            world_position -= vec3<f32>(f32(grid_aligned_subvoxel_object.size_x), 0., f32(grid_aligned_subvoxel_object.size_z));
        }
        case 6u: {
            world_position -= vec3<f32>(f32(grid_aligned_subvoxel_object.size_x), f32(grid_aligned_subvoxel_object.size_y), f32(grid_aligned_subvoxel_object.size_z));
        }
        case 7u: {
            world_position -= vec3<f32>(0.0, f32(grid_aligned_subvoxel_object.size_y), f32(grid_aligned_subvoxel_object.size_z));
        }
        default: {
            world_position = vec3<f32>(0.0, 0.0, 0.0);
        }
    }

    return world_position;
}

//INLINE VERTEX STRUCT VertexInput GRID_ALIGNED_SUBVOXEL_VERTEX
struct VertexInput {
};
//END INLINE VERTEX STRUCT

struct VertexOutput {
    @builtin(position) clip_position:vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) gas_id: u32,
    @location(2) rotation: u32
};

@vertex
fn vs_main( model: VertexInput ) -> VertexOutput {
    // INLINE MODEL EXTRACT GRID_ALIGNED_SUBVOXEL_VERTEX
    // u32 gas_id GAS_ID
    // u32 vertex_corner VERTEX_ORIENTATION
    // END INLINE MODEL EXTRACT

    var grid_aligned_subvoxel_object = GAS_ARRAY[gas_id];
    
    var out: VertexOutput;
    out.gas_id = gas_id;
    out.world_position = get_vertex_world_position(grid_aligned_subvoxel_object, vertex_corner);
    out.clip_position = camera.view_proj * vec4<f32>(out.world_position.x, out.world_position.y, out.world_position.z, 1.0);
    out.rotation = grid_aligned_subvoxel_object.rotation;
    return out;
}

//INLINE FUNCTION raycast
//FILENAME subvoxels/raycast.template.wgsl
fn raycast(world_position: vec3<f32>, model_offset: u32, rotation_matrix: mat3x3<f32>, center: vec3<f32>, size: vec3<f32>, dimension: vec3<u32>) -> vec4<f32> {
    return vec4<f32>(0.0);
}
//END INLINE FUNCTION

fn get_rotation_matrix(rotation: u32) -> mat3x3<f32> {
    var rotation_matrix = mat3x3<f32>(
            vec3<f32>(1., 0., 0.),
            vec3<f32>(0., 1., 0.),
            vec3<f32>(0., 0., 1.)
        );

    switch rotation {
        //FRONT
        case 0u: {
            rotation_matrix = mat3x3<f32>(
                vec3<f32>(1., 0., 0.),
                vec3<f32>(0., 1., 0.),
                vec3<f32>(0., 0., 1.)
            );
        }
        //BACK
        case 1u: {
            rotation_matrix = mat3x3<f32>(
                vec3<f32>(-1., 0., 0.),
                vec3<f32>(0., 1., 0.),
                vec3<f32>(0., 0., -1.)
            );
        }
        //LEFT
        case 2u: {
            rotation_matrix = mat3x3<f32>(
                vec3<f32>(0., 0., 1.),
                vec3<f32>(0., 1., 0.),
                vec3<f32>(-1., 0., 0.)
            );
        }
        //RIGHT
        case 3u: {
            rotation_matrix = mat3x3<f32>(
                vec3<f32>(0., 0., -1.),
                vec3<f32>(0., 1., 0.),
                vec3<f32>(1., 0., 0.)
            );
        }
        //TOP
        case 4u: {
            rotation_matrix = mat3x3<f32>(
                vec3<f32>(0., -1., 0.),
                vec3<f32>(1., 0., 0.),
                vec3<f32>(0., 0., 1.)
            );
        }
        //BOTTOM
        case 5u: {
            rotation_matrix = mat3x3<f32>(
                vec3<f32>(0., 1., 0.),
                vec3<f32>(-1., 0., 0.),
                vec3<f32>(0., 0., 1.)
            );
        }
        default: {
            rotation_matrix = mat3x3<f32>(
                vec3<f32>(1., 0., 0.),
                vec3<f32>(0., 1., 0.),
                vec3<f32>(0., 0., 1.)
            );
        }
    }
    
    return rotation_matrix;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var gas_object = GAS_ARRAY[in.gas_id];
    var model_offset = gas_object.model_offset;

    var size = vec3<f32>(f32(gas_object.size_x), f32(gas_object.size_y), f32(gas_object.size_z));
    let chunk_index = 3u*gas_object.maximal_chunk_index;
    var chunk_position = vec3<i32>(CHUNK_POSITIONS[chunk_index], CHUNK_POSITIONS[chunk_index+1u], CHUNK_POSITIONS[chunk_index+2u]);
    var maximal_point = vec3<f32>(chunk_position) * f32(CHUNK_DIMENSION) 
        + vec3<f32>(f32(gas_object.maximal_block_x), f32(gas_object.maximal_block_y), f32(gas_object.maximal_block_z))
        + (1. / f32(SUBVOXEL_DIMENSION)) * vec3<f32>(f32(gas_object.maximal_subvoxel_x), f32(gas_object.maximal_subvoxel_y), f32(gas_object.maximal_subvoxel_z));
    var center = maximal_point - size / 2.;
    var dimension = vec3<u32>(SV_VOXELS[model_offset], SV_VOXELS[model_offset+1u], SV_VOXELS[model_offset+2u]);

    var rotation_matrix = get_rotation_matrix(in.rotation);
    
    return raycast(in.world_position, model_offset, rotation_matrix, center, size, dimension);
}