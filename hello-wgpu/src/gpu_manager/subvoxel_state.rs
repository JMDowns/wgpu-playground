use std::collections::HashMap;
use num_traits::Zero;
use wgpu::{Device, util::DeviceExt, Queue};
use cgmath::{EuclideanSpace, Matrix3, Point3, Rad, Vector3, Deg, Vector4};
use derivables::subvoxel_vertex::{SubvoxelVertex, generate_cube_at_center};
use bytemuck::{Zeroable, Pod};
use std::sync::{Arc, RwLock};

pub struct SubvoxelObjectSpecification {
    pub size: Vector3<f32>,
    pub subvoxel_size: Vector3<u32>,
    pub center: Point3<f32>,
    pub initial_rotation: Vector3<Deg<f32>>,
    pub subvoxel_vec: Vec<u32>
}

pub struct SubvoxelObject {
    pub id: u32,
    pub size: Vector3<f32>,
    pub subvoxel_size: Vector3<u32>,
    pub center: Point3<f32>,
    pub rotation: Vector3<Deg<f32>>,
    pub rotation_matrix: Matrix3<f32>,
    pub subvoxel_vec: Vec<u32>,
    pub subvoxel_vertices: [SubvoxelVertex; 24]
}

impl SubvoxelObject {
    pub fn new(id: u32, spec: SubvoxelObjectSpecification) -> Self {
        let x_rotation = Matrix3::<f32>::from_angle_x(spec.initial_rotation.x);
        let y_rotation = Matrix3::<f32>::from_angle_y(spec.initial_rotation.y);
        let z_rotation = Matrix3::<f32>::from_angle_z(spec.initial_rotation.z);
        let rotation_matrix = (z_rotation * y_rotation * x_rotation);
        let subvoxel_vertices = 
            generate_cube_at_center(Point3 { x: 0.0, y: 0.0, z: 0.0 }, spec.size)
            .into_iter()
            .map(|vertex| vertex.rotate(&rotation_matrix) )
            .map(|vertex| vertex.translate(spec.center.to_vec()))
            .collect::<Vec<SubvoxelVertex>>().try_into().unwrap();

        Self {
            id,
            size: spec.size,
            subvoxel_size: spec.subvoxel_size,
            center: spec.center,
            rotation: spec.initial_rotation,
            rotation_matrix,
            subvoxel_vec: spec.subvoxel_vec,
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
            generate_cube_at_center(Point3 { x: 0.0, y: 0.0, z: 0.0 }, self.size)
            .into_iter()
            .map(|vertex| vertex.rotate(&self.rotation_matrix) )
            .map(|vertex| vertex.translate(self.center.to_vec()))
            .collect::<Vec<SubvoxelVertex>>().try_into().unwrap();
    }

    pub fn to_gpu_data(&self) -> SubvoxelGpuData {
        let rotation_matrix_x = self.rotation_matrix.x.extend(0.0);
        let rotation_matrix_y = self.rotation_matrix.y.extend(0.0);
        let rotation_matrix_z = self.rotation_matrix.z.extend(0.0);
        SubvoxelGpuData { 
            rotation_padding_1: 0,
            rotation_padding_2: 0,
            rotation_padding_3: 0,
            size_x: self.size.x, 
            size_y: self.size.y, 
            size_z: self.size.z, 
            subvoxel_size_x: self.subvoxel_size.x, 
            subvoxel_size_y: self.subvoxel_size.y, 
            subvoxel_size_z: self.subvoxel_size.z,
            center_x: self.center.x,
            center_y: self.center.y,
            center_z: self.center.z,
            rotation_matrix: [
                rotation_matrix_x.into(),
                rotation_matrix_y.into(),
                rotation_matrix_z.into()
            ],
            sv_id: self.id
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct SubvoxelGpuData {
    pub rotation_matrix: [[f32; 4]; 3],
    pub rotation_padding_1: u32,
    pub rotation_padding_2: u32,
    pub rotation_padding_3: u32,
    pub size_x: f32,
    pub size_y: f32,
    pub size_z: f32,
    pub subvoxel_size_x: u32,
    pub subvoxel_size_y: u32,
    pub subvoxel_size_z: u32,
    pub center_x: f32,
    pub center_y: f32,
    pub center_z: f32,
    pub sv_id: u32,
}

pub struct SubvoxelState {
    pub subvoxel_objects: Vec<SubvoxelObject>,
    pub sv_data_buffer: wgpu::Buffer,
    pub subvoxel_bind_group_layout: wgpu::BindGroupLayout,
    pub subvoxel_bind_group: wgpu::BindGroup,
    pub queue: Arc<RwLock<Queue>>
}

impl SubvoxelState {
    pub fn new(device: &Device, queue: Arc<RwLock<Queue>>) -> Self {
        let subvoxel_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None
                    }
                ],
                label: Some("subvoxel_bind_group_layout"),
            });

        let sv_data_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Subvoxel Data Buffer"),
                size: (std::mem::size_of::<SubvoxelGpuData>() * 1) as u64,
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }
        );

        let subvoxel_bind_group = device.create_bind_group( &wgpu::BindGroupDescriptor {
            label: Some("Subvoxel Bind Group"),
            layout: &subvoxel_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: sv_data_buffer.as_entire_binding(),
                }
            ]
        });

        Self {
            subvoxel_objects: Vec::new(),
            sv_data_buffer,
            subvoxel_bind_group_layout,
            subvoxel_bind_group,
            queue
        }
    }

    pub fn add_subvoxel_object(&mut self, spec: SubvoxelObjectSpecification) -> usize {
        let id = self.subvoxel_objects.len();
        let object = SubvoxelObject::new(id as u32, spec);
        self.queue.read().unwrap().write_buffer(&self.sv_data_buffer, id as u64 * std::mem::size_of::<SubvoxelGpuData>() as u64, bytemuck::cast_slice(&[object.to_gpu_data()]));
        self.subvoxel_objects.push(object);
        return id;
    }

    pub fn rotate(&mut self, subvoxel_id: usize, rotation: Vector3<Deg<f32>>) {
        self.subvoxel_objects.get_mut(subvoxel_id).unwrap().rotate(rotation);
    }

    pub fn get_subvoxel_object(&self, id: usize) -> &SubvoxelObject {
        self.subvoxel_objects.get(id).unwrap()
    }

    pub fn apply_changes_to_sv_data(&self, id: usize) {
        let sv_object = self.get_subvoxel_object(id);
        self.queue.read().unwrap().write_buffer(&self.sv_data_buffer, id as u64 * std::mem::size_of::<SubvoxelGpuData>() as u64, bytemuck::cast_slice(&[sv_object.to_gpu_data()]));
    }
}