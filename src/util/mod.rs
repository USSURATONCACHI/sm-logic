//! Crate with widely useful stuff, like [`Vec3`], [`Map3D`] or
//! rotation matrices ([`Rot`]).

mod vec3;
mod map3d;
mod rot;
mod mat3;
pub mod palette;

pub use vec3::Vec3;
pub use map3d::Map3D;
pub use rot::*;
pub use mat3::Mat3x3;

pub type Bounds = Vec3<u32>;
pub type Point = Vec3<i32>;

pub const TICKS_PER_SECOND: u32 = 40;

/// Returns true if each coordinate lies in the `0..bound` range
pub fn is_point_in_bounds(point: Point, bounds: Bounds) -> bool {
	*point.x() >= 0 &&
		*point.y() >= 0 &&
		*point.z() >= 0 &&
		*point.x() < (*bounds.x() as i32) &&
		*point.y() < (*bounds.y() as i32) &&
		*point.z() < (*bounds.z() as i32)
}

/// Splits string at the first '/' (slash) symbol and returns
/// (all the symbols before, all the symbols after). The '/' (slash)
/// symbol itself is being dropped.
///
/// # Example
/// ```
/// # use crate::sm_logic::util::split_first_token;
/// let string = "This/could be/literally///any/$path$$$".to_string();
/// let (token, tail) = split_first_token(string);
/// assert_eq!(token, "This".to_string());
/// assert_eq!(tail, Some("could be/literally///any/$path$$$".to_string()));
///
/// ```
///
/// # Example
/// ```
/// # use crate::sm_logic::util::split_first_token;
/// let no_tail_1 = "There is no tail/".to_string();
/// let no_tail_2 = "There is no tail".to_string();
///
/// let (token, tail) = split_first_token(no_tail_1);
/// assert_eq!(token, "There is no tail".to_string());
/// assert_eq!(tail, Some("".to_string()));
///
/// let (token, tail) = split_first_token(no_tail_2);
/// assert_eq!(token, "There is no tail".to_string());
/// assert_eq!(tail, None);
///
/// ```
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