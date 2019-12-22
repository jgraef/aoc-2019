use std::env;
use std::path::Path;
use std::collections::{HashMap, VecDeque};

use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::{self, Color, Image, DrawParam, Text, Scale, Font};
use itertools::Itertools;
use mint::Point2;

use crate::intcode::{Program, Error as IntcodeError};
use crate::day13::{Arcade, Error, Tile, JoystickPosition, Instruction};
use ggez::conf::WindowMode;

struct Game {
    tileset: HashMap<Tile, Image>,
    arcade: Arcade,
    tile_size: f32,
    ball_x: i64,
    paddle_x: i64,
    autopilot: bool,
    keys: VecDeque<KeyCode>,
    time_warp: bool,
    font: Font,
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
        println!("screen loaded");
        println!("window size: {:?}", graphics::drawable_size(ctx));

        arcade.machine.set_contant_input(JoystickPosition::default().into());

        Ok(Game {
            arcade,
            tileset,
            tile_size: 64.,
            ball_x: 0,
            paddle_x: 0,
            autopilot: false,
            keys: VecDeque::with_capacity(10),
            time_warp: true,
            font: Font::default(),
        })
    }

    fn control(&mut self) -> Result<(), Error> {
        self.arcade.wait_until(|arcade| {
            arcade.screen.last_instruction
                .as_ref()
                .map(|instruction| instruction.is_ball() || instruction.is_paddle())
                .unwrap_or(false)
        })?;

        if let Some(instruction) = &self.arcade.screen.last_instruction {
            match instruction {
                Instruction::Draw { tile: Tile::Ball, x, .. } => {
                    self.ball_x = *x;
                },
                Instruction::Draw { tile: Tile::Paddle, x, .. } => {
                    self.paddle_x = *x;
                },
                _ => {},
            }
        }

        println!("autopilot: ball_x={}, paddle_x={}", self.ball_x, self.paddle_x);

        if self.ball_x < self.paddle_x {
            println!("autopilot: left");
            self.arcade.set_joystick(JoystickPosition::Left);
        }
        else if self.ball_x > self.paddle_x {
            println!("autopilot: right");
            self.arcade.set_joystick(JoystickPosition::Right);
        }
        else {
            self.arcade.set_joystick(JoystickPosition::Neutral);
        }

        Ok(())
    }

    fn quit(&mut self, ctx: &mut Context) {
        println!("score: {}", self.arcade.screen.score);
        ggez::event::quit(ctx);
    }

    pub fn score(&self) -> i64 {
        self.arcade.screen.score
    }
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.autopilot {
            //println!("autopilot on");
            if let Err(Error::Intcode(IntcodeError::Halted)) = self.control() {
                self.quit(ctx);
            }

        }
        else {
            //println!("pc: {:?}", self.arcade.machine.pc());
            if !self.time_warp {
                self.arcade.wait_frame().expect("Arcade failed");
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::from_rgb(0x0f, 0x38, 0x0f));

        let framebuffer = &self.arcade.screen.framebuffer;
        let minmax = framebuffer.keys().minmax();

        let window_size = graphics::drawable_size(ctx);
        // 36 x 19
        let screen_size = self.arcade.screen.screen_size().unwrap();
        let scale = (window_size.0 / (screen_size.0 as f32)).min(window_size.1 / (screen_size.1 as f32)) / self.tile_size;
        //println!("window_size={:?}, screen_size={:?}, scale={}", window_size, screen_size, scale);

        if let Some((min, max)) = minmax.into_option() {
            for y in min.1 ..= max.1 {
                for x in min.0 ..= max.0 {
                    let tile = framebuffer.get(&(x, y))
                        .copied()
                        .unwrap_or_default();

                    //println!("Rendering: {},{} {:?}", x, y, tile);

                    let sprite = self.tileset.get(&tile).unwrap();

                    let draw_params = DrawParam::new()
                        .dest(Point2::from([(x - min.0) as f32 * scale * self.tile_size, (y - min.1) as f32 * scale * self.tile_size]))
                        .scale(Point2::from([scale, scale]));

                    graphics::draw(ctx, sprite, draw_params)?;
                }
            }
        }

        let fps = ggez::timer::fps(ctx);
        let mut text = Text::new(format!("FPS: {:.1}", fps));
        text.set_font(self.font.clone(), Scale::uniform(32.));
        let draw_params = DrawParam::new()
            .dest(Point2::from([window_size.0 - (text.width(ctx) as f32) - 100., 100.]));
        graphics::draw(ctx, &text, draw_params)?;

        graphics::present(ctx)
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods, _repeat: bool) {
        //println!("key down: {:?}", keycode);
        match keycode {
            KeyCode::A => self.arcade.set_joystick(JoystickPosition::Left),
            KeyCode::D => self.arcade.set_joystick(JoystickPosition::Right),
            KeyCode::J => {
                self.autopilot = true;
                self.arcade.set_joystick(JoystickPosition::Left);
            },
            KeyCode::Escape => ggez::event::quit(ctx),
            KeyCode::LShift => {
                if self.time_warp {
                    //println!("timewarp: wait update");
                    self.arcade.wait_frame().expect("Arcade failed");
                }
            },
            _ => {},
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        //println!("key up: {:?}", keycode);
        self.arcade.set_joystick(JoystickPosition::Neutral);

        self.keys.push_back(keycode);
        if self.keys.len() == 4 {
            self.keys.pop_front();
        }
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
    println!("set path to: {}", path.display());
    cb = cb.add_resource_path(path);

    let window_mode = WindowMode::default()
        .dimensions(1920.0, 1080.0)
        //.maximized(true)
        .resizable(true);
    cb = cb.window_mode(window_mode);

    let (mut ctx, mut event_loop) = cb.build().unwrap();

    let mut game = Game::new(&mut ctx, program).unwrap();
    game.autopilot = autopilot;

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }

    game.score()
}
