use super::subvoxel_gpu_data::SubvoxelGpuData;
use derivables::subvoxel_vertex::{SubvoxelVertex, generate_cube_at_center};
use cgmath::{Matrix3, Deg, Vector3, Point3, EuclideanSpace};
use super::subvoxel_object_specification::SubvoxelObjectSpecification;

pub struct SubvoxelObject {
    pub id: u32,
    pub size: Vector3<f32>,
    pub center: Point3<f32>,
    pub rotation: Vector3<Deg<f32>>,
    pub rotation_matrix: Matrix3<f32>,
    pub model_name: String,
    pub subvoxel_vertices: [SubvoxelVertex; 24]
}

impl SubvoxelObject {
    pub fn new(id: u32, spec: SubvoxelObjectSpecification) -> Self {
        let x_rotation = Matrix3::<f32>::from_angle_x(spec.initial_rotation.x);
        let y_rotation = Matrix3::<f32>::from_angle_y(spec.initial_rotation.y);
        let z_rotation = Matrix3::<f32>::from_angle_z(spec.initial_rotation.z);
        let rotation_matrix = (z_rotation * y_rotation * x_rotation);
        let subvoxel_vertices = 
            generate_cube_at_center(Point3 { x: 0.0, y: 0.0, z: 0.0 }, spec.size, id)
            .into_iter()
            .map(|vertex| vertex.rotate(&rotation_matrix) )
            .map(|vertex| vertex.translate(spec.center.to_vec()))
            .collect::<Vec<SubvoxelVertex>>().try_into().unwrap();

        Self {
            id,
            size: spec.size,
            center: spec.center,
            rotation: spec.initial_rotation,
            rotation_matrix,
            model_name: spec.model_name,
            subvoxel_vertices
        }
    }

    pub fn rotate(&mut self, rotation: Vector3<Deg<f32>>) {
        self.rotation.x += rotation.x;
        self.rotation.y += rotation.y;
        self.rotation.z += rotation.z;
        let x_rotation = Matrix3::<f32>::from_angle_x(self.rotation.x);
        let y_rotation = Matrix3::<f32>::from_angle_y(self.rotation.y);
        let z_rotation = Matrix3::<f32>::from_angle_z(self.rotation.z);
        self.rotation_matrix = (z_rotation * y_rotation * x_rotation);
        self.subvoxel_vertices = 
            generate_cube_at_center(Point3 { x: 0.0, y: 0.0, z: 0.0 }, self.size, self.id)
            .into_iter()
            .map(|vertex| vertex.rotate(&self.rotation_matrix) )
            .map(|vertex| vertex.translate(self.center.to_vec()))
            .collect::<Vec<SubvoxelVertex>>().try_into().unwrap();
    }

    pub fn to_gpu_data(&self, model_offset: u32) -> SubvoxelGpuData {
        SubvoxelGpuData {
            size_x: self.size.x, 
            size_y: self.size.y, 
            size_z: self.size.z,
            center_x: self.center.x,
            center_y: self.center.y,
            center_z: self.center.z,
            rotation_matrix: [
                self.rotation_matrix.x.into(),
                self.rotation_matrix.y.into(),
                self.rotation_matrix.z.into()
            ],
            model_offset,
        }
    }
}