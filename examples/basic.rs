extern crate chrono;
extern crate fern;
extern crate ggez;
#[macro_use]
extern crate ggez_inputty;
#[macro_use]
extern crate log;

use ggez::{Context, ContextBuilder, GameResult};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::graphics::{self, DrawMode, Point2};
use ggez::event::{run, Axis, Button, EventHandler, Keycode};
use ggez::timer;
use ggez_inputty::{InputHandler, InputHandlerDefGen, InputtyResult, PhysicalInput as PI,
                   PhysicalInputValue as PIV};
use ggez_inputty::virtual_axis::{self, VirtualAxisInput, VirtualAxisPhase, VirtualAxisState};

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
enum Input {
    Spin(VirtualAxisInput),
    ReturnError,
    Exit,
}

struct InputState {
    should_exit: bool,
    spin_axis: VirtualAxisState,
}

impl InputState {
    fn new() -> Self {
        InputState {
            should_exit: false,
            spin_axis: VirtualAxisState::new(0.1, 0.2, 0.1),
        }
    }
}

struct App {
    rotation_angle: f32,
    mesh: graphics::Mesh,
    input_state: InputState,
    input_handler: InputHandler<Input, InputState>,
}

impl App {
    fn new(ctx: &mut Context) -> GameResult<App> {
        let input_state = InputState::new();
        let mut input_handler = InputHandler::<Input, InputState>::new();
        define_virtual_axis!(input_handler, Input::Spin, spin_axis);
        input_handler
            .define(Input::Exit, |_state, _physical, _value| -> InputtyResult {
                info!(
                    "Logical input 'Exit' triggered via {:?}: {:?}",
                    _physical, _value
                );
                _state.should_exit = true;
                Ok(())
            })
            .define(
                Input::ReturnError,
                |_state, _physical, _value| -> InputtyResult {
                    if let PIV::Button(true) = _value {
                        info!("Propagating an error");
                        Err("Oh no!")
                    } else {
                        Ok(())
                    }
                },
            )
            .bind(PI::Key(Keycode::Escape, true), Input::Exit)
            .bind(PI::Key(Keycode::E, false), Input::ReturnError)
            .bind(PI::CButton(0, Button::Back), Input::Exit)
            .bind(
                PI::CAxis(0, Axis::LeftX),
                Input::Spin(VirtualAxisInput::Analog),
            )
            .bind(
                PI::CButton(0, Button::DPadLeft),
                Input::Spin(VirtualAxisInput::Negative),
            )
            .bind(
                PI::CButton(0, Button::DPadRight),
                Input::Spin(VirtualAxisInput::Positive),
            )
            .bind(
                PI::Key(Keycode::Left, false),
                Input::Spin(VirtualAxisInput::Negative),
            )
            .bind(
                PI::Key(Keycode::Right, false),
                Input::Spin(VirtualAxisInput::Positive),
            );

        let mesh = graphics::MeshBuilder::new()
            .line(
                &[
                    Point2::new(0.0, -32.0),
                    Point2::new(-30.0, 20.0),
                    Point2::new(30.0, 20.0),
                    Point2::new(0.0, -32.0),
                ],
                1.0,
            )
            .ellipse(DrawMode::Fill, Point2::new(0.0, -7.0), 2.0, 15.0, 2.0)
            .circle(DrawMode::Fill, Point2::new(0.0, 12.0), 2.0, 2.0)
            .build(ctx)?;

        Ok(App {
            rotation_angle: 0.0,
            mesh,
            input_state,
            input_handler,
        })
    }
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_UPS: u32 = 60;
        while timer::check_update_time(ctx, DESIRED_UPS) {
            self.rotation_angle += self.input_state.spin_axis.value() / 10.0;
            let mut f = 0.0;
            self.input_state.spin_axis.update(1.0);
            if self.input_state.should_exit {
                ctx.quit()?;
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        graphics::draw(
            ctx,
            &self.mesh,
            graphics::Point2::new(150.0, 150.0),
            self.rotation_angle,
        ).unwrap();
        graphics::present(ctx);
        timer::yield_now();
        Ok(())
    }

    fn resize_event(&mut self, ctx: &mut Context, width: u32, height: u32) {
        graphics::set_screen_coordinates(
            ctx,
            graphics::Rect::new(0.0, 0.0, width as f32, height as f32),
        ).unwrap();
    }

    impl_input_handling!(input_handler, input_state);
}

pub fn main() {
    fern::Dispatch::new()
        .format(|out, msg, rec| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                chrono::Local::now().format("%H:%M:%S"),
                rec.target(),
                rec.level(),
                msg,
            ))
        })
        .level(log::LevelFilter::Warn)
        .level_for("ggez_inputty", log::LevelFilter::Trace)
        .level_for("basic", log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    let ctx = &mut ContextBuilder::new("basic", "ggez_inputty")
        .window_setup(
            WindowSetup::default()
                .title("ggez_inputty basic example!")
                .resizable(true),
        )
        .window_mode(WindowMode::default().dimensions(640, 480))
        .build()
        .unwrap();

    use std::panic::{self, AssertUnwindSafe};

    panic::set_hook(Box::new(|e| {
        let payload = match e.payload().downcast_ref::<&str>() {
            Some(line) => line,
            None => match e.payload().downcast_ref::<String>() {
                Some(line) => &line,
                None => "unknown",
            },
        };
        let location = match e.location() {
            Some(loc) => {
                let line = match loc.file().splitn(2, "src").last() {
                    Some(line) => line,
                    None => loc.file(),
                };
                format!("'{}', line {}, col {}", line, loc.line(), loc.column())
            }
            None => "unknown".to_string(),
        };
        error!("Panic: '{}' at {}", payload, location);
    }));

    match App::new(ctx) {
        Err(e) => error!("Could not initialize: {}", e),
        Ok(ref mut app) => {
            let mut result = Ok(());
            while panic::catch_unwind(AssertUnwindSafe(|| result = run(ctx, app))).is_err() {}
            match result {
                Err(e) => error!("Error occurred: {}", e),
                Ok(_) => debug!("Exited cleanly."),
            }
        }
    }
}
