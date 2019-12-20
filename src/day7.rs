use std::ops::Range;

use aoc_runner_derive::{aoc, aoc_generator};
use itertools::Itertools;

use crate::intcode::{Program, Machine, Error};

type PhaseSettings = [u8; 5];

struct Circuit<'p> {
    program: &'p Program,
}

impl<'p> Circuit<'p> {
    pub fn new(program: &'p Program) -> Self {
        Self {
            program
        }
    }

    pub fn run_amplifier(&self, amplifier: &mut Machine, signal: i64) -> Result<Option<i64>, Error> {
        amplifier.push_input(signal);
        Ok(loop {
            if amplifier.is_halted() {
                break None;
            }
            amplifier.step()?;
            if let Some(output) = amplifier.pop_output() {
                break Some(output)
            }
        })
    }

    pub fn run_circuit(&self, phase_settings: &PhaseSettings, loopback: bool) -> Result<i64, Error> {
        let mut amplifiers = [
            Machine::new(self.program.clone()),
            Machine::new(self.program.clone()),
            Machine::new(self.program.clone()),
            Machine::new(self.program.clone()),
            Machine::new(self.program.clone()),
        ];
        let mut signal = 0;
        let mut done = false;

        for i in 0 .. 5 {
            amplifiers[i].push_input(phase_settings[i] as i64);
        }

        while !done {
            for i in 0 .. 5 {
                if let Some(output) = self.run_amplifier(&mut amplifiers[i], signal)? {
                    println!("Amplifier #{}: input={}, output={}", i, signal, output);
                    signal = output;
                }
                else {
                    println!("Amplifier #{} halted", i);
                    done = true;
                }
            }
            if !loopback {
                done = true;
            }
        }

        Ok(signal)
    }
}

#[aoc_generator(day7)]
pub fn input_generator(input: &str) -> Program {
    input.parse().unwrap()
}

pub fn try_phase_settings(program: &Program, phase_settings_range: Range<u8>, loopback: bool) -> i64 {
    let circuit = Circuit::new(program);
    let mut best_output = 0;

    for perm in phase_settings_range.permutations(5) {
        let mut phase_settings: PhaseSettings = [0; 5];
        assert_eq!(phase_settings.len(), 5);
        phase_settings.copy_from_slice(&perm);

        println!("Trying phase settings {:?}", phase_settings);
        let output = circuit.run_circuit(&phase_settings, loopback).expect("Circuit failed");
        println!("Circuit output: {}", output);
        if output > best_output {
            best_output = output;
        }
        println!();
    }

    println!("Best thruster output: {}", best_output);

    best_output
}

#[aoc(day7, part1)]
pub fn solve_part1(program: &Program) -> i64 {
    try_phase_settings(program, 0 .. 5, false)
}

#[aoc(day7, part2)]
pub fn solve_part2(program: &Program) -> i64 {
    try_phase_settings(program, 5 .. 10, true)
}
