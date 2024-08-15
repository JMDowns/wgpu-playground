use winit::{event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};

pub struct InputState {
    pub is_up_pressed: bool,
    pub is_down_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_mouse_pressed: bool,
    pub mouse_delta_x: f64,
    pub mouse_delta_y: f64,
    pub mouse_scroll_delta: MouseScrollDelta,
}

pub struct InputManager {
    pub input_state: InputState
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            input_state: InputState {
                is_up_pressed: false,
                is_down_pressed: false,
                is_left_pressed: false,
                is_right_pressed: false,
                is_forward_pressed: false,
                is_backward_pressed: false,
                is_mouse_pressed: false,
                mouse_delta_x: 0.0,
                mouse_delta_y: 0.0,
                mouse_scroll_delta: MouseScrollDelta::LineDelta(0.0, 0.0)
            },
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> (bool, bool, bool) {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(key),
                        ..
                    },
                ..
            } => 
            {
                let mut input_movement_character = false;
                let mut should_switch_wireframe = false;
                let mut should_rotate_subvoxel = false;
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
                if key == &KeyCode::ControlLeft {
                    if state == &ElementState::Pressed {
                        should_switch_wireframe = true;
                    }
                }
                if key == &KeyCode::KeyR {
                    if state == &ElementState::Pressed {
                        should_rotate_subvoxel = true;
                    }
                }
                (input_movement_character, should_switch_wireframe, should_rotate_subvoxel)
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.input_state.mouse_scroll_delta = *delta;
                (true, false, false)
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                if *state == ElementState::Pressed {
                    self.input_state.is_mouse_pressed = true;
                } else {
                    self.input_state.is_mouse_pressed = false;
                    self.input_state.mouse_delta_x = 0.0;
                    self.input_state.mouse_delta_y = 0.0;
                }
                (true, false, false)
            }
            _ => (false, false, false)
        }
    }

    pub fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        if self.input_state.is_mouse_pressed {
            self.input_state.mouse_delta_x = delta.0;
            self.input_state.mouse_delta_y = delta.1;
        }
    }
}