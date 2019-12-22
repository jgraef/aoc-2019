use std::convert::{TryFrom, TryInto};
use std::collections::BTreeMap;
use std::fmt::{self, Write};
use std::cmp::Ordering;

use aoc_runner_derive::{aoc, aoc_generator};
use failure::Fail;
use itertools::Itertools;

use crate::intcode::{Machine, Program, Error as IntcodeError};
use crate::util;


#[derive(Clone, Debug, Fail)]
pub enum Error {
    #[fail(display = "Intcode error: {}", _0)]
    Intcode(#[cause] IntcodeError),
    #[fail(display = "Invalid tile value: {}", _0)]
    InvalidTile(i64),
    #[fail(display = "Incomplete instruction")]
    IncompleteInstruction,
}

impl From<IntcodeError> for Error {
    fn from(e: IntcodeError) -> Self {
        Self::Intcode(e)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Tile {
    Empty,
    Wall,
    Block,
    Paddle,
    Ball,
}

impl TryFrom<i64> for Tile {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Empty),
            1 => Ok(Self::Wall),
            2 => Ok(Self::Block),
            3 => Ok(Self::Paddle),
            4 => Ok(Self::Ball),
            _ => Err(Error::InvalidTile(value))
        }
    }
}

impl From<Tile> for char {
    fn from(tile: Tile) -> Self {
        match tile {
            Tile::Empty => ' ',
            Tile::Wall => '#',
            Tile::Block => '█',
            Tile::Paddle => '|',
            Tile::Ball => '⬤',
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Empty
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Instruction {
    Draw {
        x: i64,
        y: i64,
        tile: Tile,
    },
    Score {
        score: i64,
    }
}

impl Instruction {
    pub fn is_frame(&self) -> bool {
        debug!("is_frame: {:?}", self);
        match self {
            /*Instruction::Draw { tile: Tile::Block, .. }
            | Instruction::Draw { tile: Tile::Ball, .. }
            | Instruction::Draw { tile: Tile::Paddle, .. }
            | Instruction::Draw { tile: Tile::Wall, .. } => true,*/
            //Instruction::Draw { x: 37, y: 20, .. } => true,
            Instruction::Draw { tile: Tile::Ball, .. } => true,
            _ => false,
        }
    }

    pub fn is_update(&self) -> bool {
        match self {
            Instruction::Draw { tile: Tile::Ball, .. }
            | Instruction::Draw { tile: Tile::Paddle, .. }
            | Instruction::Score { .. } => true,
            _ => false,
        }
    }

    pub fn is_ball(&self) -> bool {
        match self {
            Instruction::Draw { tile: Tile::Ball, .. } => true,
            _ => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Instruction::Draw { tile: Tile::Empty, .. } => true,
            _ => false,
        }
    }

    pub fn is_paddle(&self) -> bool {
        match self {
            Instruction::Draw { tile: Tile::Paddle, .. } => true,
            _ => false,
        }
    }

    pub fn paddle_x(&self) -> Option<i64> {
        debug!("paddle_x: {:?}", self);
        match self {
            Instruction::Draw { tile: Tile::Paddle, x, .. } => Some(*x),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Screen {
    pub framebuffer: BTreeMap<(i64, i64), Tile>,
    pub last_instruction: Option<Instruction>,
    pub score: i64,
    pub ready: bool,
    pub paddle_x: i64,
    pub ball_x: i64,
    pub num_blocks: usize,
}

impl Screen {
    pub fn run_instruction(&mut self, instruction: &Instruction) {
        debug!("screen: instruction: {:?}", instruction);
        self.last_instruction = Some(instruction.clone());
        match instruction {
            Instruction::Draw { x, y, tile } => {
                match tile {
                    Tile::Paddle => self.paddle_x = *x,
                    Tile::Ball => self.ball_x = *x,
                    Tile::Block => self.num_blocks += 1,
                    Tile::Empty => {
                        match self.framebuffer.get(&(*x, *y)) {
                            Some(Tile::Block) => self.num_blocks -= 1,
                            _ => {},
                        }
                    }
                    _ => {},
                }
                self.framebuffer.insert((*x, *y), *tile);
            }
            Instruction::Score { score } => {
                debug!("score: {}", score);
                self.score = *score;
            }
        }

    }

    pub fn screen_size(&self) -> Option<(i64, i64)> {
        let (_, max) = self.framebuffer.keys().minmax().into_option()?;
        debug!("screen size: {} x {}", max.0, max.1);
        Some((max.0 + 1, max.1 + 1))
    }

    pub fn find(&self, tile: Tile) -> Option<(i64, i64)> {
        self.framebuffer.iter()
            .find(|(_, other)| tile == **other)
            .map(|(pos, _)| pos)
            .copied()
    }
}

impl fmt::Display for Screen {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let minmax = self.framebuffer.keys().minmax();
        if let Some((min, max)) = minmax.into_option() {
            for y in min.1 ..= max.1 {
                for x in min.0 ..= max.0 {
                    let tile = self.framebuffer.get(&(x, y))
                        .copied()
                        .unwrap_or_default();
                    f.write_char(tile.into())?;
                }
                f.write_char('\n')?;
            }
        }
        write!(f, "Score: {}", self.score)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum JoystickPosition {
    Neutral,
    Left,
    Right,
}

impl From<JoystickPosition> for i64 {
    fn from(value: JoystickPosition) -> Self {
        match value {
            JoystickPosition::Neutral => 0,
            JoystickPosition::Left => -1,
            JoystickPosition::Right => 1,
        }
    }
}

impl Default for JoystickPosition {
    fn default() -> Self {
        Self::Neutral
    }
}

#[derive(Clone, Debug)]
pub struct Arcade {
    pub machine: Machine,
    pub screen: Screen,
    pub program: Program,
}

impl Arcade {
    pub fn new(program: Program) -> Self {
        let mut arcade = Self {
            machine: Machine::new(program.clone()),
            screen: Screen::default(),
            program,
        };

        arcade.init_machine();

        arcade
    }

    fn read_instruction(&mut self) -> Result<Option<Instruction>, Error> {
        debug!("read instruction");
        let a = if let Some(a) = self.machine.next_output()? {
            a
        }
        else {
            return Ok(None);
        };
        debug!("read instruction: a = {:?}", a);
        let b = if let Some(b) = self.machine.next_output()? {
            b
        }
        else {
            return Ok(None);
        };
        debug!("read instruction: b = {:?}", b);
        let c = if let Some(c) = self.machine.next_output()? {
            c
        } else {
            return Ok(None);
        };
        debug!("read instruction: c = {:?}", c);

        let instruction = match (a, b, c) {
            (-1, 0, score) => {
                Instruction::Score {
                    score,
                }
            },
            (x, y, tile) => {
                Instruction::Draw {
                    x,
                    y,
                    tile: tile.try_into()?,
                }
            }
        };

        Ok(Some(instruction))
    }

    pub fn step(&mut self) -> Result<(), Error> {
        debug!("arcade: step");
        if let Some(instruction) = self.read_instruction()? {
            self.screen.run_instruction(&instruction);
            debug!("instruction: {:?}", instruction);
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Error> {
        while !self.machine.is_halted() {
            self.step()?;
        }
        Ok(())
    }

    pub fn run_until<F: FnMut(&mut Self) -> bool>(&mut self, mut f: F) -> Result<(), Error> {
        debug!("run_until: f() = {:?}", f(self));
        while !f(self) {
            debug!("run_until: step");
            self.step()?;
        }
        Ok(())
    }

    pub fn wait_until<F: FnMut(&mut Self) -> bool>(&mut self, f: F) -> Result<(), Error> {
        self.run_until(|arcade| {
            arcade.screen.last_instruction
                .as_ref()
                .map(|instruction| instruction.is_empty()).unwrap_or_default()
        })?;
        self.run_until(f)?;
        Ok(())
    }

    pub fn wait_frame(&mut self) -> Result<(), Error> {
        self.wait_until(|arcade| {
            arcade.screen.last_instruction
                .as_ref()
                .map(|instruction| instruction.is_frame())
                .unwrap_or(false)
            })
    }

    pub fn load_screen(&mut self) -> Result<(), Error> {
        self.run_until(|arcade| arcade.screen.screen_size() == Some((37, 20)))
    }

    pub fn set_joystick(&mut self, joystick: JoystickPosition) {
        self.machine.set_contant_input(joystick.into())
    }

    pub fn autopilot(&mut self) -> Result<(), Error> {
        let joystick = match self.screen.ball_x.cmp(&self.screen.paddle_x) {
            Ordering::Equal => JoystickPosition::Neutral,
            Ordering::Less => JoystickPosition::Left,
            Ordering::Greater => JoystickPosition::Right,
        };

        debug!("autopilot: ball_x={}, paddle_x={}, joystick={:?}", self.screen.ball_x, self.screen.paddle_x, joystick);

        self.set_joystick(joystick);

        Ok(())
    }

    fn init_machine(&mut self) {
        // Initialize joystick position
        debug!("initialize joystick");
        self.machine.set_contant_input(JoystickPosition::default().into());

        // Make machine free
        info!("Hacking coin slot");
        self.machine.set_data(0, 2);
    }
}


#[aoc_generator(day13)]
pub fn input_generator(input: &str) -> Program {
    util::init();
    input.parse().unwrap()
}

#[aoc(day13, part1)]
pub fn solve_part1(program: &Program) -> usize {
    let mut arcade = Arcade::new(program.clone());

    info!("Waiting for screen");
    arcade.load_screen().expect("Arcade failed");
    info!("Number of blocks: {}", arcade.screen.num_blocks);

    arcade.screen.num_blocks
}

#[aoc(day13, part2)]
pub fn solve_part2(program: &Program) -> i64 {
    let mut arcade = Arcade::new(program.clone());

    info!("Waiting for screen");
    arcade.load_screen().expect("Arcade failed");

    let mut i = 0;

    let score = loop {
        if i % 100 == 0 {
            info!("Progress: blocks={}, score={}", arcade.screen.num_blocks, arcade.screen.score);
        }

        match arcade.step() {
            Err(Error::Intcode(IntcodeError::Halted)) => {
                break arcade.screen.score;
            },
            Err(_) => panic!("Arcade failed"),
            _ => {},
        }

        arcade.autopilot().expect("Autopilot failed");

        i += 1;
    };

    info!("Score: {}", score);

    score
}
