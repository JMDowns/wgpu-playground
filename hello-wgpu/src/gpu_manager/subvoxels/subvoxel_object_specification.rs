use cgmath::{Matrix3, Deg, Vector3, Point3, EuclideanSpace};

pub struct SubvoxelObjectSpecification {
    pub size: Vector3<f32>,
    pub center: Point3<f32>,
    pub initial_rotation: Vector3<Deg<f32>>,
    pub model_name: String
}