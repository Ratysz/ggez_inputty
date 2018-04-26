extern crate ggez;
#[macro_use]
extern crate log;

mod input_handler;
mod macros;
pub mod virtual_axis;

pub use input_handler::InputHandler;
pub use input_handler::InputHandlerDefGen;
pub use input_handler::InputtyResult;
pub use input_handler::PhysicalInput;
pub use input_handler::PhysicalInputValue;
