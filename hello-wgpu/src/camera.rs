use cgmath::*;
use winit::event::*;
use winit::dpi::PhysicalPosition;
use instant::Duration;
use std::f32::consts::FRAC_PI_2;

use crate::state::input_state::InputState;

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

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
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
        Self {
            aspect,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
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

    pub fn process_keyboard(&mut self, button_state: &InputState) -> bool {
        let mut movement = false;
        if button_state.is_forward_pressed {
            self.amount_forward = self.speed;
            movement = true;
        }
        if button_state.is_backward_pressed {
            self.amount_backward = self.speed;
            movement = true;
        }
        if button_state.is_left_pressed {
            self.amount_left = self.speed;
            movement = true;
        }
        if button_state.is_right_pressed {
            self.amount_right = self.speed;
            movement = true;
        }
        if button_state.is_up_pressed {
            self.amount_up = self.speed;
            movement = true;
        }
        if button_state.is_down_pressed {
            self.amount_down = self.speed;
            movement = true;
        }
        movement
    }

    pub fn process_mouse(&mut self, input_state: &mut InputState, sensitivity: f64) -> bool {
        let mut movement = false;
        if input_state.mouse_delta_x.abs() > fundamentals::consts::MOUSE_SENSITIVITY {
            self.rotate_horizontal = (input_state.mouse_delta_x*4.0*sensitivity) as f32;
            movement = true;
        }
        if input_state.mouse_delta_y.abs() > fundamentals::consts::MOUSE_SENSITIVITY {
            self.rotate_vertical = (input_state.mouse_delta_y*4.0*sensitivity) as f32;
            movement = true;
        }
        movement
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

        // Same for process_keyboard
        self.amount_up = 0.0;
        self.amount_down = 0.0;
        self.amount_left = 0.0;
        self.amount_right = 0.0;
        self.amount_forward = 0.0;
        self.amount_backward = 0.0;

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