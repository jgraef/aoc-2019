use std::hash::{Hash, Hasher};
use std::collections::HashSet;

use regex::Regex;
use nalgebra::Vector3;
use num_traits::Zero;
use num::integer::lcm;
use itertools::Itertools;

use aoc_runner_derive::{aoc, aoc_generator};


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Body {
    position: Vector3<i64>,
    velocity: Vector3<i64>,
}

impl Body {
    pub fn new(position: Vector3<i64>) -> Self {
        Self {
            position,
            velocity: Vector3::zero(),
        }
    }

    pub fn acceleration_towards(&self, other: &Self) -> Vector3<i64> {
        let d = other.position - self.position;
        Vector3::new(
            d.x.signum(),
            d.y.signum(),
            d.z.signum()
        )
    }

    pub fn potential_energy(&self) -> i64 {
        self.position.x.abs() + self.position.y.abs() + self.position.z.abs()
    }

    pub fn kinetic_energy(&self) -> i64 {
        self.velocity.x.abs() + self.velocity.y.abs() + self.velocity.z.abs()
    }

    pub fn energy(&self) -> i64 {
        self.potential_energy() * self.kinetic_energy()
    }
}

#[derive(Clone, Debug)]
pub struct DimensionalState {
    positions: Vec<i64>,
    velocities: Vec<i64>,
    step: usize,
}

impl PartialEq for DimensionalState {
    fn eq(&self, other: &DimensionalState) -> bool {
        self.positions == other.positions && self.velocities == other.velocities
    }
}

impl Eq for DimensionalState {}

impl Hash for DimensionalState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.positions.hash(state);
        self.velocities.hash(state);
    }
}

impl PartialEq<System> for DimensionalState {
    fn eq(&self, other: &System) -> bool {
        self.positions.iter()
            .zip(&other.bodies)
            .all(|(a, b)| *a == b.position.x)
    }
}

#[derive(Clone, Debug)]
pub struct SplitDimensions {
    x: DimensionalState,
    y: DimensionalState,
    z: DimensionalState,
}

#[derive(Clone, Debug, Default)]
pub struct System {
    bodies: Vec<Body>,
    step: usize,
}

impl System {
    pub fn add_body(&mut self, body: Body) {
        self.bodies.push(body);
    }

    pub fn step(&mut self) {
        let mut accelerations: Vec<Vector3<i64>> = Vec::with_capacity(self.bodies.len());
        accelerations.resize_with(self.bodies.len(), Vector3::zero);

        for ((i, body_i), (j, body_j)) in self.bodies.iter().enumerate().tuple_combinations() {
            let acceleration = body_i.acceleration_towards(body_j);
            *accelerations.get_mut(i).unwrap() += acceleration;
            *accelerations.get_mut(j).unwrap() -= acceleration;
        }

        for (body, acceleration) in self.bodies.iter_mut().zip(&accelerations) {
            body.velocity += acceleration;
        }

        for body in &mut self.bodies {
            body.position += body.velocity;
        }

        self.step += 1;
    }

    pub fn energy(&self) -> i64 {
        self.bodies.iter()
            .map(|body| body.energy())
            .sum()
    }

