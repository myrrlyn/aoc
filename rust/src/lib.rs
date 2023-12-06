use std::ops::RangeInclusive;

pub use funty::{Integral, Signed};
use nom::{character::complete::digit1, combinator::map_res, IResult};
use tap::Tap;

pub mod coords;

pub use crate::coords::{
    Cartesian2DPoint as Coord2D, Cartesian2DSpace as Grid2D, Cartesian3DPoint as Coord3D,
    Cartesian3DSpace as Grid3D,
};

#[macro_export]
macro_rules! input {
    () => {
        include_str!("../../input.txt")
    };
}

#[macro_export]
macro_rules! sample {
    () => {
        include_str!("../../sample.txt")
    };
}

pub trait Puzzle {
    type Input;
    type State;
    type ParseError<'a>;
    type ComputeError;

    fn parse(input: &str) -> Result<Self::Input, Self::ParseError<'_>>;

    fn prepare_state(input: Self::Input) -> Result<Self::State, Self::ComputeError>;

    fn part_1(state: &mut Self::State) -> Result<i64, Self::ComputeError>;

    fn part_2(state: &mut Self::State) -> Result<i64, Self::ComputeError>;
}

/// Unifies a series of inclusive ranges by joining any that overlap.
pub fn unify_ranges_inclusive<I: Integral>(
    ranges: impl Iterator<Item = RangeInclusive<I>>,
) -> Vec<RangeInclusive<I>> {
    ranges
        .collect::<Vec<_>>()
        .tap_mut(|v| v.sort_by_key(|r| *r.start()))
        .into_iter()
        .fold(Vec::<RangeInclusive<I>>::new(), |mut acc, next| {
            if let Some(prev) = acc.last_mut() {
                let (a1, a2) = (*prev.start(), *prev.end());
                let (b1, b2) = (*next.start(), *next.end());
                if a2 >= b1 {
                    *prev = a1.min(b1)..=a2.max(b2);
                    return acc;
                }
            }
            acc.push(next);
            acc
        })
}

pub fn parse_number<T: FromStr>(text: &str) -> IResult<&str, T> {
    map_res(digit1, T::from_str)(text)
}
