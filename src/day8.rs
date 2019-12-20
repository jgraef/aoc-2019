use std::str::FromStr;
use std::convert::{TryFrom, TryInto};

use aoc_runner_derive::{aoc, aoc_generator};
use failure::Fail;


#[derive(Clone, Debug, Fail)]
pub enum ParseError {
    #[fail(display = "Invalid digit: {}", _0)]
    InvalidDigit(char),
    #[fail(display = "Invalid pixel: {}", _0)]
    InvalidPixel(u32),
    #[fail(display = "Incomplete layer")]
    IncompleteLayer,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Pixel {
    Black,
    White,
    Transparent,
}

impl TryFrom<u32> for Pixel {
    type Error = ParseError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Pixel::Black),
            1 => Ok(Pixel::White),
            2 => Ok(Pixel::Transparent),
            _ => Err(ParseError::InvalidPixel(value))
        }
    }
}

#[derive(Clone, Debug)]
pub struct Layer {
    pub pixels: Vec<Pixel>,
}

impl Layer {
    pub fn count_pixels(&self, digit: Pixel) -> usize {
        self.pixels.iter()
            .filter(|px| **px == digit)
            .count()
    }
}

#[derive(Clone, Debug)]
pub struct Display<'l> {
    width: usize,
    height: usize,
    layer: &'l Layer,
}

impl<'l> Display<'l> {
    const CHAR_WHITE: char = '█';
    const CHAR_BLACK: char = ' ';
    const CHAR_TRANSPARENT: char = '░';
}

impl<'l> std::fmt::Display for Display<'l> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        for y in 0 .. self.height {
            for x in 0 .. self.width {
                let px = match self.layer.pixels.get(y * self.width + x).unwrap() {
                    Pixel::Black => Self::CHAR_BLACK,
                    Pixel::White => Self::CHAR_WHITE,
                    Pixel::Transparent => Self::CHAR_TRANSPARENT,
                };
                write!(f, "{}", px)?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct SpaceImage {
    pub width: usize,
    pub height: usize,
    pub layers: Vec<Layer>,
}

impl SpaceImage {
    pub fn merge_layers(&self) -> Option<Layer> {
        let mut layer_iter = self.layers.iter();

        let mut merged = layer_iter.next()?.clone();
        let mut done;

        while let Some(layer) = layer_iter.next() {
            done = true;
            for (merged_px, px) in merged.pixels.iter_mut().zip(layer.pixels.iter()) {
                if *merged_px == Pixel::Transparent {
                    *merged_px = *px;
                    if *px == Pixel::Transparent {
                        done = false
                    }
                }
            }
            if done {
                break;
            }
        }

        Some(merged)
    }

    pub fn display<'l>(&self, layer: &'l Layer) -> Display<'l> {
        Display {
            width: self.width,
            height: self.height,
            layer,
        }
    }
}

fn to_radix(s: &str) -> Result<Vec<Pixel>, ParseError> {
    s.chars()
        .map(|c| {
            Ok(c.to_digit(10)
                .ok_or_else(|| ParseError::InvalidDigit(c))?
                .try_into()?)
        })
        .collect::<Result<Vec<Pixel>, ParseError>>()
}

impl FromStr for SpaceImage {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let width = 25;
        let height = 6;

        let mut layers = Vec::new();
        let mut current = s;

        while current.len() >= width * height {
            let (layer, rest) = current.split_at(width * height);
            current = rest;

            let pixels = to_radix(layer)?;
            assert_eq!(pixels.len(), width * height);

            layers.push(Layer {
                pixels,
            })
        }

        if current.is_empty() {
            Ok(SpaceImage {
                width,
                height,
                layers,
            })
        }
        else {
            Err(ParseError::IncompleteLayer)
        }
    }
}

#[aoc_generator(day8)]
pub fn input_generator(input: &str) -> SpaceImage {
    input.parse().unwrap()
}

#[aoc(day8, part1)]
pub fn solve_part1(image: &SpaceImage) -> usize {
    let layer = image.layers.iter()
        .min_by_key(|layer| layer.count_pixels(Pixel::Black))
        .unwrap();

    layer.count_pixels(Pixel::White) * layer.count_pixels(Pixel::Transparent)
}

#[aoc(day8, part2)]
pub fn solve_part2(image: &SpaceImage) -> String {
    format!("Image:\n{}", image.display(&image.merge_layers().unwrap()))
}
