use aoc_runner_derive::{aoc, aoc_generator};
use std::ops::RangeInclusive;


#[aoc_generator(day4)]
pub fn input_generator(input: &str) -> RangeInclusive<u64> {
    let parts = input.split("-")
        .map(|s| s.parse().unwrap())
        .collect::<Vec<u64>>();

    RangeInclusive::new(parts[0], parts[1])
}

fn to_radix(mut x: u64) -> [u8; 6] {
    let mut radix = [0; 6];

    for i in 0 .. 6 {
        radix[5 - i] = (x % 10) as u8;
        x /= 10;
    }

    radix
}

#[aoc(day4, part1)]
pub fn solve_part1(range: &RangeInclusive<u64>) -> u64 {
    println!("Range: {} - {}", range.start(), range.end());

    let mut num_matches = 0;

    for num in range.clone() {
        let radix = to_radix(num);

        let mut found_repeating = false;
        let mut is_increasing = true;
        let mut repetitions = 0;

        for i in 1..6 {
            if radix[i - 1] == radix[i] {
                repetitions += 1;
            }
            else {
                if repetitions == 1 {
                    found_repeating = true;
                }
                repetitions = 0;
            }

            if radix[i - 1] > radix[i] {
                is_increasing = false;
            }
        }
        if repetitions == 1 {
            found_repeating = true;
        }

        
        if !found_repeating || !is_increasing {
            continue;
        }

        println!("Found match: {}", num);

        num_matches += 1;
    }

    num_matches
}
