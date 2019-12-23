extern crate aoc_2019;

use std::fs::read_to_string;
use std::env;
use std::path::Path;


pub fn main() {
    aoc_2019::util::init();
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("input/2019/day13.txt");
    let program = read_to_string(path).unwrap().parse().unwrap();
    aoc_2019::arcade_game::solve(program, false);
}
