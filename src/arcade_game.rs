use std::env;
use std::path::Path;
use std::collections::HashMap;
use std::fmt::{Debug, Display};

use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::{self, Color, Image, DrawParam, Text, Scale, Font};
use ggez::conf::WindowMode;
use itertools::Itertools;
use nalgebra::Vector2;
use num_traits::identities::Zero;

use crate::intcode::{Program, Error as IntcodeError};
use crate::day13::{Arcade, Error, Tile, JoystickPosition};


struct Transition {
    to: Box<dyn Stage>,
}

trait Stage: Debug {
    fn init(&self, ctx: &mut Context, state: &mut GameState);
    fn update(&self, ctx: &mut Context, state: &mut GameState) -> GameResult<Option<Transition>>;
    fn draw(&self, ctx: &mut Context, state: &mut GameState, scale: f32) -> GameResult<Option<Transition>>;
    fn key_down_event(&self, ctx: &mut Context, state: &mut GameState, keycode: KeyCode, keymod: KeyMods, _repeat: bool) -> Option<Transition>;
    fn key_up_event(&self, ctx: &mut Context, state: &mut GameState, keycode: KeyCode, keymod: KeyMods) -> Option<Transition>;
}

#[derive(Clone, Debug, Default)]
struct StartingScreen {}

impl Stage for StartingScreen {
    fn init(&self, _ctx: &mut Context, _state: &mut GameState) {}

    fn update(&self, _ctx: &mut Context, _state: &mut GameState) -> GameResult<Option<Transition>> {
        Ok(None)
    }

    fn draw(&self, ctx: &mut Context, state: &mut GameState, _scale: f32) -> GameResult<Option<Transition>> {
        state.draw_text(ctx, 256., &TextAlign::centered(), &"PRESS SPACE")?;
        Ok(None)
    }

    fn key_down_event(&self, _ctx: &mut Context, _state: &mut GameState, _keycode: KeyCode, _keymod: KeyMods, _repeat: bool) -> Option<Transition> {
        None
    }

    fn key_up_event(&self, _ctx: &mut Context, _state: &mut GameState, keycode: KeyCode, _keymod: KeyMods) -> Option<Transition> {
        match keycode {
            KeyCode::Space => return Some(Transition { to: Box::new(GameScreen::default()) }),
            _ => {},
        }
        None
    }
}

#[derive(Clone, Debug, Default)]
struct GameScreen {}

impl Stage for GameScreen {
    fn init(&self, _ctx: &mut Context, state: &mut GameState) {
        state.arcade = state.initial_arcade.clone();
    }

    fn update(&self, _ctx: &mut Context, state: &mut GameState) -> GameResult<Option<Transition>> {
        if state.autopilot {
            debug!("autopilot on");
            if let Err(Error::Intcode(IntcodeError::Halted)) = state.arcade.autopilot() {
                return Ok(Some(Transition { to: Box::new(ScoreScreen { score: state.score() }) }));
            }
        }
        Ok(None)
    }

    fn draw(&self, ctx: &mut Context, state: &mut GameState, scale: f32) -> GameResult<Option<Transition>> {
        state.frame_counter += 1;
        if state.frame_counter >= state.speed {
            debug!("waiting for frame event");
            match state.arcade.wait_frame() {
                Err(Error::Intcode(IntcodeError::Halted)) => {
                    return Ok(Some(Transition { to: Box::new(ScoreScreen { score: state.score() }) }));
                },
                Err(_) => panic!("Arcade failed"),
                Ok(()) => {},
            }
            state.frame_counter = 0;
        }

        debug!("draw game screen");
        let framebuffer = &state.arcade.screen.framebuffer;
        let minmax = framebuffer.keys().minmax();

        if let Some((min, max)) = minmax.into_option() {
            for y in min.1 ..= max.1 {
                for x in min.0 ..= max.0 {
                    let tile = framebuffer.get(&(x, y))
                        .copied()
                        .unwrap_or_default();

                    //debug!("Rendering: {},{} {:?}", x, y, tile);

                    let sprite = state.tileset.get(&tile).unwrap();

                    let pos = Vector2::new((x - min.0) as f32, (y - min.1) as f32) * scale;
                    //let pos = Vector2::from([(x - min.0) as f32 * scale, (y - min.1) as f32 * scale]);

                    let draw_params = DrawParam::new()
                        .dest(mint::Point2::from([pos.x, pos.y]))
                        .scale(mint::Vector2::from([scale / state.tile_size, scale / state.tile_size]));

                    graphics::draw(ctx, sprite, draw_params)?;
                }
            }
        }

        state.draw_info(ctx, &mut 0, &"SCORE", Some(state.score()))?;

        Ok(None)
    }

