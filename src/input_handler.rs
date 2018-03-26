use std::hash::Hash;
use std::collections::{HashMap, VecDeque};
use ggez::Context;
use ggez::event::{Axis, Button, Keycode, Mod, MouseButton, MouseState};

/// Gathers kinds of physical (read: SDL2-specific) sources of input under a single enum.
#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum PhysicalInput {
    // TODO: Look at joysticks, etc.
    CAxis(Axis),
    CButton(Button),
    MButton(MouseButton),
    MWheelX(bool),
    MWheelY(bool),
    MMotion,
    Key(Keycode),
}

/// Facilitates passing concrete values to parsing callbacks; types are as used in SDL2.
#[derive(Debug)]
pub enum PhysicalInputValue {
    // TODO: Look at joysticks, etc.
    /// Instance ID, value.
    Axis(i32, i16),
    /// Instance ID, value.
    Button(i32, bool),
    /// Instance ID, X, Y, relative X, relative Y.
    XY(i32, i32, i32, i32, i32),
}

/// A struct containing a mapping from physical input events to callbacks.
pub struct InputHandler<LogicalInput, State>
where
    LogicalInput: Hash + Eq + Clone,
{
    definitions: HashMap<LogicalInput, Box<Fn(&mut Context, &mut State, PhysicalInputValue)>>,
    dynamics: Vec<Box<Fn(&mut Context, &mut State)>>,
    bindings: HashMap<PhysicalInput, Vec<LogicalInput>>,
    input_queue: VecDeque<(LogicalInput, PhysicalInputValue)>,
}

