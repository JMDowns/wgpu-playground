pub mod flag_state;
pub mod input_manager;
mod task_manager;

use fundamentals::consts::MOUSE_SENSITIVITY;
use flag_state::FlagState;
use fundamentals::loge;
use winit::application::ApplicationHandler;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::keyboard::PhysicalKey;
use winit::window::WindowId;
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

pub struct State<'a> {
    pub gpu_manager: GPUManager<'a>,
    flag_state: FlagState,
    input_manager: InputManager,
    task_manager: TaskManager,
    camera_controller: camera::CameraController,
    pub world: Arc<RwLock<World>>,
    pub window: &'a Window,
    pub last_render_time: instant::Instant
}

impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &'a Window) -> Self {
        let gpu_manager = pollster::block_on(GPUManager::new(window));

        let camera_controller = camera::CameraController::new(MOVEMENT_SPEED, MOUSE_SENSITIVITY);
        let world = Arc::new(RwLock::new(World::new()));

        let mut task_manager = TaskManager::new();
        for pos in gpu_manager.chunk_index_state.read().unwrap().chunk_index_array.iter().rev() {
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
            window,
            last_render_time: instant::Instant::now()
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

impl<'a> ApplicationHandler for State<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Your application got resumed.
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if window_id == self.window.id() && !self.input(&event) {
            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                    ..
                } => event_loop.exit(),
                WindowEvent::Resized(physical_size) => {
                    self.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    let now = instant::Instant::now();
                    let dt = now - self.last_render_time;
                    if dt.as_millis() > 0 {
                        self.last_render_time = now;
                        self.process_input();
                        self.update(dt);
                        match self.render() {
                            Ok(_) => {}
                            //Reconfigure if surface is lost
                            Err(wgpu::SurfaceError::Lost) => self.resize(self.gpu_manager.surface_state.size),
                            //System is out of memory, so we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                            Err(_e) => {
                                loge!("{:?}", _e)
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion{ delta, } => self.handle_mouse_motion(delta),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.process_tasks();
        self.window.request_redraw();
    }
}

