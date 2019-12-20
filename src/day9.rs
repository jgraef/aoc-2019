use aoc_runner_derive::{aoc, aoc_generator};

use crate::intcode::{Program, Machine};

#[aoc_generator(day9)]
pub fn input_generator(input: &str) -> Program {
    input.parse().unwrap()
}

#[aoc(day9, part1)]
pub fn solve_part1(program: &Program) -> i64 {
    let mut machine = Machine::new(program.clone());

    machine.push_input(1);

    machine.run().expect("Machine failed");

    let outputs = machine.get_output();

    if outputs.len() > 1 {
        println!("Some checks failed:");
        for (i, output) in outputs.iter().enumerate() {
            println!("Output #{}: {:?}", i, output);
        }
        0
    }
    else {
        *outputs.get(0).unwrap()
    }
}

#[aoc(day9, part2)]
pub fn solve_part2(program: &Program) -> i64 {
    let mut machine = Machine::new(program.clone());
    machine.push_input(2);
    machine.run().expect("Machine failed");
    machine.pop_output().unwrap()
}
