mod vec3;
mod map3d;
mod rot;
mod mat3;

pub use vec3::Vec3;
pub use map3d::Map3D;
pub use rot::*;
pub use mat3::Mat3x3;

use crate::scheme::Scheme;
use crate::shape::Shape;
use crate::shape::vanilla::Gate;

pub type Bounds = Vec3<u32>;
pub type Point = Vec3<i32>;

pub const TICKS_PER_SECOND: u32 = 40;

/// Temporary list of color of inputs.
pub const INPUTS_PALETTE: [&str; 4] = [
	"0A3EE2",
	"D02525",
	"7514ED",
	"CF11D2",
];

/// Temporary list of color of outputs.
pub const OUTPUTS_PALETTE: [&str; 4] = [
	"19E753",
	"A0EA00",
	"68FF88",
	"CBF66F",
];

pub fn get_input_color(input_id: usize) -> String {
	INPUTS_PALETTE[input_id % INPUTS_PALETTE.len()].to_string()
}

pub fn get_output_color(input_id: usize) -> String {
	OUTPUTS_PALETTE[input_id % OUTPUTS_PALETTE.len()].to_string()
}

#[derive(Debug, Clone, Copy)]
pub enum GateMode {
	AND,
	OR,
	XOR,
	NAND,
	NOR,
	XNOR,
}

impl GateMode {
	pub fn to_number(self) -> usize {
		match self {
			GateMode::AND => 0,
			GateMode::OR => 1,
			GateMode::XOR => 2,
			GateMode::NAND => 3,
			GateMode::NOR => 4,
			GateMode::XNOR => 5,
		}
	}
}

impl Into<Shape> for GateMode {
	fn into(self) -> Shape {
		Gate::new(self)
	}
}

impl Into<Scheme> for GateMode {
	fn into(self) -> Scheme {
		let shape: Shape = self.into();
		shape.into()
	}
}

pub fn is_point_in_bounds(point: Point, bounds: Bounds) -> bool {
	*point.x() >= 0 &&
		*point.y() >= 0 &&
		*point.z() >= 0 &&
		*point.x() < (*bounds.x() as i32) &&
		*point.y() < (*bounds.y() as i32) &&
		*point.z() < (*bounds.z() as i32)
}

pub fn split_path(path: String) -> Vec<String> {
	path.split("/")
		.map(|s| s.to_string())
		.collect()
}

pub fn split_first_token(path: String) -> (String, Option<String>) {
	match path.find("/") {
		None => (path, None),
		Some(pos) => {
			let (_, tail) = path.split_at(pos + 1);
			let (token, _) = path.split_at(pos);
			let tail = tail.to_string();
			(token.to_string(), Some(tail))
		}
	}
}