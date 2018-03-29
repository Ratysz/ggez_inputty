use ggez::GameResult;
use ggez::event::{Axis, Button, Keycode, Mod, MouseButton, MouseState};
use std::hash::Hash;
use std::collections::{HashMap, VecDeque};

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
    Key(Keycode, bool),
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

type LogicalInputCallback<State> =
    Box<Fn(&mut State, PhysicalInput, PhysicalInputValue) -> GameResult<()>>;
type DynamicsCallback<State> = Box<Fn(&mut State) -> GameResult<()>>;

/// A struct containing a mapping from physical input events to callbacks.
pub struct InputHandler<LogicalInput, State>
where
    LogicalInput: Hash + Eq + Clone,
{
    definitions: HashMap<LogicalInput, LogicalInputCallback<State>>,
    dynamics: Vec<DynamicsCallback<State>>,
    bindings: HashMap<PhysicalInput, Vec<LogicalInput>>,
    input_queue: VecDeque<(LogicalInput, PhysicalInput, PhysicalInputValue)>,
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

    pub fn define(mut self, logical: LogicalInput, callback: LogicalInputCallback<State>) -> Self {
        self.definitions.insert(logical, callback);
        self
    }

    pub fn add_dynamics(mut self, callback: DynamicsCallback<State>) -> Self {
        self.dynamics.push(callback);
        self
    }

    pub fn bind(mut self, physical: PhysicalInput, logical: LogicalInput) -> Self {
        self.bindings
            .entry(physical)
            .or_insert_with(Vec::new)
            .push(logical);
        self
    }

    pub fn update(&mut self, state: &mut State) -> GameResult<()> {
        for callback in &self.dynamics {
            callback(state)?;
        }
        while let Some((logical, physical, value)) = self.input_queue.pop_front() {
            if let Some(callback) = self.definitions.get(&logical) {
                callback(state, physical, value)?;
            }
        }
        Ok(())
    }

    pub fn mouse_button_down_event(&mut self, button: MouseButton, _x: i32, _y: i32) {
        #[cfg(feature = "logging")]
        debug!(
            "raw mouse button down: {:?} | x: {} | y: {} | instance: {}",
            button, _x, _y, 0
        );
        if let Some(bindings) = self.bindings.get(&PhysicalInput::MButton(button)) {
            for logical in bindings {
                self.input_queue.push_back((
                    logical.clone(),
                    PhysicalInput::MButton(button),
                    PhysicalInputValue::Button(0, true),
                ));
            }
        };
    }

    pub fn mouse_button_up_event(&mut self, button: MouseButton, _x: i32, _y: i32) {
        #[cfg(feature = "logging")]
        debug!(
            "raw mouse button up: {:?} | x: {} | y: {} | instance: {}",
            button, _x, _y, 0
        );
        if let Some(bindings) = self.bindings.get(&PhysicalInput::MButton(button)) {
            for logical in bindings {
                self.input_queue.push_back((
                    logical.clone(),
                    PhysicalInput::MButton(button),
                    PhysicalInputValue::Button(0, false),
                ));
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
                self.input_queue.push_back((
                    logical.clone(),
                    PhysicalInput::MMotion,
                    PhysicalInputValue::XY(0, x, y, xrel, yrel),
                ));
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
                        self.input_queue.push_back((
                            logical.clone(),
                            PhysicalInput::MWheelX(true),
                            PhysicalInputValue::Button(0, true),
                        ));
                    }
                    x -= 1;
                }
            }
        } else if x < 0 {
            if let Some(bindings) = self.bindings.get(&PhysicalInput::MWheelX(false)) {
                while x < 0 {
                    for logical in bindings {
                        self.input_queue.push_back((
                            logical.clone(),
                            PhysicalInput::MWheelX(false),
                            PhysicalInputValue::Button(0, true),
                        ));
                    }
                    x += 1;
                }
            }
        }
        if y > 0 {
            if let Some(bindings) = self.bindings.get(&PhysicalInput::MWheelY(true)) {
                while y > 0 {
                    for logical in bindings {
                        self.input_queue.push_back((
                            logical.clone(),
                            PhysicalInput::MWheelY(true),
                            PhysicalInputValue::Button(0, true),
                        ));
                    }
                    y -= 1;
                }
            }
        } else if y < 0 {
            if let Some(bindings) = self.bindings.get(&PhysicalInput::MWheelY(false)) {
                while y < 0 {
                    for logical in bindings {
                        self.input_queue.push_back((
                            logical.clone(),
                            PhysicalInput::MWheelY(false),
                            PhysicalInputValue::Button(0, true),
                        ));
                    }
                    y += 1;
                }
            }
        }
    }

    pub fn key_down_event(&mut self, keycode: Keycode, _keymod: Mod, repeat: bool) {
        #[cfg(feature = "logging")]
        debug!(
            "raw key down: {} | modifiers: {:?} | repeat: {} | instance: {}",
            keycode, _keymod, repeat, 0,
        );
        if let Some(bindings) = self.bindings.get(&PhysicalInput::Key(keycode, repeat)) {
            for logical in bindings {
                self.input_queue.push_back((
                    logical.clone(),
                    PhysicalInput::Key(keycode, repeat),
                    PhysicalInputValue::Button(0, true),
                ));
            }
        }
    }

    pub fn key_up_event(&mut self, keycode: Keycode, _keymod: Mod, repeat: bool) {
        #[cfg(feature = "logging")]
        debug!(
            "raw key up: {} | modifiers: {:?} | repeat: {} | instance: {}",
            keycode, _keymod, repeat, 0,
        );
        if let Some(bindings) = self.bindings.get(&PhysicalInput::Key(keycode, repeat)) {
            for logical in bindings {
                self.input_queue.push_back((
                    logical.clone(),
                    PhysicalInput::Key(keycode, repeat),
                    PhysicalInputValue::Button(0, false),
                ));
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
                    PhysicalInput::CButton(button),
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
                    PhysicalInput::CButton(button),
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
                    PhysicalInput::CAxis(axis),
                    PhysicalInputValue::Axis(instance_id, value),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn sanity_check() {
        assert_eq!(2 + 2, 4);
    }
}