    pub fn dimensions(&self) -> SplitDimensions {
        SplitDimensions {
            x: DimensionalState {
                positions: self.bodies.iter()
                    .map(|body| body.position.x)
                    .collect_vec(),
                velocities: self.bodies.iter()
                    .map(|body| body.velocity.x)
                    .collect_vec(),
                step: self.step,
            },
            y: DimensionalState {
                positions: self.bodies.iter()
                    .map(|body| body.position.y)
                    .collect_vec(),
                velocities: self.bodies.iter()
                    .map(|body| body.velocity.y)
                    .collect_vec(),
                step: self.step,
            },
            z: DimensionalState {
                positions: self.bodies.iter()
                    .map(|body| body.position.z)
                    .collect_vec(),
                velocities: self.bodies.iter()
                    .map(|body| body.velocity.z)
                    .collect_vec(),
                step: self.step,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Cycle {
    x0: DimensionalState,
    x1: DimensionalState,
    n: usize,
}

impl Cycle {
    pub fn new(x0: DimensionalState, x1: DimensionalState) -> Self {
        let n = x1.step - x0.step;
        Self {
            x0,
            x1,
            n
        }
    }
}

#[derive(Clone, Debug)]
pub struct Cycles {
    x: Cycle,
    y: Cycle,
    z: Cycle,
}

impl Cycles {
    pub fn length(&self) -> usize {
        lcm(self.x.n, lcm(self.y.n, self.z.n))
    }
}

#[derive(Clone, Debug, Default)]
pub struct History {
    cycle_x: Option<Cycle>,
    cycle_y: Option<Cycle>,
    cycle_z: Option<Cycle>,
    x: HashSet<DimensionalState>,
    y: HashSet<DimensionalState>,
    z: HashSet<DimensionalState>,
}

impl History {
    pub fn insert(&mut self, system: &System) {
        let SplitDimensions { x, y, z } = system.dimensions();

        if let Some(x0) = self.x.get(&x) {
            if self.cycle_x.is_none() {
                let x = Some(Cycle::new(x0.clone(), x.clone()));
                self.cycle_x = x;
            }
        }
        if let Some(y0) = self.y.get(&y) {
            if self.cycle_y.is_none() {
                let y = Some(Cycle::new(y0.clone(), y.clone()));
                self.cycle_y = y;
            }
        }
        if let Some(z0) = self.z.get(&z) {
            if self.cycle_z.is_none() {
                let z = Some(Cycle::new(z0.clone(), z.clone()));
                self.cycle_z = z;
            }
        }

        self.x.insert(x);
        self.y.insert(y);
        self.z.insert(z);
    }

    pub fn get_complete_cycles(&self) -> Option<Cycles> {
        match (&self.cycle_x, &self.cycle_y, &self.cycle_z) {
            (Some(x), Some(y), Some(z)) => {
                Some(Cycles {
                    x: x.clone(),
                    y: y.clone(),
                    z: z.clone(),
                })
            },
            _ => None,
        }
    }
}

#[aoc_generator(day12)]
pub fn input_generator(input: &str) -> System {
    let re = Regex::new(r"<x=([-+]?\d+), y=([-+]?\d+), z=([-+]?\d+)>").unwrap();

    let mut system = System::default();

    for capture in re.captures_iter(input) {
        let position = Vector3::new(
            capture.get(1).unwrap().as_str().parse::<i64>().unwrap(),
            capture.get(2).unwrap().as_str().parse::<i64>().unwrap(),
            capture.get(3).unwrap().as_str().parse::<i64>().unwrap(),
        );
        system.add_body(Body::new(position));
    }

    system
}

fn report_system(system: &System, interval: usize) {
    if system.step % interval == 0 {
        println!("[{:.2} %] After {} steps:", (system.step as f64) * 100.0 / 4686774924.0, system.step);
        println!("Energy: {}", system.energy());
        for body in &system.bodies {
            println!(
                "pos=<{:>3}, {:>3}, {:>3}>, vel=<{:>3}, {:>3}, {:>3}>, potential={:?}, kinetic={:?}",
                body.position.x,
                body.position.y,
                body.position.z,
                body.velocity.x,
                body.velocity.y,
                body.velocity.z,
                body.potential_energy(),
                body.kinetic_energy()
            );
        }
        println!();
    }
}

#[aoc(day12, part1)]
pub fn solve_part1(system: &System) -> i64 {
    let mut system = system.clone();

    println!("System {:#?}", system);

    for _ in 0 .. 1000 {
        report_system(&system, 100);
        system.step();
    }

    system.energy()
}

#[aoc(day12, part2)]
pub fn solve_part2(initial_state: &System) -> usize {
    let mut system = initial_state.clone();
    let mut history = History::default();

    loop {
        report_system(&system, 1000000);

        history.insert(&system);

        if let Some(cycles) = history.get_complete_cycles() {
            println!("Found complete cycle: {:#?}", cycles);
            println!("X cycle: {}", cycles.x.n);
            println!("Y cycle: {}", cycles.y.n);
            println!("Z cycle: {}", cycles.z.n);
            let length = cycles.length();
            println!("Length: {}", length);
            break length;
        }

        system.step();
    }
}
