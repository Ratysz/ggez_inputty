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
use ggez::nalgebra;
use ggez::timer;
use ggez_inputty::{InputHandler, InputtyResult, PhysicalInput as PI, PhysicalInputValue as PIV};

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
enum Input {
    SpinAnalog,
    SpinDigitalPos,
    SpinDigitalNeg,
    ReturnError,
    Exit,
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
enum VirtualAxisState {
    Increase,
    Decrease,
    Relax,
    Ignore,
}

struct InputState {
    should_exit: bool,
    spin_axis: f32,
    spin_axis_state: VirtualAxisState,
}

impl InputState {
    fn new() -> Self {
        InputState {
            should_exit: false,
            spin_axis: 0.0,
            spin_axis_state: VirtualAxisState::Relax,
        }
    }
}

struct App {
    //font: graphics::Font,
    rotation_angle: f32,
    input_state: InputState,
    input_handler: InputHandler<Input, InputState>,
}

impl App {
    fn new(ctx: &mut Context) -> GameResult<App> {
        //let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf", 48)?;
        let input_state = InputState::new();
        let input_handler = InputHandler::<Input, InputState>::new()
            .define(Input::Exit, |state, physical, value| -> InputtyResult {
                info!(
                    "Logical input 'Exit' triggered via {:?}: {:?}",
                    physical, value
                );
                state.should_exit = true;
                Ok(())
            })
            .define(
                Input::ReturnError,
                |state, physical, value| -> InputtyResult {
                    if let PIV::Button(true) = value {
                        info!("Propagating an error");
                        Err("Oh no!")
                    } else {
                        Ok(())
                    }
                },
            )
            .define(
                Input::SpinAnalog,
                |state, physical, value| -> InputtyResult {
                    if let PIV::Axis(raw_axis) = value {
                        state.spin_axis_state = VirtualAxisState::Ignore;
                        state.spin_axis = raw_axis as f32 / i16::max_value() as f32;
                    }
                    Ok(())
                },
            )
            .define(
                Input::SpinDigitalPos,
                |state, physical, value| -> InputtyResult {
                    if let PIV::Button(raw_button) = value {
                        if raw_button {
                            state.spin_axis_state = VirtualAxisState::Increase;
                        } else if state.spin_axis_state != VirtualAxisState::Decrease {
                            state.spin_axis_state = VirtualAxisState::Relax;
                        }
                    }
                    Ok(())
                },
            )
            .define(
                Input::SpinDigitalNeg,
                |state, physical, value| -> InputtyResult {
                    if let PIV::Button(raw_button) = value {
                        if raw_button {
                            state.spin_axis_state = VirtualAxisState::Decrease;
                        } else if state.spin_axis_state != VirtualAxisState::Increase {
                            state.spin_axis_state = VirtualAxisState::Relax;
                        }
                    }
                    Ok(())
                },
            )
            .bind(PI::Key(Keycode::Escape, true), Input::Exit)
            .bind(PI::Key(Keycode::E, false), Input::ReturnError)
            .bind(PI::CButton(0, Button::Back), Input::Exit)
            .bind(PI::CAxis(0, Axis::LeftX), Input::SpinAnalog)
            .bind(PI::CButton(0, Button::DPadLeft), Input::SpinDigitalNeg)
            .bind(PI::CButton(0, Button::DPadRight), Input::SpinDigitalPos)
            .bind(PI::Key(Keycode::Left, false), Input::SpinDigitalNeg)
            .bind(PI::Key(Keycode::Right, false), Input::SpinDigitalPos)
            .bind(PI::Key(Keycode::Left, true), Input::SpinDigitalNeg)
            .bind(PI::Key(Keycode::Right, true), Input::SpinDigitalPos);

        Ok(App {
            //font,
            rotation_angle: 0.0,
            input_state,
            input_handler,
        })
    }
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_UPS: u32 = 60;
        while timer::check_update_time(ctx, DESIRED_UPS) {
            self.rotation_angle += self.input_state.spin_axis / 10.0;
            match self.input_state.spin_axis_state {
                VirtualAxisState::Relax => {
                    if self.input_state.spin_axis > 0.1 {
                        self.input_state.spin_axis -= 0.1;
                    } else if self.input_state.spin_axis < -0.1 {
                        self.input_state.spin_axis += 0.1;
                    } else {
                        self.input_state.spin_axis = 0.0;
                    }
                }
                VirtualAxisState::Increase => {
                    self.input_state.spin_axis += 0.1;
                }
                VirtualAxisState::Decrease => {
                    self.input_state.spin_axis -= 0.1;
                }
                VirtualAxisState::Ignore => (),
            }
            self.input_state.spin_axis = nalgebra::clamp(self.input_state.spin_axis, -1.0, 1.0);
            //self.input_state.
            if self.input_state.should_exit {
                ctx.quit()?;
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
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
        graphics::draw(
            ctx,
            &mesh,
            graphics::Point2::new(150.0, 150.0),
            self.rotation_angle,
        ).unwrap();
        /*let text = graphics::Text::new(
            ctx,
            &format!("Spin: {}", self.input_state.spin_axis),
            &self.font,
        ).unwrap();
        graphics::draw(ctx, &text, graphics::Point2::new(50.0, 250.0), 0.0);*/
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
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
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
