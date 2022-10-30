pub mod input_state;
pub mod flag_state;

use input_state::InputState;
use flag_state::FlagState;
use crate::gpu_manager::GPUManager;

use crate::camera;
use crate::tasks::Task;
use crate::tasks::TaskResult;
use crate::thread_task_manager::ThreadTaskManager;
use crate::voxels::world::World;
use fundamentals::world_position::WorldPosition;
use std::sync::Arc;
use std::sync::RwLock;

use winit::{
    event::*,
    window::Window,
};

pub struct State {
    pub gpu_manager: GPUManager,
    pub flag_state: FlagState,
    pub input_state: InputState,
    pub camera_controller: camera::CameraController,
    pub thread_task_manager: ThreadTaskManager,
    pub chunk_positions_to_load: Vec<WorldPosition>,
    pub world: Arc<RwLock<World>>,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let gpu_manager = pollster::block_on(GPUManager::new(window));

        let camera_controller = camera::CameraController::new(4.0, 0.4);
        let world = Arc::new(RwLock::new(World::new()));

        let mut thread_task_manager = ThreadTaskManager::new();
        for pos in gpu_manager.vertex_gpu_data.read().unwrap().chunk_index_array.iter().rev() {
            thread_task_manager.push_task(Task::GenerateChunk { chunk_position: *pos, world: world.clone() });
        }

        Self {
            gpu_manager,
            flag_state: FlagState {
                mouse_pressed: false,
                should_render_wireframe: false,
                has_moved: false,
            },
            thread_task_manager,
            world, 
            camera_controller,
            chunk_positions_to_load: Vec::new(),
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

    pub fn process_input(&mut self) {
        let sensitivity = 1.0;
        self.flag_state.has_moved = self.camera_controller.process_mouse(&mut self.input_state, sensitivity);
        self.flag_state.has_moved = self.camera_controller.process_keyboard(&self.input_state) || self.flag_state.has_moved;
        self.gpu_manager.process_input(&self.flag_state);
    }

    pub fn update(&mut self, dt: instant::Duration) {
        self.gpu_manager.update_camera_and_reset_conroller(&mut self.camera_controller, dt);
    }

    pub fn process_tasks(&mut self) {
        let mut task_results = self.thread_task_manager.process_tasks();

        let mut chunks_generated = 0;
        let mut meshes_generated = 0;

        for task_result in task_results.drain(..) {
            match task_result {
                TaskResult::Requeue { task } => self.thread_task_manager.push_task(task),
                TaskResult::GenerateChunk { chunk_position } => {
                    println!("Generated chunk {}!", chunks_generated);
                    chunks_generated += 1;
                    self.thread_task_manager.push_task(self.gpu_manager.create_generate_chunk_mesh_task(chunk_position, self.world.clone()));
                    
                },
                TaskResult::GenerateChunkMesh { } => {
                    println!("Generated mesh {}!", meshes_generated);
                    self.gpu_manager.process_generate_chunk_mesh_task_result();
                    meshes_generated += 1;
                }
            }
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.gpu_manager.render()
    }
}

