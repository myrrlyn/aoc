use nom::{
    bytes::complete::{tag, take},
    character::complete::i32 as get_i32,
    combinator::map,
    sequence::{preceded, separated_pair, tuple},
    IResult,
};
use rayon::prelude::*;
use std::{ops::RangeInclusive, str::FromStr};
use tap::Tap;
use wyz_aoc::{Coord2D, Grid2D};

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
    sensors.sort_by_key(|s| s.location.manhattan_distance(Coord2D::ZERO));

    let mut grid = Grid2D::<i32, Fill>::default();
    grid.seed(coverage_and_sensors(
        sensors.iter().copied(),
        Coord2D {
            x: i32::MIN,
            y: SOUGHT_ROW,
        },
        Coord2D {
            x: i32::MAX,
            y: SOUGHT_ROW,
        },
    ));
    let pt1 = grid
        .row(&SOUGHT_ROW)
        .expect("this row should be present")
        .values()
        .filter(|f| matches!(**f, Fill::Covered))
        .count();
    println!("part 1: {pt1}"); // (1835569, 3983183)

    for (x, y, tune) in (0..=MAX_RANK)
        .into_par_iter()
        .inspect(|y| {
            y.tap_dbg(|&&y| {
                let mil = y / 1_000_000;
                let thou = (y % 1_000_000) / 1_000;
                let base = y % 1_000;
                println!("searching {mil},{thou:0>3},{base:0>3}");
            });
        })
        .map(|y| {
            (
                y,
                sensors
                    .iter()
                    // Get the row coverage for this sensor, clamped to the search space
                    .flat_map(move |s| s.coverage_in_row(y, 0, MAX_RANK))
                    // Collect them all so they can be sorted by x position
                    .collect::<Vec<_>>()
                    .tap_mut(|v| v.sort_by_key(|r| *r.start()))
                    .into_iter()
                    // Unify as many as possible
                    .fold(Vec::<RangeInclusive<i32>>::new(), |mut acc, next| {
                        // If the vector has a right-most range, get it
                        if let Some(last) = acc.last_mut() {
                            let (a1, a2) = (*last.start(), *last.end());
                            let (b1, b2) = (*next.start(), *next.end());
                            // If the existing range overlaps the next range, unify them
                            if a2 >= b1 {
                                *last = a1.min(b1)..=a2.max(b2);
                            }
                            // Otherwise, they are disjoint, so push the incoming
                            else {
                                acc.push(next);
                            }
                        }
                        // If the collection is empty, this is the first range in the series.
                        else {
                            acc.push(next)
                        }
                        acc
                    })
                    // Walk over each *pair* of coverage ranges
                    .windows(2)
                    // and get the range *in between* them.
                    .map(|windows| {
                        let left = &windows[0];
                        let right = &windows[1];
                        // coverage is inclusive, so start after the left end and end before the right start.
                        *left.end() + 1..*right.start()
                    })
                    // and collect each *uncovered* range.
                    .collect::<Vec<_>>(),
            )
        })
        // discard rows that have no exclusions
        .filter(|(_, uncovered)| !uncovered.is_empty())
        .inspect(|(row, v)| {
            v.tap_dbg(|&v| {
                for r in v {
                    println!("{row}: {r:?}");
                }
            });
        })
        .collect::<Vec<_>>()
        .into_iter()
        .flat_map(|(y, exclusions)| {
            exclusions
                .into_iter()
                .flatten()
                .map(move |x| (x, y, ((x as i64) * (MAX_RANK as i64)) + y as i64))
        })
    {
        println!("part 2: ({x}, {y}) is {tune}");
    }
}

fn coverage_and_sensors(
    sensors: impl Iterator<Item = Sensor>,
    small_corner: Coord2D<i32>,
    large_corner: Coord2D<i32>,
) -> impl Iterator<Item = (Coord2D<i32>, Fill)> {
    sensors.flat_map(
        move |s @ Sensor {
                  location, nearest, ..
              }| {
            s.covered_points_in_region(small_corner, large_corner)
                .map(|c| (c, Fill::Covered))
                .chain([(location, Fill::Sensor), (nearest, Fill::Beacon)])
        },
    )
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

    fn covered_points_in_region(
        &self,
        small_corner: Coord2D<i32>,
        large_corner: Coord2D<i32>,
    ) -> impl Iterator<Item = Coord2D<i32>> {
        let src = self.location;
        let dst = self.nearest;

        let radius = src.manhattan_distance(dst);
        let y_min = src.y - radius;
        let y_max = src.y + radius;
        (y_min..=y_max)
            .filter(move |y| (small_corner.y..=large_corner.y).contains(y))
            .flat_map(move |y| {
                let x_range = radius - (src.y.abs_diff(y) as i32);
                let x_min = src.x - x_range;
                let x_max = src.x + x_range;
                (x_min..=x_max)
                    .filter(move |x| (small_corner.x..=large_corner.x).contains(x))
                    .map(move |x| Coord2D { x, y })
            })
    }

    fn search_radius(&self) -> i32 {
        self.location.manhattan_distance(self.nearest)
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

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Fill {
    #[default]
    Unknown,
    Covered,
    Sensor,
    Beacon,
}
