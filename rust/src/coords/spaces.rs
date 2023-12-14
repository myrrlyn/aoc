#![doc = include_str!("../../doc/spaces.md")]

pub mod sparse;

pub use self::sparse::Cartesian2D as Sparse2D;

pub type Cartesian2D<I, T> = Sparse2D<I, T>;