impl<LogicalInput, State> InputHandler<LogicalInput, State>
where
    LogicalInput: Hash + Eq + Clone,
{
    pub fn new() -> Self {
        InputHandler {
            definitions: HashMap::new(),
            dynamics: Vec::new(),
            bindings: HashMap::new(),
            input_queue: VecDeque::new(),
        }
    }

    pub fn define(
        mut self,
        logical: LogicalInput,
        callback: Box<Fn(&mut Context, &mut State, PhysicalInputValue)>,
    ) -> Self {
        self.definitions.insert(logical, callback);
        self
    }

    pub fn add_dynamics(mut self, callback: Box<Fn(&mut Context, &mut State)>) -> Self {
        self.dynamics.push(callback);
        self
    }

    pub fn bind(mut self, physical: PhysicalInput, logical: LogicalInput) -> Self {
        self.bindings
            .entry(physical)
            .or_insert(Vec::new())
            .push(logical);
        self
    }

    pub fn update(&mut self, context: &mut Context, state: &mut State) {
        for callback in self.dynamics.iter() {
            callback(context, state);
        }
        while let Some((logical, value)) = self.input_queue.pop_front() {
            if let Some(callback) = self.definitions.get(&logical) {
                callback(context, state, value);
            }
        }
    }

    pub fn mouse_button_down_event(&mut self, button: MouseButton, x: i32, y: i32) {
        #[cfg(feature = "logging")]
        debug!(
            "raw mouse button down: {:?} | x: {} | y: {} | instance: {}",
            button, x, y, 0
        );
        if let Some(bindings) = self.bindings.get(&PhysicalInput::MButton(button)) {
            for logical in bindings {
                self.input_queue
                    .push_back((logical.clone(), PhysicalInputValue::Button(0, true)));
            }
        };
    }

    pub fn mouse_button_up_event(&mut self, button: MouseButton, x: i32, y: i32) {
        #[cfg(feature = "logging")]
        debug!(
            "raw mouse button up: {:?} | x: {} | y: {} | instance: {}",
            button, x, y, 0
        );
        if let Some(bindings) = self.bindings.get(&PhysicalInput::MButton(button)) {
            for logical in bindings {
                self.input_queue
                    .push_back((logical.clone(), PhysicalInputValue::Button(0, false)));
            }
        };
    }

    pub fn mouse_motion_event(&mut self, _state: MouseState, x: i32, y: i32, xrel: i32, yrel: i32) {
        #[cfg(feature = "logging")]
        debug!(
            "raw mouse motion: x: {} | y: {} | xrel: {} | yrel: {} | instance: {}",
            x, y, xrel, yrel, 0,
        );
        if let Some(bindings) = self.bindings.get(&PhysicalInput::MMotion) {
            for logical in bindings {
                self.input_queue
                    .push_back((logical.clone(), PhysicalInputValue::XY(0, x, y, xrel, yrel)));
            }
        };
    }

    pub fn mouse_wheel_event(&mut self, x: i32, y: i32) {
        #[cfg(feature = "logging")]
        debug!("raw mouse wheel: x: {} | y: {} | instance: {}", x, y, 0);
        let (mut x, mut y) = (x, y);
        if x > 0 {
            if let Some(bindings) = self.bindings.get(&PhysicalInput::MWheelX(true)) {
                while x > 0 {
                    for logical in bindings {
                        self.input_queue
                            .push_back((logical.clone(), PhysicalInputValue::Button(0, true)));
                    }
                    x -= 1;
                }
            }
        } else if x < 0 {
            if let Some(bindings) = self.bindings.get(&PhysicalInput::MWheelX(false)) {
                while x < 0 {
                    for logical in bindings {
                        self.input_queue
                            .push_back((logical.clone(), PhysicalInputValue::Button(0, true)));
                    }
                    x += 1;
                }
            }
        }
        if y > 0 {
            if let Some(bindings) = self.bindings.get(&PhysicalInput::MWheelY(true)) {
                while y > 0 {
                    for logical in bindings {
                        self.input_queue
                            .push_back((logical.clone(), PhysicalInputValue::Button(0, true)));
                    }
                    y -= 1;
                }
            }
        } else if y < 0 {
            if let Some(bindings) = self.bindings.get(&PhysicalInput::MWheelY(false)) {
                while y < 0 {
                    for logical in bindings {
                        self.input_queue
                            .push_back((logical.clone(), PhysicalInputValue::Button(0, true)));
                    }
                    y += 1;
                }
            }
        }
    }

    pub fn key_down_event(&mut self, keycode: Keycode, keymod: Mod, repeat: bool) {
        #[cfg(feature = "logging")]
        debug!(
            "raw key down: {} | modifier: {:?} | repeat: {} | instance: {}",
            keycode, keymod, repeat, 0,
        );
        if let Some(bindings) = self.bindings.get(&PhysicalInput::Key(keycode)) {
            if !repeat {
                for logical in bindings {
                    self.input_queue
                        .push_back((logical.clone(), PhysicalInputValue::Button(0, true)));
                }
            }
        }
    }

    pub fn key_up_event(&mut self, keycode: Keycode, keymod: Mod, repeat: bool) {
        #[cfg(feature = "logging")]
        debug!(
            "raw key up: {} | modifier: {:?} | repeat: {} | instance: {}",
            keycode, keymod, repeat, 0,
        );
        if let Some(bindings) = self.bindings.get(&PhysicalInput::Key(keycode)) {
            for logical in bindings {
                self.input_queue
                    .push_back((logical.clone(), PhysicalInputValue::Button(0, false)));
            }
        }
    }

    pub fn controller_button_down_event(&mut self, button: Button, instance_id: i32) {
        #[cfg(feature = "logging")]
        debug!("raw button down: {:?} | instance: {}", button, instance_id,);
        if let Some(bindings) = self.bindings.get(&PhysicalInput::CButton(button)) {
            for logical in bindings {
                self.input_queue.push_back((
                    logical.clone(),
                    PhysicalInputValue::Button(instance_id, true),
                ));
            }
        }
    }

    pub fn controller_button_up_event(&mut self, button: Button, instance_id: i32) {
        #[cfg(feature = "logging")]
        debug!("raw button up: {:?} | instance: {}", button, instance_id,);
        if let Some(bindings) = self.bindings.get(&PhysicalInput::CButton(button)) {
            for logical in bindings {
                self.input_queue.push_back((
                    logical.clone(),
                    PhysicalInputValue::Button(instance_id, false),
                ));
            }
        }
    }

    pub fn controller_axis_event(&mut self, axis: Axis, value: i16, instance_id: i32) {
        #[cfg(feature = "logging")]
        debug!(
            "raw axis event: {:?} | {} | instance: {}",
            axis, value, instance_id
        );
        if let Some(bindings) = self.bindings.get(&PhysicalInput::CAxis(axis)) {
            for logical in bindings {
                self.input_queue.push_back((
                    logical.clone(),
                    PhysicalInputValue::Axis(instance_id, value),
                ));
            }
        }
    }
}
