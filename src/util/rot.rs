use crate::util::Mat3x3;
use crate::util::Point;
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

impl Rot {
	/// Returns (xaxis, zaxis, pos offset)
	pub fn to_sm_data(&self) -> (i32, i32, Point) {
		let (facing, orient) = self.to_facing_orient();
		let (xaxis, zaxis, dx, dy, dz) = facing.to_data(orient);
		(xaxis, zaxis, Point::new(dx, dy, dz))
	}

	pub fn to_facing_orient(&self) -> (Facing, Orient) {
		let z_axis = self.apply((0, 0, 1).into());
		let x_axis = self.apply((1, 0, 0).into());

		let facing =
			if *z_axis.z() == 1 		{ Facing::PosZ }
			else if *z_axis.z() == -1 	{ Facing::NegZ }
			else if *z_axis.y() == 1 	{ Facing::PosY }
			else if *z_axis.y() == -1 	{ Facing::NegY }
			else if *z_axis.x() == 1 	{ Facing::PosX }
			else if *z_axis.x() == -1 	{ Facing::NegX }
			else { panic!("Incorrect rotations") };

		if *x_axis.x() == 1 {
			return (facing, Orient::Up);
		} else if *x_axis.x() == -1 {
			return (facing, Orient::Down);
		}

		if *x_axis.z() == 1 {
			return (facing, Orient::Up);
		} else if *x_axis.z() == -1 {
			return (facing, Orient::Down);
		}

		if *x_axis.y() == 1 {
			return match facing {
				Facing::PosZ | Facing::NegX =>
					(facing, Orient::Right),
				_ => (facing, Orient::Left),
			}
		} else if *x_axis.y() == -1 {
			return match facing {
				Facing::PosZ | Facing::NegX =>
					(facing, Orient::Left),
				_ => (facing, Orient::Right),
			}
		}

		panic!("Incorrect rotations");
	}
}

///\[\(xaxis, zaxis, offset_x, offset_y, offset_z\)\]
const ROTATIONS_DATA: [(i32, i32, i32, i32, i32); 24] = [
	( 1, -2, 0,  0, 0),
	(-2, -1, 1,  0, 0),
	(-1,  2, 1, -1, 0),
	( 2,  1, 0, -1, 0),
	( 3, -1, 1, -1, 0),
	(-1, -3, 1, -1, 1),
	(-3,  1, 0, -1, 1),
	( 1,  3, 0, -1, 0),
	( 3,  2, 0, -1, 0),
	( 2, -3, 0, -1, 1),
	(-3, -2, 0,  0, 1),
	(-2,  3, 0,  0, 0),
	( 1,  2, 0, -1, 1),
	( 2, -1, 1, -1, 1),
	(-1, -2, 1,  0, 1),
	(-2,  1, 0,  0, 1),
	( 3,  1, 0,  0, 0),
	( 1, -3, 0,  0, 1),
	(-3, -1, 1,  0, 1),
	(-1,  3, 1,  0, 0),
	( 3, -2, 1,  0, 0),
	(-2, -3, 1,  0, 1),
	(-3,  2, 1, -1, 1),
	( 2,  3, 1, -1, 0),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Facing {
	PosX,
	PosY,
	PosZ,
	NegX,
	NegY,
	NegZ,
}

impl Facing {
	pub fn to_data(&self, orient: Orient) -> (i32, i32, i32, i32, i32) {
		let facing_id = match self {
			Facing::PosZ => 0,
			Facing::PosY => 1,
			Facing::PosX => 2,
			Facing::NegZ => 3,
			Facing::NegY => 4,
			Facing::NegX => 5,
		};

		let orient_id = match orient {
			Orient::Up => 0,
			Orient::Down => 1,
			Orient::Left => 2,
			Orient::Right => 3,
		};

		ROTATIONS_DATA[facing_id * 4 + orient_id]
	}

	pub fn to_rot(&self) -> Rot {
		match self {
			Facing::PosX => Rot::new(0, 1, 0),
			Facing::PosY => Rot::new(-1, 0, 0),
			Facing::PosZ => Rot::new(0, 0, 0),
			Facing::NegX => Rot::new(0, -1, 0),
			Facing::NegY => Rot::new(1, 0, 0),
			Facing::NegZ => Rot::new(2, 0, 0),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Orient {
	Up,
	Down,
	Left,
	Right,
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

#[test]
fn facing_to_rot_test() {
	let vec = Vec3::new_ng(0_i32, 0, 1);

	let pos_x = Facing::PosX.to_rot().apply(vec.clone());
	assert_eq!(Vec3::new_ng(1_i32, 0, 0), pos_x);

	let pos_y = Facing::PosY.to_rot().apply(vec.clone());
	assert_eq!(Vec3::new_ng(0_i32, 1, 0), pos_y);

	let pos_z = Facing::PosZ.to_rot().apply(vec.clone());
	assert_eq!(Vec3::new_ng(0_i32, 0, 1), pos_z);

	let neg_x = Facing::NegX.to_rot().apply(vec.clone());
	assert_eq!(Vec3::new_ng(-1_i32, 0, 0), neg_x);

	let neg_y = Facing::NegY.to_rot().apply(vec.clone());
	assert_eq!(Vec3::new_ng(0_i32, -1, 0), neg_y);

	let neg_z = Facing::NegZ.to_rot().apply(vec.clone());
	assert_eq!(Vec3::new_ng(0_i32, 0, -1), neg_z);
}

impl<N1, N2, N3> Into<Rot> for (N1, N2, N3)
	where N1: IntoNumber, N2: IntoNumber, N3: IntoNumber
{
	fn into(self) -> Rot {
		Rot::new(self.0.into_number(), self.1.into_number(), self.2.into_number())
	}
}

trait IntoNumber {
	fn into_number(self) -> i32;
}

impl IntoNumber for i8 		{ fn into_number(self) -> i32 { self as i32 } }
impl IntoNumber for i16 	{ fn into_number(self) -> i32 { self as i32 } }
impl IntoNumber for i32 	{ fn into_number(self) -> i32 { self } }
impl IntoNumber for i64 	{ fn into_number(self) -> i32 { self as i32 } }
impl IntoNumber for i128 	{ fn into_number(self) -> i32 { self as i32 } }

impl IntoNumber for u8 		{ fn into_number(self) -> i32 { self as i32 } }
impl IntoNumber for u16 	{ fn into_number(self) -> i32 { self as i32 } }
impl IntoNumber for u32 	{ fn into_number(self) -> i32 { self as i32 } }
impl IntoNumber for u64 	{ fn into_number(self) -> i32 { self as i32 } }
impl IntoNumber for u128 	{ fn into_number(self) -> i32 { self as i32 } }