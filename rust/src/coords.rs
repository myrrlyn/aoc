pub use funty::Signed;

pub mod points;
pub mod spaces;

pub use self::{
	points::{
		Cartesian2D as Cartesian2DPoint,
		Cartesian3D as Cartesian3DPoint,
	},
	spaces::{
		Cartesian2D as Cartesian2DSpace,
		Cartesian3D as Cartesian3DSpace,
	},
};
