use ggez::event::{Axis, Button, Keycode, Mod, MouseButton, MouseState};
use std::fmt::Debug;
use std::hash::Hash;
use std::collections::HashMap;

/// Gathers kinds of physical (read: SDL2-specific) sources of input under a single enum.
#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum PhysicalInput {
    // TODO: Look at joysticks, etc.
    /// Instance ID, axis.
    CAxis(i32, Axis),
    /// Instante ID, button.
    CButton(i32, Button),
    MButton(MouseButton),
    /// Positive/negative.
    MWheelX(bool),
    /// Positive/negative.
    MWheelY(bool),
    MMotion,
    /// Keycode, repeated.
    Key(Keycode, bool),
}

/// Facilitates passing concrete values to parsing callbacks; types are as used in SDL2.
#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum PhysicalInputValue {
    // TODO: Look at joysticks, etc.
    /// Raw value.
    Axis(i16),
    /// Down/up.
    Button(bool),
    /// X, Y, relative X, relative Y.
    XY(i32, i32, i32, i32),
}

type LogicalInputCallback<State> =
    Fn(&mut State, PhysicalInput, PhysicalInputValue) -> InputtyResult;
pub type InputtyResult = Result<(), &'static str>;

/// A struct containing a mapping from physical input events to callbacks.
pub struct InputHandler<LogicalInput, State>
where
    LogicalInput: Hash + Eq + Clone + Debug,
{
    definitions: HashMap<LogicalInput, Box<LogicalInputCallback<State>>>,
    bindings: HashMap<PhysicalInput, Vec<LogicalInput>>,
}

impl<LogicalInput, State> InputHandler<LogicalInput, State>
where
    LogicalInput: Hash + Eq + Clone + Debug,
{
    pub fn new() -> Self {
        InputHandler {
            definitions: HashMap::new(),
            bindings: HashMap::new(),
        }
    }

    pub fn define<F>(mut self, logical: LogicalInput, callback: F) -> Self
    where
        F: Fn(&mut State, PhysicalInput, PhysicalInputValue) -> InputtyResult + 'static,
    {
        self.definitions.insert(logical, Box::new(callback));
        self
    }

    pub fn bind(mut self, physical: PhysicalInput, logical: LogicalInput) -> Self {
        self.bindings
            .entry(physical)
            .or_insert_with(Vec::new)
            .push(logical);
        self
    }

    pub fn resolve_and_invoke(
        &mut self,
        state: &mut State,
        physical: PhysicalInput,
        value: PhysicalInputValue,
    ) {
        if let Some(bindings) = self.bindings.get(&physical) {
            for logical in bindings {
                if let Some(callback) = self.definitions.get(&logical) {
                    if let Err(e) = callback(state, physical, value) {
                        error!(
                            "Logical input callback {:?} ( {:?}, {:?} ) returned an error: {}",
                            logical, &physical, &value, e
                        );
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
        trace!(
            "raw mouse button down: {:?} | x: {} | y: {} | instance: {}",
            button,
            _x,
            _y,
            0
        );
        self.resolve_and_invoke(
            state,
            PhysicalInput::MButton(button),
            PhysicalInputValue::Button(true),
        );
    }

    pub fn mouse_button_up_event(
        &mut self,
        state: &mut State,
        button: MouseButton,
        _x: i32,
        _y: i32,
    ) {
        trace!(
            "raw mouse button up: {:?} | x: {} | y: {} | instance: {}",
            button,
            _x,
            _y,
            0
        );
        self.resolve_and_invoke(
            state,
            PhysicalInput::MButton(button),
            PhysicalInputValue::Button(false),
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
        trace!(
            "raw mouse motion: x: {} | y: {} | xrel: {} | yrel: {} | instance: {}",
            x,
            y,
            xrel,
            yrel,
            0,
        );
        self.resolve_and_invoke(
            state,
            PhysicalInput::MMotion,
            PhysicalInputValue::XY(x, y, xrel, yrel),
        );
    }

    pub fn mouse_wheel_event(&mut self, state: &mut State, x: i32, y: i32) {
        trace!("raw mouse wheel: x: {} | y: {} | instance: {}", x, y, 0);
        let (mut x, mut y) = (x, y);
        if x > 0 {
            while x > 0 {
                self.resolve_and_invoke(
                    state,
                    PhysicalInput::MWheelX(true),
                    PhysicalInputValue::Button(true),
                );
                x -= 1;
            }
        } else if x < 0 {
            while x < 0 {
                self.resolve_and_invoke(
                    state,
                    PhysicalInput::MWheelX(false),
                    PhysicalInputValue::Button(true),
                );
                x += 1;
            }
        }
        if y > 0 {
            while y > 0 {
                self.resolve_and_invoke(
                    state,
                    PhysicalInput::MWheelY(true),
                    PhysicalInputValue::Button(true),
                );

                y -= 1;
            }
        } else if y < 0 {
            while y < 0 {
                self.resolve_and_invoke(
                    state,
                    PhysicalInput::MWheelY(false),
                    PhysicalInputValue::Button(true),
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
        trace!(
            "raw key down: {} | modifiers: {:?} | repeat: {} | instance: {}",
            keycode,
            _keymod,
            repeat,
            0,
        );
        self.resolve_and_invoke(
            state,
            PhysicalInput::Key(keycode, repeat),
            PhysicalInputValue::Button(true),
        );
    }

    pub fn key_up_event(
        &mut self,
        state: &mut State,
        keycode: Keycode,
        _keymod: Mod,
        repeat: bool,
    ) {
        trace!(
            "raw key up: {} | modifiers: {:?} | repeat: {} | instance: {}",
            keycode,
            _keymod,
            repeat,
            0,
        );
        self.resolve_and_invoke(
            state,
            PhysicalInput::Key(keycode, repeat),
            PhysicalInputValue::Button(false),
        );
    }

    pub fn controller_button_down_event(
        &mut self,
        state: &mut State,
        button: Button,
        instance_id: i32,
    ) {
        trace!("raw button down: {:?} | instance: {}", button, instance_id,);
        self.resolve_and_invoke(
            state,
            PhysicalInput::CButton(instance_id, button),
            PhysicalInputValue::Button(true),
        );
    }

    pub fn controller_button_up_event(
        &mut self,
        state: &mut State,
        button: Button,
        instance_id: i32,
    ) {
        trace!("raw button up: {:?} | instance: {}", button, instance_id,);
        self.resolve_and_invoke(
            state,
            PhysicalInput::CButton(instance_id, button),
            PhysicalInputValue::Button(false),
        );
    }

    pub fn controller_axis_event(
        &mut self,
        state: &mut State,
        axis: Axis,
        value: i16,
        instance_id: i32,
    ) {
        trace!(
            "raw axis event: {:?} | {} | instance: {}",
            axis,
            value,
            instance_id
        );
        self.resolve_and_invoke(
            state,
            PhysicalInput::CAxis(instance_id, axis),
            PhysicalInputValue::Axis(value),
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
