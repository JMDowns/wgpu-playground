pub mod input_state;
pub mod flag_state;
mod task_manager;

use fundamentals::consts::MOUSE_SENSITIVITY;
use input_state::InputState;
use flag_state::FlagState;
use crate::gpu_manager::GPUManager;

use crate::camera;
use crate::tasks::Task;
use crate::voxels::world::World;
use std::sync::Arc;
use std::sync::RwLock;

use fundamentals::consts::MOVEMENT_SPEED;

use winit::{
    event::*,
    window::Window,
};

use self::task_manager::TaskManager;

pub struct State {
    pub gpu_manager: GPUManager,
    flag_state: FlagState,
    input_state: InputState,
    task_manager: TaskManager,
    camera_controller: camera::CameraController,
    pub world: Arc<RwLock<World>>,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let gpu_manager = pollster::block_on(GPUManager::new(window));

        let camera_controller = camera::CameraController::new(MOVEMENT_SPEED, MOUSE_SENSITIVITY);
        let world = Arc::new(RwLock::new(World::new()));

        let mut task_manager = TaskManager::new();
        for pos in gpu_manager.vertex_gpu_data.read().unwrap().chunk_index_array.iter().rev() {
            task_manager.push_task(Task::GenerateChunk { chunk_position: *pos, world: world.clone() });
        }

        Self {
            gpu_manager,
            flag_state: FlagState {
                mouse_pressed: false,
                should_render_wireframe: false,
                has_moved: false,
            },
            task_manager,
            world, 
            camera_controller,
            input_state: input_state::InputState {
                is_up_pressed: false,
                is_down_pressed: false,
                is_left_pressed: false,
                is_right_pressed: false,
                is_forward_pressed: false,
                is_backward_pressed: false,
                mouse_delta_x: 0.0,
                mouse_delta_y: 0.0,
                mouse_scroll_delta: MouseScrollDelta::LineDelta(0.0, 0.0)
            },
            
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.gpu_manager.resize(new_size);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => 
            {
                let mut input_movement_character = false;
                if key == &fundamentals::consts::FORWARD_KEY {
                    self.input_state.is_forward_pressed = state == &ElementState::Pressed;
                    input_movement_character = true;
                }
                if key == &fundamentals::consts::BACKWARD_KEY {
                    self.input_state.is_backward_pressed = state == &ElementState::Pressed;
                    input_movement_character = true;
                }
                if key == &fundamentals::consts::LEFT_KEY {
                    self.input_state.is_left_pressed = state == &ElementState::Pressed;
                    input_movement_character = true;
                }
                if key == &fundamentals::consts::RIGHT_KEY {
                    self.input_state.is_right_pressed = state == &ElementState::Pressed;
                    input_movement_character = true;
                }
                if key == &fundamentals::consts::UP_KEY {
                    self.input_state.is_up_pressed = state == &ElementState::Pressed;
                    input_movement_character = true;
                }
                if key == &fundamentals::consts::DOWN_KEY {
                    self.input_state.is_down_pressed = state == &ElementState::Pressed;
                    input_movement_character = true;
                }
                if key == &VirtualKeyCode::LControl {
                    if state == &ElementState::Pressed {
                        self.flag_state.should_render_wireframe = !self.flag_state.should_render_wireframe;
                    }
                }
                input_movement_character
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.input_state.mouse_scroll_delta = *delta;
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                if *state == ElementState::Pressed {
                    self.flag_state.mouse_pressed = true;
                } else {
                    self.flag_state.mouse_pressed = false;
                    self.input_state.mouse_delta_x = 0.0;
                    self.input_state.mouse_delta_y = 0.0;
                }
                true
            }
            _ => false
        }
    }

    pub fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        if self.flag_state.mouse_pressed {
            self.input_state.mouse_delta_x = delta.0;
            self.input_state.mouse_delta_y = delta.1;
        }
    }

    pub fn process_input(&mut self) {
        self.flag_state.has_moved = self.camera_controller.process_mouse(&mut self.input_state);
        self.flag_state.has_moved = self.camera_controller.process_keyboard(&self.input_state) || self.flag_state.has_moved;
        self.gpu_manager.process_input(&self.flag_state);
    }

    pub fn update(&mut self, dt: instant::Duration) {
        self.gpu_manager.update_camera_and_reset_conroller(&mut self.camera_controller, dt);
    }

    pub fn process_tasks(&mut self) {
        self.task_manager.process_tasks(self.world.clone(), &mut self.gpu_manager);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.gpu_manager.render()
    }
}

