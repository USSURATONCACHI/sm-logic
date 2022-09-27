use crate::util::mat3::Mat3x3;
use crate::util::Vec3;

#[derive(Debug, Clone, PartialEq)]
pub struct Rot {
	matrix: Mat3x3,
}

impl Rot {
	pub fn new(rot_x: i32, rot_y: i32, rot_z: i32) -> Self {
		Rot {
			matrix: Mat3x3::rot_mat(rot_x, rot_y, rot_z),
		}
	}

	pub fn from_tuple(rot: (i32, i32, i32)) -> Self {
		Rot::new(rot.0, rot.1, rot.2)
	}

	pub fn from_vec(vec: Vec3<i32>) -> Self {
		Self::from_tuple(vec.tuple())
	}

	pub fn apply(&self, vec: Vec3<i32>) -> Vec3<i32> {
		self.matrix.clone() * vec
	}

	pub fn apply_to_rot(&self, rhs: Rot) -> Rot {
		Rot {
			matrix: self.matrix.clone() * rhs.matrix
		}
	}
}

#[test]
fn rotation_test() {
	let mut rots: Vec<Rot> = vec![];
	for ax in 0..4 {
		for ay in 0..4 {
			for az in 0..4 {
				rots.push(Rot::new(ax, ay, az));
			}
		}
	}
	let rots = rots;

	for rot_1 in rots.iter() {
		for rot_2 in rots.iter() {
			let vec = Vec3::new_ng(1i32, 0, 0);

			// Order must not matter
			let order_1 = rot_2.apply(rot_1.apply(vec.clone()));
			let order_2 = rot_2.apply_to_rot(rot_1.clone()).apply(vec.clone());

			assert_eq!(order_1, order_2);
		}
	}
}