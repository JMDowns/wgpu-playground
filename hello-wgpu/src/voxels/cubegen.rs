use crate::voxels::vertex::Vertex;

pub fn generate_cube(x: f32,y: f32, z: f32, wcolor: wgpu::Color) -> Vec<Vertex> {
    [
        Vertex::new(x,y,z,wcolor),
        Vertex::new(x-1.0,y,z,wcolor),
        Vertex::new(x-1.0,y-1.0,z,wcolor),
        Vertex::new(x,y-1.0,z,wcolor),
        Vertex::new(x,y,z-1.0,wcolor),
        Vertex::new(x-1.0,y,z-1.0,wcolor),
        Vertex::new(x-1.0,y-1.0,z-1.0,wcolor),
        Vertex::new(x,y-1.0,z-1.0,wcolor),
    ].to_vec()
}

pub fn generate_cube_indices(num_cubes: u16) -> Vec<u16> {
    let mut cube_indices: Vec<u16> = Vec::new();
    for i in 0..num_cubes {
        cube_indices = [
            cube_indices,
            [
                // Front Face
                0+8*i,1+8*i,2+8*i,
                0+8*i,2+8*i,3+8*i,
                // Back Face
                5+8*i,4+8*i,7+8*i,
                5+8*i,7+8*i,6+8*i,
                // Right Face
                1+8*i,5+8*i,6+8*i,
                1+8*i,6+8*i,2+8*i,
                // Left Face
                4+8*i,0+8*i,3+8*i,
                4+8*i,3+8*i,7+8*i,
                // Bottom Face
                2+8*i,6+8*i,7+8*i,
                2+8*i,7+8*i,3+8*i,
                // Bottom Face
                5+8*i,1+8*i,0+8*i,
                5+8*i,0+8*i,4+8*i,
            ].to_vec()
        ].concat();
    }
    cube_indices
}

pub fn generate_nxn_cube_from_lower_front_left(x: f32, y: f32, z: f32, n: u16, wcolor: wgpu::Color) -> Vec<Vertex>{
    let mut cubes = Vec::new();
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                cubes = [
                    cubes,
                    generate_cube(x+i as f32, y+j as f32, z+k as f32, wcolor)
                ].concat();
            }
        }
    }
    
    cubes
}