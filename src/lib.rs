extern crate ggez;
#[macro_use]
extern crate log;

mod input_handler;
mod macros;

pub use input_handler::InputHandler;
pub use input_handler::PhysicalInput;
pub use input_handler::PhysicalInputValue;
