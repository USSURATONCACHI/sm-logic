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

pub fn split_last_token(path: String) -> (Option<String>, String) {
	match path.rfind("/") {
		None => (None, path),
		Some(pos) => {
			let (start, _) = path.split_at(pos);
			let start = start.to_string();
			(Some(start), path)
		}
	}
}