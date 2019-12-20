use aoc_runner_derive::{aoc, aoc_generator};


fn fuel_required(mass: u64) -> u64 {
    (mass / 3).saturating_sub(2)
}

fn with_extra_fuel(fuel: u64) -> u64 {
    match fuel {
        0 => 0,
        fuel => fuel + with_extra_fuel(fuel_required(fuel))
    }
}


#[aoc_generator(day1)]
pub fn input_generator(input: &str) -> Vec<u64> {
    input.lines()
        .map(|line| line.parse::<u64>().unwrap())
        .collect::<Vec<u64>>()
}

#[aoc(day1, part1)]
pub fn solve_part1(input: &[u64]) -> u64 {
    input.iter()
        .map(|mass| fuel_required(*mass))
        .sum()
}

#[aoc(day1, part2)]
pub fn solve_part2(input: &[u64]) -> u64 {
    input.iter()
        .map(|mass| with_extra_fuel(fuel_required(*mass)))
        .sum()
}
