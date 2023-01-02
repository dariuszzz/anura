use std::collections::HashMap;
use winit::{event::{VirtualKeyCode, MouseButton, ElementState}, dpi::PhysicalPosition};

#[derive(Debug, Default, Clone)]
pub struct KeyPressState {
    frame_count: u32,
    was_just_released: bool
}

#[derive(Debug, Default, Clone)]
pub struct MouseButtonPressState {
    frame_count: u32, 
    was_just_released: bool,
    starting_position: PhysicalPosition<f64>
}

#[derive(Debug, Default, Clone)]
pub(crate) struct InputManager {
    pub keys: HashMap<VirtualKeyCode, KeyPressState>,
    pub mouse_buttons: HashMap<MouseButton, MouseButtonPressState>,
    pub mouse_position: PhysicalPosition<f64>,
    pub last_received_char: Option<char>
}

impl InputManager {
    pub fn update_mouse_pos(&mut self, position: &PhysicalPosition<f64>) {
        self.mouse_position = *position;
    }

    pub fn update_mouse_button(&mut self, state: &ElementState, button: &MouseButton) {
        let was_released = *state == ElementState::Released;

        let mouse_button_state = match self.mouse_buttons.remove(button) {
            Some(mut mb_state) => {
                mb_state.was_just_released = was_released;
                mb_state
            }, 
            None => {
                MouseButtonPressState {
                    frame_count: 1,
                    was_just_released: was_released,
                    starting_position: self.mouse_position
                }
            }
        };

        self.mouse_buttons.insert(*button, mouse_button_state);
    }

    pub fn update_key(&mut self, keycode: &VirtualKeyCode, state: &ElementState) {
        let was_released = *state == ElementState::Released;
        
        let key_state = match self.keys.remove(keycode) {
            Some(mut key_state) => {
                key_state.was_just_released = was_released;
                key_state
            },
            None => {
                KeyPressState {
                    frame_count: 1,
                    was_just_released: was_released
                }
            }
        };

        self.keys.insert(*keycode, key_state);
    }

    pub fn update_inputs(&mut self) {
        self.keys.iter_mut().for_each(|(_, key_state)| {
            if !key_state.was_just_released {
                key_state.frame_count += 1;
            }
        });

        self.mouse_buttons.iter_mut().for_each(|(_, mb_state)| {
            if !mb_state.was_just_released {
                mb_state.frame_count += 1;
            }
        });

        self.last_received_char = None;

        self.keys.drain_filter(|_, key_state| key_state.was_just_released);
        self.mouse_buttons.drain_filter(|_, mb_state| mb_state.was_just_released);
    }

}