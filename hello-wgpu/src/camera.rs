use cgmath::*;
use fastapprox::fast;
use fundamentals::world_position::WorldPosition;
use fundamentals::consts::CHUNK_DIMENSION;
use winit::event::*;
use winit::dpi::PhysicalPosition;
use instant::Duration;
use std::f32::consts::FRAC_PI_2;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    pub view_x_vec: cgmath::Vector3<f32>,
    pub view_y_vec: cgmath::Vector3<f32>,
    pub view_z_vec: cgmath::Vector3<f32>
}

impl Camera {
    pub fn new<
        V: Into<Point3<f32>>,
        Y: Into<Rad<f32>>,
        P: Into<Rad<f32>>,
    >(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        let yaw_rad = yaw.into();
        let pitch_rad = pitch.into();
        let (yaw_sin, yaw_cos) = yaw_rad.sin_cos();
        let (pitch_sin, pitch_cos) = pitch_rad.sin_cos();
        Self {
            position: position.into(),
            yaw: yaw_rad,
            pitch: pitch_rad,
            view_x_vec: cgmath::Vector3::new(yaw_cos, pitch_sin, yaw_sin).normalize(),
            view_y_vec: cgmath::Vector3::new(-pitch_sin, pitch_cos, 0.0).normalize(),
            view_z_vec: cgmath::Vector3::new(-yaw_sin, 0.0, -yaw_cos).normalize(),
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_to_rh(
            self.position,
            Vector3::new(
                self.yaw.0.cos(),
                self.pitch.0.sin(),
                self.yaw.0.sin(),
            ).normalize(),
            Vector3::unit_y(),
        )
    }
}

pub struct Plane {
    d: f32,
    normal: cgmath::Vector3<f32>
}

impl Plane {
    pub fn distance_to_point(&self, point: cgmath::Vector3<f32>) -> f32 {
        self.normal.dot(point) + self.d
    }
}

pub struct Frustum {
    front_plane: Plane,
    back_plane: Plane,
    left_plane: Plane,
    right_plane: Plane,
    top_plane: Plane,
    bottom_plane: Plane
}

impl Frustum {
    pub fn does_chunk_intersect_frustum(&self, chunk_position: &WorldPosition) -> bool {
        if self.front_plane.distance_to_point(Frustum::calculate_positive_vertex(&self.front_plane, chunk_position).to_vec()) < 0.0 {
            return false;
        }
        if self.back_plane.distance_to_point(Frustum::calculate_positive_vertex(&self.back_plane, chunk_position).to_vec()) < 0.0 {
            return false;
        }
        if self.left_plane.distance_to_point(Frustum::calculate_positive_vertex(&self.left_plane, chunk_position).to_vec()) < 0.0 {
            return false;
        }
        if self.right_plane.distance_to_point(Frustum::calculate_positive_vertex(&self.right_plane, chunk_position).to_vec()) < 0.0 {
            return false;
        }
        if self.top_plane.distance_to_point(Frustum::calculate_positive_vertex(&self.top_plane, chunk_position).to_vec()) < 0.0 {
            return false;
        }
        if self.bottom_plane.distance_to_point(Frustum::calculate_positive_vertex(&self.bottom_plane, chunk_position).to_vec()) < 0.0 {
            return false;
        }
        true
    }

    fn calculate_positive_vertex(plane: &Plane, chunk_position: &WorldPosition) -> cgmath::Point3<f32> {
        let xmin = (chunk_position.x * CHUNK_DIMENSION) as f32;
        let ymin = (chunk_position.y * CHUNK_DIMENSION) as f32;
        let zmin = (chunk_position.z * CHUNK_DIMENSION) as f32;
        let xmax = xmin + CHUNK_DIMENSION as f32;
        let ymax = ymin + CHUNK_DIMENSION as f32;
        let zmax = zmin + CHUNK_DIMENSION as f32;

        let mut p = cgmath::point3(xmin, ymin, zmin);
        if plane.normal.x >= 0.0 {
            p.x = xmax;
        }
        if plane.normal.y >= 0.0 {
            p.y = ymax;
        }
        if plane.normal.z >= 0.0 {
            p.z = zmax;
        }
        p
    }
}

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
    wnear: f32,
    hnear: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(
        width: u32,
        height: u32,
        fovy: F,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let aspect = width as f32 / height as f32;
        let fovy: Rad<f32> = fovy.into();
        let tan_val = fast::tan(fovy.0);
        let hnear = 2.0 * tan_val * znear;
        let wnear = hnear * aspect;
        Self {
            aspect,
            fovy,
            znear,
            zfar,
            wnear,
            hnear,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }

    pub fn calculate_frustum(&self, camera: &Camera) -> Frustum {
        let camera_position = camera.position;
        let forward_vec = camera.view_x_vec;
        let up_vec = camera.view_y_vec;
        let right_vec = camera.view_z_vec;

        let near_point = camera_position + forward_vec*self.znear;
        let far_point = camera_position + forward_vec*self.zfar;
        let left_normal = ((near_point - right_vec * self.wnear / 2.0) - camera_position).normalize().cross(up_vec);
        let right_normal = up_vec.cross(((near_point + right_vec * self.wnear / 2.0) - camera_position).normalize());
        let bottom_normal = right_vec.cross(((near_point - up_vec * self.hnear / 2.0) - camera_position).normalize());
        let top_normal = ((near_point + up_vec * self.hnear / 2.0) - camera_position).normalize().cross(right_vec);

        Frustum {
            front_plane: Plane { d: -1.0*forward_vec.dot(near_point.to_vec()), normal: forward_vec },
            back_plane: Plane { d: forward_vec.dot(far_point.to_vec()), normal: -1.0*forward_vec },
            left_plane: Plane { d: -1.0*left_normal.dot(camera_position.to_vec()), normal: left_normal },
            right_plane: Plane { d: -1.0*right_normal.dot(camera_position.to_vec()), normal: right_normal },
            top_plane: Plane { d: -1.0*top_normal.dot(camera_position.to_vec()), normal: top_normal },
            bottom_plane: Plane { d: -1.0*bottom_normal.dot(camera_position.to_vec()), normal: bottom_normal },
        }
    }
}

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool{
        let amount = if state == ElementState::Pressed { self.speed } else { 0.0 };
        println!("{:?} {:?}", key, state);
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.amount_forward = amount;
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.amount_backward = amount;
                true
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.amount_left = amount;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.amount_right = amount;
                true
            }
            VirtualKeyCode::Space => {
                self.amount_up = amount;
                true
            }
            VirtualKeyCode::LShift => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64, sensitivity: f64) {
        self.rotate_horizontal = (mouse_dx*4.0*sensitivity) as f32;
        self.rotate_vertical = (mouse_dy*4.0*sensitivity) as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition {
                y: scroll,
                ..
            }) => *scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();
        let scrollward = Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

        // Adjust normals
        camera.view_x_vec.x = camera.yaw.cos();
        camera.view_x_vec.y = camera.pitch.sin();
        camera.view_x_vec.z = camera.yaw.sin();
        camera.view_x_vec = camera.view_x_vec.normalize();

        camera.view_y_vec.x = -camera.pitch.sin();
        camera.view_y_vec.y = camera.pitch.cos();
        camera.view_y_vec = camera.view_y_vec.normalize();

        camera.view_z_vec.x = -camera.yaw.sin();
        camera.view_z_vec.z = camera.yaw.cos();
        camera.view_z_vec = camera.view_z_vec.normalize();

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
            camera.pitch = Rad(SAFE_FRAC_PI_2);
        }
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have to conver the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self { view_proj: cgmath::Matrix4::identity().into()}
    }

    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}