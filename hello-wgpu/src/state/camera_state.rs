use crate::camera;

pub struct CameraState {
    pub camera: camera::Camera,
    pub camera_uniform: camera::CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub projection: camera::Projection,
    pub camera_controller: camera::CameraController,
}