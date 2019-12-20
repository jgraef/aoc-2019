use std::str::FromStr;

use failure::Fail;
use std::convert::{TryFrom, TryInto};
use std::collections::VecDeque;


#[derive(Debug, Clone, Fail)]
pub enum Error {
    #[fail(display = "Invalid opcode: {}", _0)]
    InvalidInstruction(i64),
    #[fail(display = "Invalid address: {}", _0)]
    InvalidAddress(usize),
    #[fail(display = "Machine is halted")]
    Halted,
    #[fail(display = "Invalid program")]
    InvalidProgram,
    #[fail(display = "Invalid parameter mode: {}", _0)]
    InvalidParameterMode(u8),
    #[fail(display = "No input available")]
    NoInput,
    #[fail(display = "Invalid argument: {}", _0)]
    InvalidArgument(i64)
}

pub enum ParameterMode {
    Position,
    Immediate,
}

impl TryFrom<u8> for ParameterMode {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Position),
            1 => Ok(Self::Immediate),
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
        }
    }

    pub fn push_input(&mut self, value: i64) {
        self.input.push_back(value);
    }

    pub fn pop_output(&mut self) -> Option<i64> {
        self.output.pop_front()
    }

    pub fn get_output(&mut self) -> Vec<i64> {
        self.output.drain(..).collect()
    }

    pub fn get_data(&self, address: usize) -> Result<i64, Error> {
        let data = *self.memory.get(address)
            .ok_or_else(|| Error::InvalidAddress(address))?;
        // println!("Get {}, is {}", address, data);
        Ok(data)
    }

    pub fn set_data(&mut self, address: usize, value: i64) -> Result<(), Error> {
        let opcode = self.memory.get_mut(address)
            .ok_or_else(|| Error::InvalidAddress(address))?;
        // println!("Set {} to {}", address, value);
        *opcode = value;
        Ok(())
    }

    fn get_arg(&self, arg_num: usize, opcode: i64) -> Result<i64, Error> {
        let arg = self.get_data(self.pc + 1 + arg_num)?;
        Ok(match Self::get_param_mode(opcode, arg_num)? {
            ParameterMode::Position => self.get_data(arg as usize)?,
            ParameterMode::Immediate => arg
        })
    }

    fn get_param_mode(mut opcode: i64, arg: usize) -> Result<ParameterMode, Error> {
        opcode /= 100;
        for _ in 0 .. arg {
            opcode /= 10;
        }
        ParameterMode::try_from((opcode % 10) as u8)
    }

    fn set_return(&mut self, arg: usize, value: i64) -> Result<(), Error> {
        let ptr = self.get_data(self.pc + 1 + arg)? as usize;
        self.set_data(ptr, value)?;
        Ok(())
    }

    fn bin_op<F: FnOnce(i64, i64) -> i64>(&mut self, op: F, opcode: i64) -> Result<(), Error> {
        let r = op(self.get_arg(0, opcode)?, self.get_arg(1, opcode)?);
        self.set_return(2, r)?;
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

        let opcode = *self.memory.get(self.pc)
            .ok_or_else(|| Error::InvalidAddress(self.pc))?;

        //println!("Executing {:?}", opcode);
        match opcode % 100 {
            1 => self.bin_op(|a, b| a + b, opcode)?,
            2 => self.bin_op(|a, b| a * b, opcode)?,
            3 => {
                let input = self.input.pop_front()
                    .ok_or(Error::NoInput)?;
                self.set_return(0, input)?;
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
}

#[derive(Clone, Debug)]
pub struct Program(Vec<i64>);

impl FromStr for Program {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let program = s.split(",")
            .map(|num| {
                num.parse::<i64>()
                    .map_err(|_| Error::InvalidProgram)
            })
            .collect::<Result<Vec<i64>, Error>>()?;
        Ok(Self(program))
    }
}
