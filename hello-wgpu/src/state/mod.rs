pub mod flag_state;
pub mod input_manager;
mod task_manager;

use fundamentals::consts::MOUSE_SENSITIVITY;
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

use self::input_manager::InputManager;
use self::task_manager::TaskManager;

pub struct State {
    pub gpu_manager: GPUManager,
    flag_state: FlagState,
    input_manager: InputManager,
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
                should_render_wireframe: false,
                has_moved: false,
            },
            input_manager: InputManager::new(),
            task_manager,
            world, 
            camera_controller,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.gpu_manager.resize(new_size);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        let (has_taken_input, should_switch_wireframe, should_rotate_subvoxel) = self.input_manager.input(event);
        if should_switch_wireframe {
            self.flag_state.should_render_wireframe = !self.flag_state.should_render_wireframe;
        }
        if (should_rotate_subvoxel) {
            self.gpu_manager.rotate_subvoxel_object(0);
        }
        has_taken_input
    }

    pub fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        self.input_manager.handle_mouse_motion(delta);
    }

    pub fn process_input(&mut self) {
        self.flag_state.has_moved = self.camera_controller.process_mouse(&mut self.input_manager.input_state);
        self.flag_state.has_moved = self.camera_controller.process_keyboard(&self.input_manager.input_state) || self.flag_state.has_moved;
        self.gpu_manager.process_input(&self.flag_state);
    }

    pub fn update(&mut self, dt: instant::Duration) {
        if (self.camera_controller.has_updates()) {
            self.gpu_manager.update_camera_and_reset_conroller(&mut self.camera_controller, dt);
        }
    }

    pub fn process_tasks(&mut self) {
        self.task_manager.process_tasks(self.world.clone(), &mut self.gpu_manager);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.gpu_manager.render()
    }
}

