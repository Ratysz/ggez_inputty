use ggez::nalgebra;
use input_handler::{InputtyResult, PhysicalInputValue};

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum VirtualAxisState {
    Increase,
    Decrease,
    Relax,
    Ignore,
}

pub fn axis_update(
    value: &mut f32,
    state: &VirtualAxisState,
    delta: f32,
    reverse_delta: f32,
    relax_delta: f32,
) {
    let mut val = *value;
    match *state {
        VirtualAxisState::Relax => {
            if val > relax_delta {
                val -= relax_delta;
            } else if val < -relax_delta {
                val += relax_delta;
            } else {
                val = 0.0;
            }
        }
        VirtualAxisState::Increase => {
            if val > 0.0 {
                val += delta;
            } else {
                val += reverse_delta;
            }
        }
        VirtualAxisState::Decrease => {
            if val < 0.0 {
                val -= delta;
            } else {
                val -= reverse_delta;
            }
        }
        VirtualAxisState::Ignore => (),
    }
    *value = nalgebra::clamp(val, -1.0, 1.0);
}

pub fn axis_input_pos(
    axis_state: &mut VirtualAxisState,
    value: PhysicalInputValue,
) -> InputtyResult {
    if let PhysicalInputValue::Button(raw_button) = value {
        if raw_button {
            *axis_state = VirtualAxisState::Increase;
        } else if *axis_state != VirtualAxisState::Decrease {
            *axis_state = VirtualAxisState::Relax;
        }
    }
    Ok(())
}

pub fn axis_input_neg(
    axis_state: &mut VirtualAxisState,
    value: PhysicalInputValue,
) -> InputtyResult {
    if let PhysicalInputValue::Button(raw_button) = value {
        if raw_button {
            *axis_state = VirtualAxisState::Decrease;
        } else if *axis_state != VirtualAxisState::Increase {
            *axis_state = VirtualAxisState::Relax;
        }
    }
    Ok(())
}
