mod vec3;
mod map3d;
pub use vec3::Vec3;
pub use map3d::Map3D;

pub type Bounds = Vec3<u32>;
pub type Point = Vec3<i32>;
pub type Rotation = Vec3<i32>;