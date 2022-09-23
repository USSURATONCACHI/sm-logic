use std::ops::Neg;
use crate::util::Vec3;

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

	fn apply_to_basis(&self, basis: Vec3<Vec3<i32>>) -> Vec3<Vec3<i32>> {
		let (ax, ay, az) = basis.tuple();

		Vec3::new_ng(
			self.apply(ax),
			self.apply(ay),
			self.apply(az)
		)
	}

	fn to_basis(&self) -> Vec3<Vec3<i32>> {
		let x = Vec3::new_ng(1_i32, 0, 0);
		let y = Vec3::new_ng(0, 1_i32, 0);
		let z = Vec3::new_ng(0, 0, 1_i32);

		let res = Vec3::new_ng(
			self.apply(x),
			self.apply(y),
			self.apply(z)
		);
		res
	}

	fn from_basis(basis: Vec3<Vec3<i32>>) -> Rot {
		let main_basis = Vec3::new_ng(
			Vec3::new_ng(1_i32, 0, 0),
			Vec3::new_ng(0, 1_i32, 0),
			Vec3::new_ng(0, 0, 1_i32)
		);

		fn get_angle(axis_to_check: Axis, main_basis: &Vec3<Vec3<i32>>, deviant_basis: &Vec3<Vec3<i32>>) -> i32 {
			let main_axis = match axis_to_check {
				Axis::X => main_basis.x(),
				Axis::Y => main_basis.y(),
				Axis::Z => main_basis.z(),
			};

			let axis = get_perpendicular_axis(main_axis, deviant_basis);

			if axis == axis_to_check {
				panic!("Something went wrong - axis is perpendicular to itself");
			}

			match axis {
				Axis::X => get_ang_betw_axes(main_basis.x().clone(), deviant_basis.x().clone()),
				Axis::Y => get_ang_betw_axes(main_basis.y().clone(), deviant_basis.y().clone()),
				Axis::Z => get_ang_betw_axes(main_basis.z().clone(), deviant_basis.z().clone()),
			}
		}

		let ang_z = get_angle(Axis::Z, &main_basis, &basis);
		let basis = Rot::new(0, 0, -ang_z)
			.apply_to_basis(basis);

		let ang_y = get_angle(Axis::Y, &main_basis, &basis);
		let basis = Rot::new(0, -ang_y, 0)
			.apply_to_basis(basis);

		let ang_x = get_angle(Axis::X, &main_basis, &basis);

		Rot::new(ang_x, ang_y, ang_z)
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Axis {
	X, Y, Z
}

fn cross_product(vec_a: Vec3<i32>, vec_b: Vec3<i32>) -> Vec3<i32> {
	let (ax, ay, az) = vec_a.tuple();
	let (bx, by, bz) = vec_b.tuple();

	Vec3::new_ng(
		ay * bz - az * by,
		ax * bz - az * bx,
		ax * by - ay * bx,
	)
}

fn vec3_i32_len(vec: &Vec3<i32>) -> f32 {
	let pow = vec.x().pow(2) + vec.y().pow(2) + vec.z().pow(2);
	(pow as f32).sqrt()
}

// Only use if one of axes are grid-aligned
fn get_ang_betw_axes(main_axis: Vec3<i32>, deviant_axis: Vec3<i32>) -> i32 {
	if main_axis == deviant_axis {
		return 0;
	}
	if main_axis == -deviant_axis {
		return 2;
	}

	let axis_a = main_axis;
	let axis_b = deviant_axis;
	let ang_cos = (axis_a.dot(axis_b) as f32) / (vec3_i32_len(&axis_a) * vec3_i32_len(&axis_b));
	let ang = ang_cos.acos().to_degrees() / 90.0;
	let ang = ang.round() as i32;

	let normal = cross_product(axis_a, axis_b);

	let rot = if normal.x().abs() > 0 {
		Rot::new(ang, 0, 0)
	} else if normal.y().abs() > 0 {
		Rot::new(ang, 0, 0)
	} else if normal.z().abs() > 0 {
		Rot::new(ang, 0, 0)
	} else {
		panic!("Unintended use of private function - one of axes are not grid-aligned ({:?} | {:?} = {:?})", axis_a, axis_b, normal);
	};

	if rot.apply(axis_b) == axis_a {
		-ang
	} else {
		ang
	}
}

fn get_perpendicular_axis(perp_to: &Vec3<i32>, axes: &Vec3<Vec3<i32>>) -> Axis {
	let (ax, ay, az) = axes.tuple_ref();

	if ax.dot(perp_to.clone()) == 0 && *perp_to.x() == 0 {
		Axis::X
	} else if ay.dot(perp_to.clone()) == 0 && *perp_to.y() == 0 {
		Axis::Y
	} else if az.dot(perp_to.clone()) == 0 && *perp_to.z() == 0 {
		Axis::Z
	} else {
		panic!("Unintended use of private function - incorrect basis")
	}
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
	for rx in -5..5 {
		for ry in -5..5 {
			for rz in -5..5 {
				let rot = Rot::new(rx, ry, rz);
				let basis = rot.to_basis();
				assert_eq!(rot, Rot::from_basis(basis));
			}
		}
	}
}