    fn key_down_event(&self, _ctx: &mut Context, state: &mut GameState, keycode: KeyCode, _keymod: KeyMods, _repeat: bool) -> Option<Transition> {
        match keycode {
            KeyCode::A => state.arcade.set_joystick(JoystickPosition::Left),
            KeyCode::D => state.arcade.set_joystick(JoystickPosition::Right),
            _ => {},
        }
        None
    }

    fn key_up_event(&self, _ctx: &mut Context, state: &mut GameState, keycode: KeyCode, _keymod: KeyMods) -> Option<Transition> {
        match keycode {
            KeyCode::A | KeyCode::D => state.arcade.set_joystick(JoystickPosition::Neutral),
            _ => {},
        }
        None
    }
}

#[derive(Clone, Debug)]
struct ScoreScreen {
    score: i64,
}

impl Stage for ScoreScreen {
    fn init(&self, _ctx: &mut Context, _state: &mut GameState) {}

    fn update(&self, _ctx: &mut Context, _state: &mut GameState) -> GameResult<Option<Transition>> {
        Ok(None)
    }

    fn draw(&self, ctx: &mut Context, state: &mut GameState, _scale: f32) -> GameResult<Option<Transition>> {
        let message = if state.won() { "YOU WON :)" } else { "YOU LOST :(" };
        let message = format!("{}\n\nYOUR SCORE:\n\n{}", message, state.score());

        state.draw_text(ctx, 128., &TextAlign::centered(), &message)?;

        Ok(None)
    }

    fn key_down_event(&self, _ctx: &mut Context, _state: &mut GameState, _keycode: KeyCode, _keymod: KeyMods, _repeat: bool) -> Option<Transition> {
        None
    }

    fn key_up_event(&self, _ctx: &mut Context, _state: &mut GameState, keycode: KeyCode, _keymod: KeyMods) -> Option<Transition> {
        match keycode {
            KeyCode::Space => return Some(Transition { to: Box::new(GameScreen::default()) }),
            _ => {},
        }
        None
    }
}

struct TextAlign {
    absolute: Vector2<f32>,
    window: Vector2<f32>,
    text: Vector2<f32>,
}

impl TextAlign {
    pub fn centered() -> Self {
        Self {
            absolute: Vector2::zero(),
            window: Vector2::new(0.5, 0.5),
            text: Vector2::new(-0.5, -0.5),
        }
    }

    pub fn position(&self, window_size: Vector2<f32>, text_size: Vector2<f32>) -> Vector2<f32> {
        self.absolute
            + self.window.component_mul(&window_size)
            + self.text.component_mul(&text_size)

    }
}

#[derive(Clone, Debug)]
struct GameState {
    initial_arcade: Arcade,
    arcade: Arcade,
    tile_size: f32,
    autopilot: bool,
    show_fps: bool,
    tileset: HashMap<Tile, Image>,
    font: Font,
    frame_counter: usize,
    speed: usize,
}

impl GameState {
    const INFO_PADDING: f32 = 8.;
    const INFO_TEXT_SIZE: f32 = 32.;
    const INFO_NUM: usize = 4;

    pub fn score(&self) -> i64 {
        self.arcade.screen.score
    }

    pub fn won(&self) -> bool {
        self.arcade.screen.num_blocks == 0
    }

    pub fn draw_text<S: AsRef<str>>(&self, ctx: &mut Context, scale: f32, align: &TextAlign, text: &S) -> GameResult<()> {
        let mut text = Text::new(text.as_ref());
        text.set_font(self.font.clone(), Scale::uniform(scale));

        let window_size = graphics::drawable_size(ctx);
        let window_size = Vector2::new(window_size.0, window_size.1);
        let text_size =  text.dimensions(ctx);
        let text_size = Vector2::new(text_size.0 as f32, text_size.1 as f32);

        let pos = align.position(window_size, text_size);
        let draw_params = DrawParam::new()
            .dest(mint::Point2::from([pos.x, pos.y]));

        graphics::draw(ctx, &text, draw_params)?;

        Ok(())
    }

    pub fn draw_info<T: Display>(&self, ctx: &mut Context, index: &mut usize, text: &T, number: Option<i64>) -> GameResult<()> {
        let info = match number {
            Some(number) => format!("{} {:04}", text, number),
            None => format!("{}", text),
        };

        self.draw_text(ctx, Self::INFO_TEXT_SIZE, &TextAlign {
            absolute: Vector2::new(0., Self::INFO_PADDING),
            window: Vector2::new((*index as f32 + 1.) / (Self::INFO_NUM as f32 + 1.), 0.),
            text: Vector2::new(-0.5, 0.0),
        }, &info)?;

        *index += 1;
        Ok(())
    }
}

#[derive(Debug)]
struct Game {
    state: GameState,
    stage: Box<dyn Stage>,
}

