extern crate chrono;
extern crate fern;
extern crate ggez;
#[macro_use]
extern crate ggez_inputty;
#[macro_use]
extern crate log;
extern crate rand;

use ggez::{Context, ContextBuilder, GameResult};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::graphics::{self, Point2};
use ggez::event::{run, Axis, Button, EventHandler, Keycode};
use ggez::nalgebra;
use ggez::timer;
use ggez_inputty::{InputHandler, InputtyResult, PhysicalInput as PI, PhysicalInputValue as PIV};
use ggez_inputty::virtual_axis::{self, VirtualAxisPhase};

const BALL_DIM: f32 = 0.005;
const BALL_MAX_VELOCITY: f32 = 0.015;
const FIELD_DIM: (f32, f32) = (1.5, 1.0);
const PADDLE_DIM: (f32, f32) = (0.01, 0.10);
const PADDLE_PAD: f32 = 0.05;
const PADDLE_MAX_VELOCITY: f32 = 0.02;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
enum Input {
    PaddleAnalog(usize),
    PaddleUp(usize),
    PaddleDown(usize),
    Pause(usize),
    Exit,
}

struct PlayerInputState {
    axis: f32,
    axis_state: VirtualAxisPhase,
    pause_timer: u32,
}

impl PlayerInputState {
    fn new() -> Self {
        PlayerInputState {
            axis: 0.0,
            axis_state: VirtualAxisPhase::Relax,
            pause_timer: 0,
        }
    }

    fn update(&mut self) {
        virtual_axis::axis_update(&mut self.axis, &self.axis_state, 0.1, 0.2, 0.1);
        if self.pause_timer > 0 {
            self.pause_timer -= 1;
        }
    }
}

struct InputState {
    paddle_l: PlayerInputState,
    paddle_r: PlayerInputState,
}

impl InputState {
    fn new() -> Self {
        InputState {
            paddle_l: PlayerInputState::new(),
            paddle_r: PlayerInputState::new(),
        }
    }

    fn update(&mut self) {
        self.paddle_l.update();
        self.paddle_r.update();
    }

    fn create_handler() -> InputHandler<Input, InputState> {
        InputHandler::<Input, InputState>::new()
            .define(
                Input::PaddleAnalog(0),
                |_state, _physical, _value| -> InputtyResult {
                    if let PIV::Axis(raw_axis) = _value {
                        _state.paddle_l.axis_state = VirtualAxisPhase::Ignore;
                        _state.paddle_l.axis = raw_axis as f32 / i16::max_value() as f32;
                    }
                    Ok(())
                },
            )
            .define(
                Input::PaddleUp(0),
                |_state, _physical, _value| -> InputtyResult {
                    virtual_axis::axis_input_neg(&mut _state.paddle_l.axis_state, _value)
                },
            )
            .define(
                Input::PaddleDown(0),
                |_state, _physical, _value| -> InputtyResult {
                    virtual_axis::axis_input_pos(&mut _state.paddle_l.axis_state, _value)
                },
            )
            .define(
                Input::PaddleAnalog(1),
                |_state, _physical, _value| -> InputtyResult {
                    if let PIV::Axis(raw_axis) = _value {
                        _state.paddle_r.axis_state = VirtualAxisPhase::Ignore;
                        _state.paddle_r.axis = raw_axis as f32 / i16::max_value() as f32;
                    }
                    Ok(())
                },
            )
            .define(
                Input::PaddleUp(1),
                |_state, _physical, _value| -> InputtyResult {
                    virtual_axis::axis_input_neg(&mut _state.paddle_r.axis_state, _value)
                },
            )
            .define(
                Input::PaddleDown(1),
                |_state, _physical, _value| -> InputtyResult {
                    virtual_axis::axis_input_pos(&mut _state.paddle_r.axis_state, _value)
                },
            )
            .bind(PI::CAxis(0, Axis::LeftY), Input::PaddleAnalog(0))
            .bind(PI::Key(Keycode::W, false), Input::PaddleUp(0))
            .bind(PI::Key(Keycode::S, false), Input::PaddleDown(0))
            .bind(PI::CAxis(1, Axis::LeftY), Input::PaddleAnalog(1))
            .bind(PI::Key(Keycode::Up, false), Input::PaddleUp(1))
            .bind(PI::Key(Keycode::Down, false), Input::PaddleDown(1))
    }
}

