use aoc_runner_derive::{aoc, aoc_generator};

use crate::intcode::{Program, Machine};


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
