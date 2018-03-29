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
use ggez::event::{run, Button, EventHandler, Keycode};
use ggez::timer;
use ggez_inputty::{InputHandler, PhysicalInput as PI, PhysicalInputValue as PIV};

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
enum Input {
    Toggle,
    Exit,
}

struct State {
    should_exit: bool,
    axis: f32,
}

impl State {
    fn new() -> Self {
        State {
            should_exit: false,
            axis: 0.0,
        }
    }
}

struct App {
    state: State,
    input_handler: InputHandler<Input, State>,
}

impl App {
    fn new(ctx: &mut Context) -> GameResult<App> {
        let state = State::new();

        let input_handler = InputHandler::<Input, State>::new()
            .define(
                Input::Exit,
                Box::new(|state, physical, value| -> GameResult<()> {
                    info!(
                        "Logical input 'Exit' triggered via {:?}: {:?}",
                        physical, value
                    );
                    state.should_exit = true;
                    Ok(())
                }),
            )
            //.bind(PI::Key(Keycode::Escape, false), Input::Exit)
            .bind(PI::Key(Keycode::Escape, true), Input::Exit)
            .bind(PI::CButton(Button::Back), Input::Exit);

        Ok(App {
            state,
            input_handler,
        })
    }
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_UPS: u32 = 60;
        while timer::check_update_time(ctx, DESIRED_UPS) {
            if self.state.should_exit {
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
                    Point2::new(0.0, 0.0),
                    Point2::new(-30.0, 52.0),
                    Point2::new(30.0, 52.0),
                    Point2::new(0.0, 0.0),
                ],
                1.0,
            )
            .ellipse(DrawMode::Fill, Point2::new(0.0, 25.0), 2.0, 15.0, 2.0)
            .circle(DrawMode::Fill, Point2::new(0.0, 45.0), 2.0, 2.0)
            .build(ctx)?;
        graphics::draw(ctx, &mesh, graphics::Point2::new(150.0, 150.0), 0.0).unwrap();
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

    impl_input_handling!(input_handler, state);
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

    match App::new(ctx) {
        Err(e) => {
            error!("Could not initialize: {}", e);
        }
        Ok(ref mut app) => match run(ctx, app) {
            Err(e) => {
                error!("Error occurred: {}", e);
            }
            Ok(_) => {
                debug!("Exited cleanly.");
            }
        },
    }
}
