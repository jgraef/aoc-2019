use std::convert::{TryFrom, TryInto};
use std::fmt;

use aoc_runner_derive::{aoc, aoc_generator};
use failure::Fail;

use crate::intcode::{Program, Machine, Error as IntcodeError};
use std::collections::HashMap;
use std::cmp::Ordering;
use itertools::Itertools;
use core::fmt::Write;


#[derive(Clone, Debug, Fail)]
pub enum Error {
    #[fail(display = "Intcode error: {}", _0)]
    Intcode(#[cause] IntcodeError),
    #[fail(display = "Invalid color value: {}", _0)]
    InvalidColor(i64),
    #[fail(display = "Invalid direction value: {}", _0)]
    InvalidDirection(i64),
    #[fail(display = "Incomplete instruction")]
    IncompleteInstruction,
}

impl From<IntcodeError> for Error {
    fn from(e: IntcodeError) -> Self {
        Self::Intcode(e)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    Black,
    White
}

impl TryFrom<i64> for Color {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Black),
            1 => Ok(Self::White),
            _ => Err(Error::InvalidColor(value))
        }
    }
}

impl From<Color> for i64 {
    fn from(color: Color) -> Self {
        match color {
            Color::Black => 0,
            Color::White => 1,
        }
    }
}

impl From<Color> for char {
    fn from(color: Color) -> Self {
        match color {
            Color::White => '█',
            Color::Black => ' ',
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::Black
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RelativeDirection {
    Left,
    Right,
}

impl TryFrom<i64> for RelativeDirection {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Left),
            1 => Ok(Self::Right),
            _ => Err(Error::InvalidDirection(value))
        }
    }
}

#[derive(Clone, Debug)]
pub struct Instruction {
    color: Color,
    direction: RelativeDirection,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    x: i64,
    y: i64,
}

impl Position {
    pub fn new(x: i64, y: i64) -> Self {
        Self {
            x,
            y
        }
    }

    pub fn go(&mut self, direction: &AbsoluteDirection) {
        match direction {
            AbsoluteDirection::North => self.y -= 1,
            AbsoluteDirection::East => self.x += 1,
            AbsoluteDirection::South => self.y += 1,
            AbsoluteDirection::West => self.x -= 1,
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Position {
            x: 0,
            y: 0,
        }
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        self.y.cmp(&other.y)
            .then_with(|| self.x.cmp(&other.x))
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AbsoluteDirection {
    North,
    East,
    South,
    West,
}

impl AbsoluteDirection {
    pub fn turned(&self, by: &RelativeDirection) -> AbsoluteDirection {
        match by {
            RelativeDirection::Left => {
                match self {
                    Self::North => Self::West,
                    Self::East => Self::North,
                    Self::South => Self::East,
                    Self::West => Self::South,
                }
            },
            RelativeDirection::Right => {
                match self {
                    Self::North => Self::East,
                    Self::East => Self::South,
                    Self::South => Self::West,
                    Self::West => Self::North,
                }
            }
        }
    }

    pub fn turn(&mut self, by: &RelativeDirection) {
        *self = self.turned(by);
    }
}

impl Default for AbsoluteDirection {
    fn default() -> Self {
        AbsoluteDirection::North
    }
}

#[derive(Clone, Debug, Default)]
pub struct Hull {
    painted: HashMap<Position, Color>,
}

impl Hull {
    pub fn paint(&mut self, position: &Position, color: Color) {
        self.painted.insert(position.clone(), color);
    }

    pub fn get_color(&self, position: &Position) -> Color {
        self.painted.get(position)
            .copied()
            .unwrap_or_default()
    }

    pub fn num_painted(&self) -> usize {
        self.painted.len()
    }
}

impl fmt::Display for Hull {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let minmax = self.painted.keys().minmax();
        if let Some((min, max)) = minmax.into_option() {
            for y in min.y ..= max.y {
                for x in min.x ..= max.x {
                    let color = self.get_color(&Position::new(x, y));
                    f.write_char(color.into())?;
                }
                f.write_char('\n')?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Robot {
    machine: Machine,
    direction: AbsoluteDirection,
    position: Position,
}

impl Robot {
    pub fn new(program: Program) -> Self {
        Self {
            machine: Machine::new(program),
            direction: AbsoluteDirection::default(),
            position: Position::default()
        }
    }

    fn next_instruction(&mut self, color: Color) -> Result<Option<Instruction>, Error> {
        self.machine.push_input(color.into());
        match (self.machine.next_output()?, self.machine.next_output()?) {
            (Some(output1), Some(output2)) => {
                Ok(Some(Instruction {
                    color: output1.try_into()?,
                    direction: output2.try_into()?,
                }))
            },
            (None, None) => Ok(None),
            _ => Err(Error::IncompleteInstruction),
        }
    }

    pub fn paint_hull(&mut self, hull: &mut Hull) -> Result<(), Error> {
        while let Some(instruction) = self.next_instruction(hull.get_color(&self.position))? {
            println!("Position: {:?}", self.position);
            println!("Instruction: {:?}", instruction);

            hull.paint(&self.position, instruction.color);
            self.direction.turn(&instruction.direction);
            self.position.go(&self.direction)
        }

        Ok(())
    }
}


#[aoc_generator(day11)]
pub fn input_generator(input: &str) -> Program {
    input.parse().unwrap()
}

#[aoc(day11, part1)]
pub fn solve_part1(program: &Program) -> usize {
    let mut hull = Hull::default();
    let mut robot = Robot::new(program.clone());

    robot.paint_hull(&mut hull).expect("Robot failed");

    hull.num_painted()
}

#[aoc(day11, part2)]
pub fn solve_part2(program: &Program) -> Option<u32> {
    let mut hull = Hull::default();
    hull.paint(&Position::default(), Color::White);

    let mut robot = Robot::new(program.clone());

    robot.paint_hull(&mut hull).expect("Robot failed");

    println!("Hull:\n{}", hull);

    None
}

