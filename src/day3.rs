use std::str::FromStr;

use aoc_runner_derive::{aoc, aoc_generator};
use failure::Fail;
use itertools::Itertools;


#[derive(Clone, Debug, Fail)]
pub enum WireError {
    #[fail(display = "Failed to parse wire description")]
    ParseError,
}

#[derive(Clone, Debug)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down
}

#[derive(Clone, Debug)]
pub struct WireSegmentDescriptor {
    direction: Direction,
    length: u64,
}

impl FromStr for WireSegmentDescriptor {
    type Err = WireError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let first = s.chars().next()
            .ok_or_else(|| WireError::ParseError)?;
        let direction = match first {
            'L' => Direction::Left,
            'R' => Direction::Right,
            'U' => Direction::Up,
            'D' => Direction::Down,
            _ => return Err(WireError::ParseError)
        };

        let length = s[1..].parse::<u64>()
            .map_err(|_| WireError::ParseError)?;

        Ok(Self {
            direction,
            length,
        })
    }
}

type Position = (i64, i64);

#[derive(Clone, Debug)]
pub struct WireSegment {
    pub direction: Direction,
    pub length: u64,
    pub start: Position,
    pub total_length: u64,
}

impl WireSegment {
    pub fn endpoint(&self) -> Position {
        let l = self.length as i64;

        let (x, y) = match self.direction {
            Direction::Left => (-l, 0),
            Direction::Right => (l, 0),
            Direction::Up => (0, -l),
            Direction::Down => (0, l),
        };

        (self.start.0 + x, self.start.1 + y)
    }

    fn match_horizontal(&self, x: i64) -> bool {
        let l = self.length as i64;
        let mut a = self.start.0;
        let mut b = a;
        match self.direction {
            Direction::Left => a -= l,
            Direction::Right => b += l,
            _ => return false,
        }
        a <= x && x <= b
    }

    fn match_vertical(&self, x: i64) -> bool {
        let l = self.length as i64;
        let mut a = self.start.1;
        let mut b = a;
        match self.direction {
            Direction::Up => a -= l,
            Direction::Down => b += l,
            _ => return false,
        }
        a <= x && x <= b
    }

    pub fn intersects(&self, other: &Self) -> Option<Position> {
        if self.match_horizontal(other.start.0)
            && other.match_vertical(self.start.1) {
            Some((other.start.0, self.start.1))
        }
        else if self.match_vertical(other.start.1)
            && other.match_horizontal(self.start.0) {
            Some((self.start.0, other.start.1))
        }
        else {
            None
        }
    }

    pub fn length_for_point(&self, p: Position) -> u64 {
        let relative = match self.direction {
            Direction::Left => self.start.0 - p.0,
            Direction::Right => p.0 - self.start.0,
            Direction::Up => self.start.1 - p.1,
            Direction::Down => p.1 - self.start.1,
        } as u64;

        self.total_length - self.length + relative
    }
}


#[derive(Clone, Debug)]
pub struct Wire {
    segments: Vec<WireSegment>,
}

impl FromStr for Wire {
    type Err = WireError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut current_position = (0, 0);
        let mut total_length = 0;

        let segments = s.split(",")
            .map(|seg| {
                let desc = seg.parse::<WireSegmentDescriptor>()?;

                total_length += desc.length;

                let segment = WireSegment {
                    direction: desc.direction,
                    length: desc.length,
                    start: current_position,
                    total_length,
                };

                current_position = segment.endpoint();

                Ok(segment)
            })
            .collect::<Result<Vec<WireSegment>, WireError>>()?;

        Ok(Wire {
            segments,
        })
    }
}



#[aoc_generator(day3)]
pub fn input_generator(input: &str) -> Vec<Wire> {
    input.lines()
        .map(|line| line.parse::<Wire>())
        .collect::<Result<Vec<Wire>, WireError>>()
        .unwrap()
}

#[aoc(day3, part1)]
pub fn solve_part1(wires: &[Wire]) -> u64 {
    assert_eq!(wires.len(), 2);
    println!("{:#?}", wires);

    let mut distance = None;

    for (segment_a, segment_b) in wires[0].segments.iter().cartesian_product(wires[1].segments.iter()) {
        if let Some(intersection) = segment_a.intersects(segment_b) {
            let new_distance = (intersection.0.abs() + intersection.1.abs()) as u64;

            println!("Intersection: {:?} (distance {})", intersection, new_distance);

            if let Some(old_distance) = distance {
                if new_distance < old_distance {
                    distance = Some(new_distance);
                }
            }
            else {
                distance = Some(new_distance);
            }
        }
    }

    distance.unwrap()
}

#[aoc(day3, part2)]
pub fn solve_part2(wires: &[Wire]) -> u64 {
    assert_eq!(wires.len(), 2);
    println!("{:#?}", wires);

    let mut length = None;

    for (segment_a, segment_b) in wires[0].segments.iter().cartesian_product(wires[1].segments.iter()) {
        if let Some(intersection) = segment_a.intersects(segment_b) {
            let new_length = segment_a.length_for_point(intersection) + segment_b.length_for_point(intersection);

            println!("New length: {}", new_length);

            if let Some(old_length) = length {
                if new_length < old_length {
                    length = Some(new_length);
                }
            }
            else {
                length = Some(new_length);
            }
        }
    }

    length.unwrap()
}
