use std::str::FromStr;

use failure::Fail;
use std::convert::{TryFrom, TryInto};
use std::collections::VecDeque;


#[derive(Debug, Clone, Fail)]
pub enum Error {
    #[fail(display = "Invalid opcode: {}", _0)]
    InvalidInstruction(i64),
    #[fail(display = "Invalid address: {}", _0)]
    InvalidAddress(i64),
    #[fail(display = "Machine is halted")]
    Halted,
    #[fail(display = "Invalid program")]
    InvalidProgram,
    #[fail(display = "Invalid parameter mode: {}", _0)]
    InvalidParameterMode(u8),
    #[fail(display = "No input available")]
    NoInput,
    #[fail(display = "Invalid argument: {}", _0)]
    InvalidArgument(i64),
    #[fail(display = "Not an integer: {}", _0)]
    NotAnInteger(String),
}

pub enum ParameterMode {
    Position,
    Immediate,
    Relative,
}

impl TryFrom<u8> for ParameterMode {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Position),
            1 => Ok(Self::Immediate),
            2 => Ok(Self::Relative),
            _ => Err(Error::InvalidParameterMode(value))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Machine {
    memory: Vec<i64>,
    pc: usize,
    halted: bool,
    input: VecDeque<i64>,
    output: VecDeque<i64>,
    relative_base: i64,
    constant_input: Option<i64>,
}

impl Machine {
    pub fn new(program: Program) -> Machine {
        //println!("Memory: {:?}", program);
        Self {
            memory: program.0,
            pc: 0,
            halted: false,
            input: VecDeque::new(),
            output: VecDeque::new(),
            relative_base: 0,
            constant_input: None,
        }
    }

    pub fn pc(&self) -> usize {
        self.pc
    }

    pub fn push_input(&mut self, value: i64) {
        self.input.push_back(value);
    }

    pub fn set_contant_input(&mut self, value: i64) {
        self.constant_input = Some(value);
    }

    pub fn pop_output(&mut self) -> Option<i64> {
        self.output.pop_front()
    }

    pub fn get_output(&mut self) -> Vec<i64> {
        self.output.drain(..).collect()
    }

    pub fn is_halted(&self) -> bool {
        self.halted
    }

    pub fn get_data(&self, address: usize) -> i64 {
        self.memory.get(address)
            .copied()
            .unwrap_or_default()
    }

    pub fn set_data(&mut self, address: usize, value: i64) {
        if self.memory.len() < address + 1 {
            self.memory.resize(address + 1, 0);
        }

        let ptr = self.memory.get_mut(address)
            .expect("Expected memory location");
        *ptr = value;
    }

    fn get_param_mode(mut opcode: i64, arg: usize) -> Result<ParameterMode, Error> {
        opcode /= 100;
        for _ in 0 .. arg {
            opcode /= 10;
        }
        ParameterMode::try_from((opcode % 10) as u8)
    }

    fn get_arg(&self, arg_num: usize, opcode: i64) -> Result<i64, Error> {
        let arg = self.get_data(self.pc + 1 + arg_num);
        Ok(match Self::get_param_mode(opcode, arg_num)? {
            ParameterMode::Position => {
                let address = arg.try_into()
                    .map_err(|_| Error::InvalidAddress(arg))?;
                self.get_data(address)
            },
            ParameterMode::Immediate => arg,
            ParameterMode::Relative => {
                let address = arg + self.relative_base;
                let address = address.try_into()
                    .map_err(|_| Error::InvalidAddress(address))?;
                self.get_data(address)
            },
        })
    }

    fn set_return(&mut self, arg_num: usize, value: i64, opcode: i64) -> Result<(), Error> {
        let arg = self.get_data(self.pc + 1 + arg_num);
        let address = match Self::get_param_mode(opcode, arg_num)? {
            ParameterMode::Position => arg,
            ParameterMode::Immediate => return Err(Error::InvalidInstruction(opcode)),
            ParameterMode::Relative => arg + self.relative_base,
        };
        let address = address.try_into().map_err(|_| Error::InvalidAddress(address))?;
        self.set_data(address, value);
        Ok(())
    }

    fn bin_op<F: FnOnce(i64, i64) -> i64>(&mut self, op: F, opcode: i64) -> Result<(), Error> {
        let r = op(self.get_arg(0, opcode)?, self.get_arg(1, opcode)?);
        self.set_return(2, r, opcode)?;
        self.pc += 4;
        Ok(())
    }

    fn jump_op(&mut self, cmp: bool, opcode: i64) -> Result<(), Error> {
        let arg = self.get_arg(0, opcode)?;
        if (arg != 0) == cmp {
            let arg = self.get_arg(1, opcode)?;
            self.pc = arg.try_into()
                .map_err(|_| Error::InvalidArgument(arg))?;
        }
        else {
            self.pc += 3;
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), Error> {
        if self.halted {
            return Err(Error::Halted)
        }

        let opcode = self.get_data(self.pc);

        //println!("Executing {:?}", opcode);
        match opcode % 100 {
            1 => self.bin_op(|a, b| a + b, opcode)?,
            2 => self.bin_op(|a, b| a * b, opcode)?,
            3 => {
                if let Some(input) = self.constant_input {
                    self.set_return(0, input, opcode)?;
                }
                else {
                    let input = self.input.pop_front()
                        .ok_or(Error::NoInput)?;
                    self.set_return(0, input, opcode)?;
                }

                self.pc += 2;
            },
            4 => {
                let output = self.get_arg(0, opcode)?;
                self.output.push_back(output);
                self.pc += 2;
            },
            5 => self.jump_op(true, opcode)?,
            6 => self.jump_op(false, opcode)?,
            7 => self.bin_op(|a, b| if a < b { 1 } else { 0 }, opcode)?,
            8 => self.bin_op(|a, b| if a == b { 1 } else { 0 }, opcode)?,
            9 => {
                self.relative_base += self.get_arg(0, opcode)?;
                self.pc += 2;
            }
            99 => self.halted = true,
            data => return Err(Error::InvalidInstruction(data)),
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Error> {
        while !self.halted {
            self.step()?;
        }
        Ok(())
    }

    pub fn next_output(&mut self) -> Result<Option<i64>, Error> {
        Ok(loop {
            if self.halted {
                return Err(Error::Halted);
            }

            self.step()?;

            if let Some(output) = self.pop_output() {
                break Some(output);
            }
        })
    }
}

#[derive(Clone, Debug)]
pub struct Program(Vec<i64>);

impl FromStr for Program {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let program = s.split(",")
            .map(|num| {
                num.trim().parse::<i64>()
                    .map_err(|_| Error::NotAnInteger(num.to_owned()))
            })
            .collect::<Result<Vec<i64>, Error>>()?;
        Ok(Self(program))
    }
}
