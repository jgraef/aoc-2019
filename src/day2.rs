use std::str::FromStr;

use aoc_runner_derive::{aoc, aoc_generator};
use failure::Fail;

#[derive(Debug, Clone, Fail)]
pub enum MachineError {
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

    fn get_arg(&self, arg: usize) -> Result<u64, MachineError> {
        let ptr = self.get_data(self.pc + 1 + arg)? as usize;
        self.get_data(ptr)
    }

    fn bin_op<F: FnOnce(u64, u64) -> u64>(&mut self, op: F) -> Result<(), MachineError> {
        let r = op(self.get_arg(0)?, self.get_arg(1)?);
        self.set_data(self.get_data(self.pc + 3)? as usize, r)?;
        self.pc += 4;
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), MachineError> {
        if self.halted {
            return Err(MachineError::Halted)
        }

        let opcode = *self.memory.get(self.pc)
            .ok_or_else(|| MachineError::InvalidAddress(self.pc))?;

        // println!("Executing {:?}", opcode);
        match opcode {
            1 => self.bin_op(|a, b| a + b)?,
            2 => self.bin_op(|a, b| a * b)?,
            99 => self.halted = true,
            data => return Err(MachineError::InvalidInstruction(data)),
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), MachineError> {
        while !self.halted {
            self.step()?;
        }
        Ok(())
    }

    pub fn get_data(&self, address: usize) -> Result<u64, MachineError> {
        let data = *self.memory.get(address)
            .ok_or_else(|| MachineError::InvalidAddress(address))?;
        // println!("Get {}, is {}", address, data);
        Ok(data)
    }

    pub fn set_data(&mut self, address: usize, value: u64) -> Result<(), MachineError> {
        let opcode = self.memory.get_mut(address)
            .ok_or_else(|| MachineError::InvalidAddress(address))?;
        // println!("Set {} to {}", address, value);
        *opcode = value;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Program(Vec<u64>);

impl FromStr for Program {
    type Err = MachineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let program = s.split(",")
            .map(|num| {
                num.parse::<u64>()
                    .map_err(|_| MachineError::InvalidProgram)
            })
            .collect::<Result<Vec<u64>, MachineError>>()?;
        Ok(Self(program))
    }
}



#[aoc_generator(day2)]
pub fn input_generator(input: &str) -> Program {
    input.parse().unwrap()
}

#[aoc(day2, part1)]
pub fn solve_part1(program: &Program) -> u64 {
    let mut machine = Machine::new(program.clone());

    machine.set_data(1, 12).unwrap();
    machine.set_data(2, 2).unwrap();

    machine.run().unwrap();

    machine.get_data(0).unwrap()
}

#[aoc(day2, part2)]
pub fn solve_part2(program: &Program) -> u64 {
    for noun in 0 .. 100 {
        for verb in 0 .. 100 {
            let mut machine = Machine::new(program.clone());

            machine.set_data(1, noun).unwrap();
            machine.set_data(2, verb).unwrap();

            machine.run().unwrap();

            let result = machine.get_data(0).unwrap();
            if result == 19690720 {
                println!("Found result: {}, {}", noun, verb);
                return 100 * noun + verb
            }
        }
    }

    panic!("No inputs found.");
}
