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
