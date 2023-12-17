#![doc = include_str!("../../doc/spaces.md")]

pub mod dense;
pub mod sparse;

pub use self::{
	dense::Cartesian2D as Dense2D,
	sparse::Cartesian2D as Sparse2D,
};

pub type Cartesian2D<I, T> = Sparse2D<I, T>;
