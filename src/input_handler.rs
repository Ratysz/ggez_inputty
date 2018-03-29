use ggez::GameResult;
use ggez::event::{Axis, Button, Keycode, Mod, MouseButton, MouseState};
use std::hash::Hash;
use std::collections::HashMap;

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
    Box<Fn(&mut State, &PhysicalInput, &PhysicalInputValue) -> GameResult<()>>;

/// A struct containing a mapping from physical input events to callbacks.
pub struct InputHandler<LogicalInput, State>
where
    LogicalInput: Hash + Eq + Clone,
{
    definitions: HashMap<LogicalInput, LogicalInputCallback<State>>,
    bindings: HashMap<PhysicalInput, Vec<LogicalInput>>,
}

impl<LogicalInput, State> InputHandler<LogicalInput, State>
where
    LogicalInput: Hash + Eq + Clone,
{
    pub fn new() -> Self {
        InputHandler {
            definitions: HashMap::new(),
            bindings: HashMap::new(),
        }
    }

    pub fn define(mut self, logical: LogicalInput, callback: LogicalInputCallback<State>) -> Self {
        self.definitions.insert(logical, callback);
        self
    }

    pub fn bind(mut self, physical: PhysicalInput, logical: LogicalInput) -> Self {
        self.bindings
            .entry(physical)
            .or_insert_with(Vec::new)
            .push(logical);
        self
    }

    fn resolve(&mut self, state: &mut State, physical: PhysicalInput, value: PhysicalInputValue) {
        if let Some(bindings) = self.bindings.get(&physical) {
            for logical in bindings {
                if let Some(callback) = self.definitions.get(&logical) {
                    if let Err(e) = callback(state, &physical, &value) {
                        error!("{}", e);
                    }
                }
            }
        }
    }

    pub fn mouse_button_down_event(
        &mut self,
        state: &mut State,
        button: MouseButton,
        _x: i32,
        _y: i32,
    ) {
        debug!(
            "raw mouse button down: {:?} | x: {} | y: {} | instance: {}",
            button, _x, _y, 0
        );
        self.resolve(
            state,
            PhysicalInput::MButton(button),
            PhysicalInputValue::Button(0, true),
        );
    }

    pub fn mouse_button_up_event(
        &mut self,
        state: &mut State,
        button: MouseButton,
        _x: i32,
        _y: i32,
    ) {
        debug!(
            "raw mouse button up: {:?} | x: {} | y: {} | instance: {}",
            button, _x, _y, 0
        );
        self.resolve(
            state,
            PhysicalInput::MButton(button),
            PhysicalInputValue::Button(0, false),
        );
    }

    pub fn mouse_motion_event(
        &mut self,
        state: &mut State,
        _state: MouseState,
        x: i32,
        y: i32,
        xrel: i32,
        yrel: i32,
    ) {
        debug!(
            "raw mouse motion: x: {} | y: {} | xrel: {} | yrel: {} | instance: {}",
            x, y, xrel, yrel, 0,
        );
        self.resolve(
            state,
            PhysicalInput::MMotion,
            PhysicalInputValue::XY(0, x, y, xrel, yrel),
        );
    }

    pub fn mouse_wheel_event(&mut self, state: &mut State, x: i32, y: i32) {
        debug!("raw mouse wheel: x: {} | y: {} | instance: {}", x, y, 0);
        let (mut x, mut y) = (x, y);
        if x > 0 {
            while x > 0 {
                self.resolve(
                    state,
                    PhysicalInput::MWheelX(true),
                    PhysicalInputValue::Button(0, true),
                );
                x -= 1;
            }
        } else if x < 0 {
            while x < 0 {
                self.resolve(
                    state,
                    PhysicalInput::MWheelX(false),
                    PhysicalInputValue::Button(0, true),
                );
                x += 1;
            }
        }
        if y > 0 {
            while y > 0 {
                self.resolve(
                    state,
                    PhysicalInput::MWheelY(true),
                    PhysicalInputValue::Button(0, true),
                );

                y -= 1;
            }
        } else if y < 0 {
            while y < 0 {
                self.resolve(
                    state,
                    PhysicalInput::MWheelY(false),
                    PhysicalInputValue::Button(0, true),
                );
                y += 1;
            }
        }
    }

    pub fn key_down_event(
        &mut self,
        state: &mut State,
        keycode: Keycode,
        _keymod: Mod,
        repeat: bool,
    ) {
        debug!(
            "raw key down: {} | modifiers: {:?} | repeat: {} | instance: {}",
            keycode, _keymod, repeat, 0,
        );
        self.resolve(
            state,
            PhysicalInput::Key(keycode, repeat),
            PhysicalInputValue::Button(0, true),
        );
    }

    pub fn key_up_event(
        &mut self,
        state: &mut State,
        keycode: Keycode,
        _keymod: Mod,
        repeat: bool,
    ) {
        debug!(
            "raw key up: {} | modifiers: {:?} | repeat: {} | instance: {}",
            keycode, _keymod, repeat, 0,
        );
        self.resolve(
            state,
            PhysicalInput::Key(keycode, repeat),
            PhysicalInputValue::Button(0, false),
        );
    }

    pub fn controller_button_down_event(
        &mut self,
        state: &mut State,
        button: Button,
        instance_id: i32,
    ) {
        debug!("raw button down: {:?} | instance: {}", button, instance_id,);
        self.resolve(
            state,
            PhysicalInput::CButton(button),
            PhysicalInputValue::Button(instance_id, true),
        );
    }

    pub fn controller_button_up_event(
        &mut self,
        state: &mut State,
        button: Button,
        instance_id: i32,
    ) {
        debug!("raw button up: {:?} | instance: {}", button, instance_id,);
        self.resolve(
            state,
            PhysicalInput::CButton(button),
            PhysicalInputValue::Button(instance_id, false),
        );
    }

    pub fn controller_axis_event(
        &mut self,
        state: &mut State,
        axis: Axis,
        value: i16,
        instance_id: i32,
    ) {
        debug!(
            "raw axis event: {:?} | {} | instance: {}",
            axis, value, instance_id
        );
        self.resolve(
            state,
            PhysicalInput::CAxis(axis),
            PhysicalInputValue::Axis(instance_id, value),
        );
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn sanity_check() {
        assert_eq!(2 + 2, 4);
    }
}