struct MeshBank {
    paddle: graphics::Mesh,
    ball: graphics::Mesh,
}

impl MeshBank {
    fn new(ctx: &mut Context) -> GameResult<MeshBank> {
        let paddle = graphics::MeshBuilder::new()
            .polygon(
                graphics::DrawMode::Fill,
                &[
                    Point2::new(PADDLE_DIM.0 / 2.0, PADDLE_DIM.1 / 2.0),
                    Point2::new(PADDLE_DIM.0 / 2.0, -PADDLE_DIM.1 / 2.0),
                    Point2::new(-PADDLE_DIM.0 / 2.0, -PADDLE_DIM.1 / 2.0),
                    Point2::new(-PADDLE_DIM.0 / 2.0, PADDLE_DIM.1 / 2.0),
                    Point2::new(PADDLE_DIM.0 / 2.0, PADDLE_DIM.1 / 2.0),
                ],
            )
            .build(ctx)?;

        let ball = graphics::MeshBuilder::new()
            .circle(
                graphics::DrawMode::Fill,
                Point2::new(0.0, 0.0),
                BALL_DIM,
                0.001,
            )
            .build(ctx)?;

        Ok(MeshBank { paddle, ball })
    }

    fn draw(&self, ctx: &mut Context, game_state: &GameState) -> GameResult<()> {
        graphics::draw(
            ctx,
            &self.paddle,
            Point2::new(PADDLE_PAD, game_state.paddle_l_pos),
            0.0,
        )?;
        graphics::draw(
            ctx,
            &self.paddle,
            Point2::new(FIELD_DIM.0 - PADDLE_PAD, game_state.paddle_r_pos),
            0.0,
        )?;
        graphics::draw(
            ctx,
            &self.ball,
            Point2::new(game_state.ball_pos.0, game_state.ball_pos.1),
            0.0,
        )?;
        Ok(())
    }
}

struct GameState {
    paddle_l_pos: f32,
    paddle_r_pos: f32,
    ball_pos: (f32, f32),
    ball_vel: (f32, f32),
    should_exit: bool,
}

impl GameState {
    fn new() -> Self {
        GameState {
            paddle_l_pos: FIELD_DIM.1 / 2.0,
            paddle_r_pos: FIELD_DIM.1 / 2.0,
            ball_pos: (FIELD_DIM.0 / 2.0, FIELD_DIM.1 / 2.0),
            ball_vel: (
                0.5 * BALL_MAX_VELOCITY * {
                    if rand::random::<f32>() > 0.5 {
                        1.0
                    } else {
                        -1.0
                    }
                },
                BALL_MAX_VELOCITY * (0.5 - rand::random::<f32>()),
            ),
            should_exit: false,
        }
    }

