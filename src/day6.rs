use std::str::FromStr;
use std::collections::HashMap;
use std::rc::Rc;

use aoc_runner_derive::{aoc, aoc_generator};
use failure::Fail;


#[derive(Clone, Debug, Fail)]
pub enum Error {
    #[fail(display = "Parse error")]
    ParseError,
}

#[derive(Clone, Debug)]
pub struct OrbitMap {
    orbits: HashMap<String, Rc<Orbit>>,
    satellites: HashMap<String, Vec<Rc<Orbit>>>,
}

impl OrbitMap {
    pub fn compute_checksum(&self) -> usize {
        self.orbits.values()
            .map(|orbit| self.compute_checksum_for_orbit(Rc::clone(&orbit)))
            .sum()
    }

    pub fn compute_checksum_for_orbit(&self, mut orbit: Rc<Orbit>) -> usize {
        let mut orbits = 1;
        while let Some(o) = self.orbits.get(&orbit.around) {
            orbit = Rc::clone(&o);
            orbits += 1;
        }
        orbits
    }

    pub fn compute_path_to_com(&self, from: &str) -> Vec<String> {
        let mut path = Vec::new();
        let mut current = from.to_owned();

        while let Some(o) = self.orbits.get(&current) {
            path.push(current);
            current = o.around.to_owned();
        }

        path
    }

    pub fn compute_path(&self, from: &str, to: &str) -> Vec<String> {
        let mut com_path_from = self.compute_path_to_com(from);
        let mut com_path_to = self.compute_path_to_com(to);

        let mut common_postfix = 0;
        for (a, b) in com_path_from.iter().rev().zip(com_path_to.iter().rev()) {
            if a != b {
                break;
            }
            common_postfix += 1;
        }

        com_path_from.resize_with(com_path_from.len() - common_postfix, || unreachable!());
        com_path_to.resize_with(com_path_to.len() - common_postfix, || unreachable!());

        com_path_to.reverse();
        com_path_from.append(&mut com_path_to);
        com_path_from
    }
}

impl FromStr for OrbitMap {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut orbits = HashMap::new();
        let mut satellites = HashMap::new();

        for line in s.lines() {
            let orbit = Rc::new(line.parse::<Orbit>()?);
            orbits.insert(orbit.object.clone(), Rc::clone(&orbit));
            satellites.entry(orbit.around.clone())
                .or_insert_with(Vec::new)
                .push(orbit)
        }

        Ok(OrbitMap {
            orbits,
            satellites,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Orbit {
    object: String,
    around: String,
}

impl FromStr for Orbit {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(")").collect::<Vec<&str>>();
        if parts.len() != 2 {
            return Err(Error::ParseError);
        }
        Ok(Orbit {
            object: parts[1].to_owned(),
            around: parts[0].to_owned(),
        })
    }
}

#[aoc_generator(day6)]
pub fn input_generator(input: &str) -> OrbitMap {
    input.parse().unwrap()
}

#[aoc(day6, part1)]
pub fn solve_part1(map: &OrbitMap) -> usize {
    let checksum = map.compute_checksum();
    println!("Checksum: {}", checksum);

    checksum
}

#[aoc(day6, part2)]
pub fn solve_part2(map: &OrbitMap) -> usize {
    let path = map.compute_path("YOU", "SAN");

    println!("Path:");
    for transfer in &path {
        println!("{}", transfer);
    }

    path.len() - 2
}
