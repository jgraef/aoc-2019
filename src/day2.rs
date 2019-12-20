use aoc_runner_derive::{aoc, aoc_generator};

use crate::intcode::{Program, Machine};


#[aoc_generator(day2)]
pub fn input_generator(input: &str) -> Program {
    input.parse().unwrap()
}

#[aoc(day2, part1)]
pub fn solve_part1(program: &Program) -> i64 {
    let mut machine = Machine::new(program.clone());

    machine.set_data(1, 12);
    machine.set_data(2, 2);

    machine.run().unwrap();

    machine.get_data(0)
}

#[aoc(day2, part2)]
pub fn solve_part2(program: &Program) -> i64 {
    for noun in 0 .. 100 {
        for verb in 0 .. 100 {
            let mut machine = Machine::new(program.clone());

            machine.set_data(1, noun);
            machine.set_data(2, verb);

            machine.run().unwrap();

            let result = machine.get_data(0);
            if result == 19690720 {
                println!("Found result: {}, {}", noun, verb);
                return 100 * noun + verb
            }
        }
    }

    panic!("No inputs found.");
}
