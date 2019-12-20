use std::str::FromStr;

use failure::Fail;


#[derive(Debug, Clone, Fail)]
pub enum Error {
    #[fail(display = "Invalid opcode: {}", _0)]
    InvalidInstruction(u64),
    #[fail(display = "Invalid address: {}", _0)]
    InvalidAddress(usize),
    #[fail(display = "Machine is halted")]
    Halted,
    #[fail(display = "Invalid program")]
    InvalidProgram,
}

#[derive(Debug, Clone)]
pub struct Machine {
    memory: Vec<u64>,
    pc: usize,
    halted: bool,
}

impl Machine {
    pub fn new(program: Program) -> Machine {
        //println!("Memory: {:?}", program);
        Self {
            memory: program.0,
            pc: 0,
            halted: false,
        }
    }

    fn get_arg(&self, arg: usize) -> Result<u64, Error> {
        let ptr = self.get_data(self.pc + 1 + arg)? as usize;
        self.get_data(ptr)
    }

    fn bin_op<F: FnOnce(u64, u64) -> u64>(&mut self, op: F) -> Result<(), Error> {
        let r = op(self.get_arg(0)?, self.get_arg(1)?);
        self.set_data(self.get_data(self.pc + 3)? as usize, r)?;
        self.pc += 4;
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), Error> {
        if self.halted {
            return Err(Error::Halted)
        }

        let opcode = *self.memory.get(self.pc)
            .ok_or_else(|| Error::InvalidAddress(self.pc))?;

        // println!("Executing {:?}", opcode);
        match opcode {
            1 => self.bin_op(|a, b| a + b)?,
            2 => self.bin_op(|a, b| a * b)?,
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

    pub fn get_data(&self, address: usize) -> Result<u64, Error> {
        let data = *self.memory.get(address)
            .ok_or_else(|| Error::InvalidAddress(address))?;
        // println!("Get {}, is {}", address, data);
        Ok(data)
    }

    pub fn set_data(&mut self, address: usize, value: u64) -> Result<(), Error> {
        let opcode = self.memory.get_mut(address)
            .ok_or_else(|| Error::InvalidAddress(address))?;
        // println!("Set {} to {}", address, value);
        *opcode = value;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Program(Vec<u64>);

impl FromStr for Program {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let program = s.split(",")
            .map(|num| {
                num.parse::<u64>()
                    .map_err(|_| Error::InvalidProgram)
            })
            .collect::<Result<Vec<u64>, Error>>()?;
        Ok(Self(program))
    }
}
