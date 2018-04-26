use ggez::nalgebra;
use input_handler::{InputHandler, InputHandlerDefGen, InputtyResult, PhysicalInputValue};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum VirtualAxisInput {
    Analog,
    Positive,
    Negative,
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum VirtualAxisPhase {
    Increase,
    Decrease,
    Relax,
    Ignore,
}

pub struct VirtualAxisState {
    value: f32,
    phase: VirtualAxisPhase,
    delta: f32,
    delta_reverse: f32,
    delta_relax: f32,
}

impl VirtualAxisState {
    pub fn new(delta: f32, delta_reverse: f32, delta_relax: f32) -> Self {
        VirtualAxisState {
            value: 0.0,
            phase: VirtualAxisPhase::Ignore,
            delta,
            delta_reverse,
            delta_relax,
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn update(&mut self, delta_time: f32) {
        axis_update(
            &mut self.value,
            &self.phase,
            delta_time * self.delta,
            delta_time * self.delta_reverse,
            delta_time * self.delta_relax,
        );
    }

    pub fn input_analog(&mut self, value: PhysicalInputValue) -> InputtyResult {
        axis_input_analog(&mut self.value, &mut self.phase, value)
    }

    pub fn input_pos(&mut self, value: PhysicalInputValue) -> InputtyResult {
        axis_input_pos(&mut self.phase, value)
    }

    pub fn input_neg(&mut self, value: PhysicalInputValue) -> InputtyResult {
        axis_input_neg(&mut self.phase, value)
    }
}

#[macro_export]
macro_rules! define_virtual_axis {
    ($handler:ident, $logical:path, $state:ident) => {
        $handler.define(
            $logical(VirtualAxisInput::Analog),
            |_state, _physical, _value| -> InputtyResult {
                _state.$state.input_analog(_value)
            });
        $handler.define(
            $logical(VirtualAxisInput::Positive),
            |_state, _physical, _value| -> InputtyResult {
                _state.$state.input_pos(_value)
            });
        $handler.define(
            $logical(VirtualAxisInput::Negative),
            |_state, _physical, _value| -> InputtyResult {
                _state.$state.input_neg(_value)
            });
    };
}

/*impl<
    LogicalInput,
    State,
> InputHandlerDefGen<
    LogicalInput,
    State,
    (LogicalInput, LogicalInput, LogicalInput),
    VirtualAxisState,
> for InputHandler<LogicalInput, State>
where
    LogicalInput: Hash + Eq + Clone + Debug,
{
    fn define_exp<F>(
        &mut self,
        logical: (LogicalInput, LogicalInput, LogicalInput),
        state_extractor: F,
    ) -> &mut InputHandler<LogicalInput, State>
    where
        F: 'static + Fn(&mut State) -> &mut VirtualAxisState,
    {
        self.define(
            logical.0,
            move |_state, _physical, _value| -> InputtyResult {
                state_extractor(_state).input_analog(_value)
            },
        );
        self.define(
            logical.1,
            move |_state, _physical, _value| -> InputtyResult {
                state_extractor(_state).input_pos(_value)
            },
        );
        self.define(
            logical.2,
            move |_state, _physical, _value| -> InputtyResult {
                state_extractor(_state).input_neg(_value)
            },
        )
    }
}*/

pub fn axis_update(
    axis_value: &mut f32,
    axis_state: &VirtualAxisPhase,
    delta: f32,
    delta_reverse: f32,
    delta_relax: f32,
) {
    let mut val = *axis_value;
    match *axis_state {
        VirtualAxisPhase::Relax => {
            if val > delta_relax {
                val -= delta_relax;
            } else if val < -delta_relax {
                val += delta_relax;
            } else {
                val = 0.0;
            }
        }
        VirtualAxisPhase::Increase => {
            if val > 0.0 {
                val += delta;
            } else {
                val += delta_reverse;
            }
        }
        VirtualAxisPhase::Decrease => {
            if val < 0.0 {
                val -= delta;
            } else {
                val -= delta_reverse;
            }
        }
        VirtualAxisPhase::Ignore => (),
    }
    *axis_value = nalgebra::clamp(val, -1.0, 1.0);
}

pub fn axis_input_analog(
    axis_value: &mut f32,
    axis_state: &mut VirtualAxisPhase,
    value: PhysicalInputValue,
) -> InputtyResult {
    if let PhysicalInputValue::Axis(raw_axis) = value {
        *axis_state = VirtualAxisPhase::Ignore;
        *axis_value = raw_axis as f32 / i16::max_value() as f32;
    }
    Ok(())
}

pub fn axis_input_pos(
    axis_state: &mut VirtualAxisPhase,
    value: PhysicalInputValue,
) -> InputtyResult {
    if let PhysicalInputValue::Button(raw_button) = value {
        if raw_button {
            *axis_state = VirtualAxisPhase::Increase;
        } else if *axis_state != VirtualAxisPhase::Decrease {
            *axis_state = VirtualAxisPhase::Relax;
        }
    }
    Ok(())
}

pub fn axis_input_neg(
    axis_state: &mut VirtualAxisPhase,
    value: PhysicalInputValue,
) -> InputtyResult {
    if let PhysicalInputValue::Button(raw_button) = value {
        if raw_button {
            *axis_state = VirtualAxisPhase::Decrease;
        } else if *axis_state != VirtualAxisPhase::Increase {
            *axis_state = VirtualAxisPhase::Relax;
        }
    }
    Ok(())
}
