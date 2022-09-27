mod vec3;
mod map3d;
mod rot;
mod mat3;

pub use vec3::Vec3;
pub use map3d::Map3D;
pub use rot::*;
pub use mat3::Mat3x3;

pub type Bounds = Vec3<u32>;
pub type Point = Vec3<i32>;


pub const TICKS_PER_SECOND: u32 = 40;


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

pub fn is_point_in_bounds(point: Point, bounds: Bounds) -> bool {
	*point.x() >= 0 &&
		*point.y() >= 0 &&
		*point.z() >= 0 &&
		*point.x() < (*bounds.x() as i32) &&
		*point.y() < (*bounds.x() as i32) &&
		*point.z() < (*bounds.x() as i32)
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
			let tail = tail.to_string();
			(path, Some(tail))
		}
	}
}