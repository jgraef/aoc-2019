use std::str::FromStr;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hasher, Hash};
use std::cmp::Ordering;

use aoc_runner_derive::{aoc, aoc_generator};
use failure::Fail;
use num::Integer;

use crate::util;


#[derive(Clone, Debug, Fail)]
pub enum ParseError {
    #[fail(display = "Empty map")]
    Empty,
    #[fail(display = "Invalid line: {}", _0)]
    InvalidLine(String),
}

#[derive(Clone, Debug)]
pub struct Ray {
    x0: i64,
    y0: i64,
    dx: i64,
    dy: i64,
}

impl Ray {
    pub fn new(from: &Asteroid, to: &Asteroid) -> Self {
        let mut dx = to.x - from.x;
        let mut dy = to.y - from.y;

        assert!(dx != 0 || dy != 0);

        if dx == 0 {
            dy = dy.signum();
        }
        else if dy == 0 {
            dx = dx.signum();
        }
        else {
            let k = dx.abs().gcd(&dy.abs());
            dx /= k;
            dy /= k;
        }

        Ray {
            x0: from.x,
            y0: from.y,
            dx,
            dy,
        }
    }

    fn angle_quadrant(&self) -> u8 {
        match (self.dx.signum(), self.dy.signum()) {
            (0, -1) => 0,
            (1, -1) => 1,
            (1, 0) => 2,
            (1, 1) => 3,
            (0, 1) => 4,
            (-1, 1) => 5,
            (-1, 0) => 6,
            (-1, -1) => 7,
            _ => unreachable!()
        }
    }
}

impl PartialEq for Ray {
    fn eq(&self, other: &Ray) -> bool {
        // Check that both rays have the same origin
        if self.x0 != other.x0 || self.y0 != other.y0 {
            return false;
        }

        // Check that both rays have the same direction. Direction is normalized.
        if self.dx != other.dx || self.dy != other.dy {
            return false;
        }

        true
    }
}

impl PartialOrd for Ray {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ray {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.angle_quadrant().cmp(&other.angle_quadrant()) {
            Ordering::Equal => {
                (other.dx * self.dy).cmp(&(self.dx * other.dy))
            },
            ordering => ordering,
        }
    }
}

impl Eq for Ray {}

impl Hash for Ray {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dx.hash(state);
        self.dy.hash(state);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asteroid {
    pub x: i64,
    pub y: i64,
}

pub struct AsteroidMap {
    asteroids: Vec<Asteroid>,
}

impl AsteroidMap {
    pub fn get_visible_asteroids(&self, asteroid: &Asteroid) -> HashMap<Ray, Vec<&Asteroid>> {
        let mut collisions = HashMap::new();

        for asteroid2 in &self.asteroids {
            if asteroid != asteroid2 {
                let ray = Ray::new(asteroid, asteroid2);
                collisions.entry(ray)
                    .or_insert_with(|| Vec::with_capacity(1))
                    .push(asteroid2)
            }
        }

        for (_, asteroids) in &mut collisions {
            asteroids.sort_by_key(|asteroid2| (asteroid2.x - asteroid.x).abs() + (asteroid2.y - asteroid.y).abs())
        }

        collisions
    }

    pub fn get_kill_order<'a>(&self, collisions: HashMap<Ray, Vec<&'a Asteroid>>) -> Vec<&'a Asteroid> {
        let mut collisions = collisions.into_iter()
            .map(|(ray, asteroids)| (ray, VecDeque::from(asteroids)))
            .collect::<HashMap<Ray, VecDeque<&'a Asteroid>>>();
        let mut rays: Vec<Ray> = collisions.keys()
            .into_iter()
            .cloned()
            .collect();
        rays.sort_unstable();
        let mut rays_cycle = rays.iter().cycle();
        let mut kills = Vec::new();

        while !collisions.is_empty() {
            let ray = rays_cycle.next().unwrap();

            if let Some(asteroids) = collisions.get_mut(ray) {
                if let Some(asteroid) = asteroids.pop_front() {
                    kills.push(asteroid);
                }
                else {
                    collisions.remove(ray);
                }
            }
        }

        kills
    }
}

impl FromStr for AsteroidMap {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines = s.lines().collect::<Vec<&str>>();
        let height = lines.len();
        if height == 0 {
            return Err(ParseError::Empty)
        }
        let width = lines[0].len();
        if width == 0 {
            return Err(ParseError::Empty)
        }

        let mut asteroids = Vec::new();
        for (y, line) in lines.into_iter().enumerate() {
            if line.len() != width {
                return Err(ParseError::InvalidLine(line.to_owned()))
            }
            for (x, c) in line.chars().enumerate() {
                if c == '#' {
                    asteroids.push(Asteroid {
                        x: x as i64,
                        y: y as i64,
                    });
                }
            }
        }

        Ok(AsteroidMap {
            asteroids
        })
    }
}


#[aoc_generator(day10)]
pub fn input_generator(input: &str) -> AsteroidMap {
    util::init();
    input.parse().unwrap()
}

fn get_best_asteroid(map: &AsteroidMap) -> Option<(&Asteroid, HashMap<Ray, Vec<&Asteroid>>)> {
    map.asteroids.iter()
        .map(|asteroid| {
            let collisions = map.get_visible_asteroids(asteroid);
            (asteroid, collisions)
        })
        .max_by_key(|(_asteroid, collisions)| collisions.len())
}

#[aoc(day10, part1)]
pub fn solve_part1(map: &AsteroidMap) -> usize {
    let (asteroid, collisions) = get_best_asteroid(map).unwrap();

    debug!("Best location: {:?}", asteroid);
    //debug!("Collisions: {:#?}", collisions);
    debug!("Visible asteroids: {}", collisions.len());

    collisions.len()
}

#[aoc(day10, part2)]
pub fn solve_part2(map: &AsteroidMap) -> i64 {
    let (laser_station, collisions) = get_best_asteroid(map).unwrap();

    let kills = map.get_kill_order(collisions);

    for (i, kill) in kills.iter().enumerate() {
        let dx = kill.x - laser_station.x;
        let dy = kill.y - laser_station.y;
        let a = (dx as f64).atan2(-dy as f64).to_degrees();
        debug!("Kill #{}: {:?} - {},{} {}", i + 1, kill, dx, dy, a);
    }

    let asteroid = kills.get(199).unwrap();
    debug!("200th asteroid: {:?}", asteroid);

    asteroid.x * 100 + asteroid.y
}