impl Game {
    pub fn new(ctx: &mut Context, program: Program) -> GameResult<Self> {
        let mut tileset = HashMap::new();
        tileset.insert(Tile::Wall, Image::new(ctx, "/wall.64.png")?);
        tileset.insert(Tile::Block, Image::new(ctx, "/block.64.png")?);
        tileset.insert(Tile::Paddle, Image::new(ctx, "/paddle.64.png")?);
        tileset.insert(Tile::Ball, Image::new(ctx, "/ball.64.png")?);
        tileset.insert(Tile::Empty, Image::new(ctx, "/empty.64.png")?);

        let mut arcade = Arcade::new(program);

        arcade.load_screen().expect("Arcade failed to load screen");
        info!("Game hot-loaded");

        /*let font = Font::new(ctx, "/font.ttf")
            .map_err(|_| GameError::FilesystemError(format!("Can't parse font", )))?;*/
        let font = Font::default();
        debug!("screen loaded");
        debug!("window size: {:?}", graphics::drawable_size(ctx));

        arcade.machine.set_contant_input(JoystickPosition::default().into());

        Ok(Game {
            state: GameState {
                initial_arcade: arcade.clone(),
                arcade,
                tileset,
                tile_size: 64.,
                autopilot: false,
                font,
                show_fps: true,
                frame_counter: 0,
                speed: 10,
            },
            stage: Box::new(StartingScreen::default()),
        })
    }

    fn transition_maybe(&mut self, ctx: &mut Context, transition: Option<Transition>) {
        if let Some(transition) = transition {
            info!("Transition to: {:?}", transition.to);
            transition.to.init(ctx, &mut self.state);
            self.stage = transition.to;
        }
    }

}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        debug!("update");
        let transition = self.stage.update(ctx, &mut self.state)?;
        self.transition_maybe(ctx, transition);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::from_rgb(0x0f, 0x38, 0x0f));

        let window_size = graphics::drawable_size(ctx);
        // 36 x 19
        let screen_size = self.state.arcade.screen.screen_size().unwrap();
        let scale = (window_size.0 / (screen_size.0 as f32)).min(window_size.1 / (screen_size.1 as f32));
        debug!("window_size={:?}, screen_size={:?}, scale={}", window_size, screen_size, scale);

        let transition = self.stage.draw(ctx, &mut self.state, scale)?;
        self.transition_maybe(ctx, transition);

        let mut menu_index = 1;

        self.state.draw_info(ctx, &mut menu_index, &"SPEED", Some(self.state.speed as i64))?;

        if self.state.show_fps {
            self.state.draw_info(ctx, &mut menu_index, &"FPS", Some(ggez::timer::fps(ctx) as i64))?;
        }

        if self.state.autopilot {
            self.state.draw_info(ctx, &mut menu_index, &"AUTO", None)?;
        }

        graphics::present(ctx)
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymod: KeyMods, repeat: bool) {
        let transition = self.stage.key_down_event(ctx, &mut self.state, keycode, keymod, repeat);
        self.transition_maybe(ctx, transition);
    }

    fn key_up_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymod: KeyMods) {
        debug!("key up: {:?}", keycode);

        match keycode {
            KeyCode::J => {
                self.state.autopilot = !self.state.autopilot;
                self.state.arcade.set_joystick(JoystickPosition::Left);
            },
            KeyCode::Escape => ggez::event::quit(ctx),
            KeyCode::F3 => self.state.show_fps = !self.state.show_fps,
            KeyCode::G => {
                if self.state.speed > 0 {
                    self.state.speed -= 1;
                }
            },
            KeyCode::H => self.state.speed += 1,
            _ => {},
        }

        let transition = self.stage.key_up_event(ctx, &mut self.state, keycode, keymod);
        self.transition_maybe(ctx, transition);
    }
}

pub fn solve(program: Program, autopilot: bool) -> i64 {
    let mut cb = ContextBuilder::new("Advent of Code 2019 Arcade", "Janosch Gräf");

    let path = match env::var("ARCADE_RESOURCE_PATH") {
        Ok(path) => Path::new(&path).canonicalize().unwrap(),
        _ => {
            eprintln!("The environment variable `ARCADE_RESOURCE_PATH` must be set.");
            panic!("ARCADE_RESOURCE_PATH not set");
        },
    };
    debug!("set path to: {}", path.display());
    cb = cb.add_resource_path(path);

    let window_mode = WindowMode::default()
        .dimensions(1920.0, 1080.0)
        //.maximized(true)
        .resizable(true);
    cb = cb.window_mode(window_mode);

    let (mut ctx, mut event_loop) = cb.build().unwrap();

    let mut game = Game::new(&mut ctx, program).unwrap();
    game.state.autopilot = autopilot;

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut game) {
        Ok(_) => debug!("Exited cleanly."),
        Err(e) => debug!("Error occured: {}", e)
    }

    game.state.score()
}
