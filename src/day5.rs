use aoc_runner_derive::{aoc, aoc_generator};

use crate::intcode::{Program, Machine};


#[aoc_generator(day5)]
pub fn input_generator(input: &str) -> Program {
    input.parse().unwrap()
}

#[aoc(day5, part1)]
pub fn solve_part1(program: &Program) -> i64 {
    let mut machine = Machine::new(program.clone());

    machine.push_input(1);

    machine.run().unwrap();

    let output = machine.get_output();
    let checks = &output[0 .. output.len() - 1];
    for (i, x) in checks.iter().enumerate() {
        println!("Check #{}: {}", i, x);
    }
    let diagnostic_code = output[output.len() - 1];
    println!("Diagnostic code: {}", diagnostic_code);

    assert!(checks.iter().all(|&x| x == 0));
    diagnostic_code
}

#[aoc(day5, part2)]
pub fn solve_part2(program: &Program) -> i64 {
    let mut machine = Machine::new(program.clone());

    machine.push_input(5);

    machine.run().unwrap();

    let diagnostic_code = machine.pop_output().expect("Expected diagnostics code");
    println!("Diagnostic code: {}", diagnostic_code);

    diagnostic_code
}