    fn update(&mut self, input_state: &InputState) {
        let paddle_l_vel = PADDLE_MAX_VELOCITY * input_state.paddle_l.axis;
        self.paddle_l_pos += paddle_l_vel;
        self.paddle_l_pos = nalgebra::clamp(
            self.paddle_l_pos,
            PADDLE_DIM.1 / 2.0,
            FIELD_DIM.1 - PADDLE_DIM.1 / 2.0,
        );

        let paddle_r_vel = PADDLE_MAX_VELOCITY * input_state.paddle_r.axis;
        self.paddle_r_pos += paddle_r_vel;
        self.paddle_r_pos = nalgebra::clamp(
            self.paddle_r_pos,
            PADDLE_DIM.1 / 2.0,
            FIELD_DIM.1 - PADDLE_DIM.1 / 2.0,
        );

        self.ball_pos.0 += self.ball_vel.0;
        if self.ball_pos.0 > FIELD_DIM.0 - BALL_DIM || self.ball_pos.0 < BALL_DIM {
            self.ball_vel.0 = -self.ball_vel.0;
        }
        self.ball_pos.0 = nalgebra::clamp(self.ball_pos.0, BALL_DIM, FIELD_DIM.0 - BALL_DIM);

        self.ball_pos.1 += self.ball_vel.1;
        if self.ball_pos.1 > FIELD_DIM.1 - BALL_DIM || self.ball_pos.1 < BALL_DIM {
            self.ball_vel.1 = -self.ball_vel.1;
        }
        self.ball_pos.1 = nalgebra::clamp(self.ball_pos.1, BALL_DIM, FIELD_DIM.1 - BALL_DIM);

        if self.ball_pos.0 < PADDLE_PAD + PADDLE_DIM.0 / 2.0
            && self.ball_pos.0 > PADDLE_PAD - PADDLE_DIM.0 / 2.0
        {
            if self.ball_pos.1 < self.paddle_l_pos + PADDLE_DIM.1 / 2.0
                && self.ball_pos.1 > self.paddle_l_pos - PADDLE_DIM.1 / 2.0
            {
                if self.ball_pos.0 < PADDLE_PAD + PADDLE_DIM.0 / 4.0
                    && self.ball_pos.0 > PADDLE_PAD - PADDLE_DIM.0 / 4.0
                    && (self.ball_pos.1 > self.paddle_l_pos + PADDLE_DIM.1 / 2.0 - PADDLE_DIM.0
                        || self.ball_pos.1 < self.paddle_l_pos - PADDLE_DIM.1 / 2.0 + PADDLE_DIM.0)
                {
                    self.ball_vel.1 = -self.ball_vel.1;
                } else {
                    self.ball_vel.0 = -self.ball_vel.0;
                    self.ball_vel.1 += paddle_l_vel;
                }
            }
        }

        if self.ball_pos.0 < FIELD_DIM.0 - PADDLE_PAD + PADDLE_DIM.0 / 2.0
            && self.ball_pos.0 > FIELD_DIM.0 - PADDLE_PAD - PADDLE_DIM.0 / 2.0
        {
            if self.ball_pos.1 < self.paddle_r_pos + PADDLE_DIM.1 / 2.0
                && self.ball_pos.1 > self.paddle_r_pos - PADDLE_DIM.1 / 2.0
            {
                if self.ball_pos.0 < FIELD_DIM.0 - PADDLE_PAD + PADDLE_DIM.0 / 4.0
                    && self.ball_pos.0 > FIELD_DIM.0 - PADDLE_PAD - PADDLE_DIM.0 / 4.0
                    && (self.ball_pos.1 > self.paddle_r_pos + PADDLE_DIM.1 / 2.0 - PADDLE_DIM.0
                        || self.ball_pos.1 < self.paddle_r_pos - PADDLE_DIM.1 / 2.0 + PADDLE_DIM.0)
                {
                    self.ball_vel.1 = -self.ball_vel.1;
                } else {
                    self.ball_vel.0 = -self.ball_vel.0;
                    self.ball_vel.1 += paddle_r_vel;
                }
            }
        }
    }
}

struct App {
    mesh: MeshBank,
    game_state: GameState,
    input_state: InputState,
    input_handler: InputHandler<Input, InputState>,
}

impl App {
    fn new(ctx: &mut Context) -> GameResult<App> {
        graphics::set_screen_coordinates(
            ctx,
            graphics::Rect::new(0.0, 0.0, FIELD_DIM.0, FIELD_DIM.1),
        )?;
        let mesh = MeshBank::new(ctx)?;
        let game_state = GameState::new();
        let input_state = InputState::new();
        let input_handler = InputState::create_handler();

        Ok(App {
            mesh,
            game_state,
            input_state,
            input_handler,
        })
    }
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_UPS: u32 = 60;
        while timer::check_update_time(ctx, DESIRED_UPS) {
            self.input_state.update();
            self.game_state.update(&self.input_state);
            if self.game_state.should_exit {
                ctx.quit()?;
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        self.mesh.draw(ctx, &self.game_state)?;
        graphics::present(ctx);
        timer::yield_now();
        Ok(())
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
        .level_for("ggez_inputty", log::LevelFilter::Debug)
        .level_for("pong", log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    let ctx = &mut ContextBuilder::new("pong", "ggez_inputty")
        .window_setup(WindowSetup::default().title("Pong!").resizable(true))
        .window_mode(
            WindowMode::default().dimensions((480.0 * FIELD_DIM.0 / FIELD_DIM.1) as u32, 480),
        )
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
