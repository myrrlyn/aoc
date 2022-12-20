use nom::{
    bytes::complete::{tag, take},
    character::complete::i32 as get_i32,
    combinator::map,
    sequence::{preceded, separated_pair, tuple},
    IResult,
};
use rayon::prelude::*;
use std::{ops::RangeInclusive, str::FromStr};
use tap::{Pipe, Tap};
use wyz_aoc::Coord2D;

static INPUT: &str = wyz_aoc::input!();

const SOUGHT_ROW: i32 = 2_000_000;
// const SOUGHT_ROW: i32 = 10;

const MAX_RANK: i32 = 4_000_000;

fn main() {
    let mut sensors = INPUT
        .lines()
        .enumerate()
        .flat_map(|(idx, line)| Sensor::parse_with_ident(idx, line).map(|(_, s)| s))
        .collect::<Vec<_>>();
    assert_eq!(sensors.len(), 29);
    sensors.sort_by_key(|s| s.location.axial_distance(Coord2D::ZERO));

    // Get all the points that are covered by a sensor's scan radius
    let covered = sensors
        .iter()
        .flat_map(|s| s.coverage_in_row(SOUGHT_ROW, i32::MIN, i32::MAX))
        .pipe(wyz_aoc::unify_ranges_inclusive)
        .into_iter()
        .map(|r| (*r.end() - *r.start() + 1))
        .sum::<i32>();
    // Get all the points that contain a sensor or a beacon.
    let objects = sensors
        .iter()
        .flat_map(
            |&Sensor {
                 location, nearest, ..
             }| [location, nearest],
        )
        .collect::<Vec<_>>()
        .tap_mut(|v| v.sort())
        .tap_mut(|v| v.dedup())
        .into_iter()
        .filter(|&Coord2D { y, .. }| y == SOUGHT_ROW)
        .count() as i32;
    let pt1 = covered - objects;
    println!("part 1: {pt1}");

    (0..=MAX_RANK)
        .into_par_iter()
        .filter_map(|y| {
            sensors
                .iter()
                .flat_map(move |s| s.coverage_in_row(y, 0, MAX_RANK))
                .pipe(wyz_aoc::unify_ranges_inclusive)
                // .array_windows::<2>()
                .windows(2)
                .map(|w| unsafe { &*(w.as_ptr().cast::<[RangeInclusive<i32>; 2]>()) })
                .map(|[left, right]| *left.end() + 1..*right.start())
                .collect::<Vec<_>>()
                .pipe(|v| if v.is_empty() { None } else { Some((y, v)) })
        })
        .for_each(|(y, exclusions)| {
            for x in exclusions.into_iter().flatten() {
                let tune = ((x as i64) * (MAX_RANK as i64)) + y as i64;
                println!("part 2: ({x}, {y}) is {tune}");
            }
        });
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Sensor {
    location: Coord2D<i32>,
    nearest: Coord2D<i32>,
    ident: char,
}

impl Sensor {
    fn parse(line: &str) -> IResult<&str, Self> {
        fn point(text: &str) -> IResult<&str, i32> {
            preceded(take(2usize), get_i32)(text)
        }
        fn points(text: &str) -> IResult<&str, (i32, i32)> {
            separated_pair(point, tag(", "), point)(text)
        }
        map(
            tuple((
                preceded(tag("Sensor at "), points),
                preceded(tag(": closest beacon is at "), points),
            )),
            |((x1, y1), (x2, y2))| Self {
                location: Coord2D::new(x1, y1),
                nearest: Coord2D::new(x2, y2),
                ident: '_',
            },
        )(line)
    }

    fn parse_with_ident(idx: usize, line: &str) -> IResult<&str, Self> {
        Self::parse(line).map(|(r, mut s)| {
            s.ident = ('A'..='Z')
                .chain('a'..='z')
                .nth(idx)
                .expect("ran out of identifiers");
            (r, s)
        })
    }

    fn coverage_in_row(&self, row: i32, x_min: i32, x_max: i32) -> Option<RangeInclusive<i32>> {
        let src = self.location;

        let radius = self.search_radius();
        let (y_min, y_max) = (src.y - radius, src.y + radius);
        if !(y_min..=y_max).contains(&row) {
            return None;
        }
        let x_range = radius - (src.y.abs_diff(row) as i32);
        let start = (src.x - x_range).clamp(x_min, x_max);
        let end = (src.x + x_range).clamp(x_min, x_max);
        Some(start..=end)
    }

    fn search_radius(&self) -> i32 {
        self.location.axial_distance(self.nearest)
    }
}

impl FromStr for Sensor {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        <Self>::parse(text)
            .map_err(|e| e.to_string())
            .and_then(|(r, s)| {
                if !r.is_empty() {
                    Err("incomplete parse".into())
                } else {
                    Ok(s)
                }
            })
    }
}
