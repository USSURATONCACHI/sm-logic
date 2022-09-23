use std::ops::Neg;
use crate::util::mat::Mat4;
use crate::util::{Point, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rot {
	rot_x: i32,
	rot_y: i32,
	rot_z: i32,
}

impl Rot {
	pub fn new(rot_x: i32, rot_y: i32, rot_z: i32) -> Self {
		Rot {
			rot_x: ((rot_x % 4) + 4) % 4 ,
			rot_y: ((rot_y % 4) + 4) % 4,
			rot_z: ((rot_z % 4) + 4) % 4
		}
	}

	pub fn from_tuple(rot: (i32, i32, i32)) -> Self {
		Rot::new(rot.0, rot.1, rot.2)
	}

	pub fn from_vec(vec: Vec3<i32>) -> Self {
		Self::from_tuple(vec.tuple())
	}

	pub fn apply<N>(&self, vec: Vec3<N>) -> Vec3<N>
		where N: Neg<Output = N>
	{
		apply_rot_z(
			apply_rot_y(
				apply_rot_x(
					vec,
					self.rot_x
				),
				self.rot_y
			),
			self.rot_z
		)
	}

	pub fn apply_to_rot<R: AsRef<Rot>>(&self, first: R) -> Rot {
		combine_rots(self, first.as_ref())
	}
}

impl AsRef<Rot> for Rot {
	fn as_ref(&self) -> &Rot {
		self
	}
}

pub fn combine_rots(first: &Rot, second: &Rot) -> Rot {
	let pi = std::f32::consts::PI;

	let basis_x = Point::new_ng(1, 0, 0);
	let basis_y = Point::new_ng(0, 1, 0);
	let basis_z = Point::new_ng(0, 0, 1);

	let basis_x = second.apply(first.apply(basis_x)).tuple();
	let basis_y = second.apply(first.apply(basis_y)).tuple();
	let basis_z = second.apply(first.apply(basis_z)).tuple();

	let c = Mat4::from([
		basis_x.0 as f32, basis_x.1 as f32, basis_x.2 as f32, 0.0,
		basis_y.0 as f32, basis_y.1 as f32, basis_y.2 as f32, 0.0,
		basis_z.0 as f32, basis_z.1 as f32, basis_z.2 as f32, 0.0,
		0.0, 0.0, 0.0, 1.0
	]);
	//Transform back to angles
	let ay = - c.get(2, 0).asin();
	let ax = if ay.cos() < 0.0 {
		(-c.get(2, 1)).atan2(-c.get(2, 2))
	} else {
		(c.get(2, 1)).atan2(c.get(2, 2))
	};
	let az = if ay.cos() < 0.0 {
		(-c.get(1, 0)).atan2(-c.get(0, 0))
	} else {
		(c.get(1, 0)).atan2(c.get(0, 0))
	};

	let ax = (ax.to_degrees() / 90.0).round() as i32;
	let ay = (ay.to_degrees() / 90.0).round() as i32;
	let az = (az.to_degrees() / 90.0).round() as i32;

	let res = Rot::new(ax, ay, az);
	res
}

fn apply_rot_x<N>(vec: Vec3<N>, amount: i32) -> Vec3<N>
	where N: Neg<Output = N>
{
	let amount = ((amount % 4) + 4) % 4;
	let (x, y, z) = vec.tuple();
	match amount {
		0 => Vec3::new_ng(x, y, z),
		1 => Vec3::new_ng(x, -z, y),
		2 => Vec3::new_ng(x, -y, -z),
		3 => Vec3::new_ng(x, z, -y),
		_ => panic!("Mod operation somehow failed :/ (internal error)"),
	}
}

fn apply_rot_y<N>(vec: Vec3<N>, amount: i32) -> Vec3<N>
	where N: Neg<Output = N>
{
	let amount = ((amount % 4) + 4) % 4;
	let (x, y, z) = vec.tuple();
	match amount {
		0 => Vec3::new_ng(x, y, z),
		1 => Vec3::new_ng(z, y, -x),
		2 => Vec3::new_ng(-x, y, -z),
		3 => Vec3::new_ng(-z, y, x),
		_ => panic!("Mod operation somehow failed :/ (internal error)"),
	}
}

fn apply_rot_z<N>(vec: Vec3<N>, amount: i32) -> Vec3<N>
	where N: Neg<Output = N>
{
	let amount = ((amount % 4) + 4) % 4;
	let (x, y, z) = vec.tuple();
	match amount {
		0 => Vec3::new_ng(x, y, z),
		1 => Vec3::new_ng(-y, x, z),
		2 => Vec3::new_ng(-x, -y, z),
		3 => Vec3::new_ng(y, -x, z),
		_ => panic!("Mod operation somehow failed :/ (internal error)"),
	}
}

#[test]
fn test_rot() {
	let mut rots: Vec<Rot> = vec![];

	for rx in -5..5 {
		for ry in -5..5 {
			for rz in -5..5 {
				rots.push(Rot::new(rx, ry, rz));
			}
		}
	}

	for rot_1 in rots.iter() {
		for rot_2 in rots.iter() {
			let vector = Point::new(1, 1, 1);
			println!("Testing {:?} + {:?} = {:?}", rot_1, rot_2, rot_2.apply_to_rot(rot_1));
			assert_eq!(
				rot_2.apply(rot_1.apply(vector)),
				rot_2.apply_to_rot(rot_1).apply(vector)
			);
		}
	}
}