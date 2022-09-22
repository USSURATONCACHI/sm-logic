mod vec3;
mod map3d;
pub use vec3::Vec3;
pub use map3d::Map3D;

pub type Bounds = Vec3<u32>;
pub type Point = Vec3<i32>;
pub type Rotation = Vec3<i32>;

pub fn is_point_in_bounds(point: Point, bounds: Bounds) -> bool {
	*point.x() >= 0 &&
		*point.y() >= 0 &&
		*point.z() >= 0 &&
		*point.x() < (*bounds.x() as i32) &&
		*point.y() < (*bounds.x() as i32) &&
		*point.z() < (*bounds.x() as i32)
}