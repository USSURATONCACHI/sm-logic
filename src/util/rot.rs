use crate::util::mat::Mat4;
use crate::util::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rot {
	matrix: Mat4,
}

impl Rot {
	pub fn new(rot_x: i32, rot_y: i32, rot_z: i32) -> Self {
		let rot_x = ((rot_x % 4) + 4) % 4;
		let rot_y = ((rot_y % 4) + 4) % 4;
		let rot_z = ((rot_z % 4) + 4) % 4;
		Rot {
			matrix: Mat4::rotation_mat(
				((rot_x * 90) as f32).to_radians(),
				((rot_y * 90) as f32).to_radians(),
				((rot_z * 90) as f32).to_radians()
			)
		}
	}

	pub fn from_tuple(rot: (i32, i32, i32)) -> Self {
		Rot::new(rot.0, rot.1, rot.2)
	}

	pub fn from_vec(vec: Vec3<i32>) -> Self {
		Self::from_tuple(vec.tuple())
	}

	pub fn apply(&self, vec: Vec3<i32>) -> Vec3<i32> {
		use crate::util::mat::Vec4;

		let (x, y, z) = vec.tuple();
		let f32_vec = Vec4::new(x as f32, y as f32, z as f32, 0.0);
		let result = self.matrix * f32_vec;
		//let result = result.unit();

		Vec3::new_ng(
			result.x().round() as i32,
			result.y().round() as i32,
			result.z().round() as i32,
		)
	}

	pub fn apply_to_rot<R: AsRef<Rot>>(&self, first: R) -> Rot {
		Rot {
			matrix: self.matrix * first.as_ref().matrix
		}
	}
}

impl AsRef<Rot> for Rot {
	fn as_ref(&self) -> &Rot {
		self
	}
}
/*
#[test]
fn test_rot() {
	let mut rots: Vec<((i32, i32, i32), Rot)> = vec![];

	for rx in -5..5 {
		for ry in -5..5 {
			for rz in -5..5 {
				rots.push(((rx, ry, rz), Rot::new(rx, ry, rz)));
			}
		}
	}

	for (ang_1, rot_1) in rots.iter() {
		//for (ang_2, rot_2) in rots.iter() {
			let vector = Point::new(1, 1, 1);
			println!("Testing {:?} + {:?} = {:?}", vector, ang_1, rot_1.apply(vector));
			//assert_eq!(
				// rot_2 * (rot_1 * vector)
				//rot_2.apply(rot_1.apply(vector)),
				// (rot_2 * rot_1) * vector
				//rot_2.apply_to_rot(rot_1).apply(vector)
			//);
		//}
	}
}*/