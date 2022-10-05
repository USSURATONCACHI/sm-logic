use std::ops::IndexMut;
use std::ops::Mul;
use std::ops::Index;
use std::ops::Add;
use std::ops::Sub;
use crate::util::Vec3;

/// Mathematical matrix with size 3 by 3. Contains numbers of type `i32`
///
/// # Example
/// ```
/// # use crate::sm_logic::util::Mat3x3;
///
/// let mat = Mat3x3::unit(7);
/// assert_eq!(mat.det(), 7 * 7 * 7);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Mat3x3 {
	values: [[i32; 3]; 3],
}

impl Mat3x3 {
	/// Creates matrix with all values set to `fill_with` value.
	///
	/// # Example
	/// ```
	/// # use crate::sm_logic::util::Mat3x3;
	/// let mat = Mat3x3::new(42);
	///
	/// for i in 0..3 {
	/// 	for j in 0..3 {
	/// 		assert_eq!(mat[i][j], 42);
	/// 	}
	/// }
	/// ```
	pub fn new(fill_with: i32) -> Self {
		let n = fill_with;
		Mat3x3 {
			values: [
				[n, n, n],
				[n, n, n],
				[n, n, n],
			],
		}
	}

	/// Creates matrix with main diagonal values equal to passed value,
	/// and all other values equal to zero.
	/// Determinant of such matrix will be equal to `val * val * val`
	///
	/// # Example
	/// ```
	/// # use crate::sm_logic::util::Mat3x3;
	///
	/// let mat = Mat3x3::unit(7);
	/// assert_eq!(mat.det(), 7 * 7 * 7);
	/// ```
	pub fn unit(val: i32) -> Self {
		let d = val;
		Mat3x3 {
			values:  [
				[d, 0, 0],
				[0, d, 0],
				[0, 0, d],
			],
		}
	}

	/// Creates matrix from raw data.
	///
	/// # Example
	/// ```
	/// # use crate::sm_logic::util::Mat3x3;
	///
	/// let mat = Mat3x3::from_raw(
	/// [
	/// 	[7, 0, 0],
	/// 	[0, 7, 0],
	/// 	[0, 0, 7]
	/// ]);
	/// assert_eq!(mat.det(), 7 * 7 * 7);
	/// ```
	pub fn from_raw(values: [[i32; 3]; 3]) -> Self {
		Mat3x3 {
			values
		}
	}

	/// Calculates determinant of the matrix.
	///
	/// # Example
	/// ```
	/// # use crate::sm_logic::util::Mat3x3;
	///
	/// let mat = Mat3x3::from_raw(
	/// [
	/// 	[1, 2, 3],
	/// 	[8, 9, 4],
	/// 	[7, 14, 5]
	/// ]);
	/// assert_eq!(mat.det(), 384);
	/// ```
	pub fn det(&self) -> i32 {
		self[0][0] * self[1][1] * self[2][2] +
		self[0][1] * self[1][2] * self[2][0] +
		self[0][2] * self[1][0] * self[2][1] -

		self[0][2] * self[1][1] * self[2][0] +
		self[0][0] * self[1][2] * self[2][1] +
		self[0][1] * self[1][0] * self[2][2]
	}
}

impl Mat3x3 {
	/// Creates matrix of rotation around X axis.
	/// Each angle unit is equal to `90 deg`.
	///
	/// `rot_x_mat(7)` means rotation around X axis for `7 * 90 deg`
	pub fn rot_x_mat(ax: i32) -> Mat3x3 {
		Mat3x3::from_raw([
			[1, 0, 0],
			[0, quarter_cos(ax),  -quarter_sin(ax)],
			[0, quarter_sin(ax),  quarter_cos(ax)]
		])
	}

	/// Creates matrix of rotation around Y axis.
	/// Each angle unit is equal to `90 deg`.
	///
	/// `rot_y_mat(7)` means rotation around Y axis for `7 * 90 deg`
	pub fn rot_y_mat(ay: i32) -> Mat3x3 {
		Mat3x3::from_raw([
			[quarter_cos(ay), 0, quarter_sin(ay)],
			[0, 1, 0],
			[-quarter_sin(ay), 0, quarter_cos(ay)],
		])
	}

	/// Creates matrix of rotation around Z axis.
	/// Each angle unit is equal to `90 deg`.
	///
	/// `rot_z_mat(7)` means rotation around Z axis for `7 * 90 deg`
	pub fn rot_z_mat(az: i32) -> Mat3x3 {
		Mat3x3::from_raw([
			[quarter_cos(az), -quarter_sin(az),  0],
			[quarter_sin(az), quarter_cos(az),  0],
			[0, 0, 1],
		])
	}

	/// Creates matrix of rotation around three axes.
	/// X, then Y, then Z - rotation order
	/// Each angle unit is equal to `90 deg`.
	///
	/// `rot_mat(7, 0, 2)` means rotation around X axis for `7 * 90 deg`
	/// and then around Z axis for `2 * 90 deg`
	pub fn rot_mat(ax: i32, ay: i32, az: i32) -> Mat3x3 {
		Mat3x3::rot_z_mat(az) *
		Mat3x3::rot_y_mat(ay) *
		Mat3x3::rot_x_mat(ax)
	}
}

impl Index<usize> for Mat3x3 {
	type Output = [i32; 3];

	fn index(&self, index: usize) -> &Self::Output {
		&self.values[index]
	}
}

impl IndexMut<usize> for Mat3x3 {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.values[index]
	}
}

impl Add for Mat3x3 {
	type Output = Mat3x3;

	fn add(mut self, rhs: Self) -> Self::Output {
		for i in 0..3 {
			for j in 0..3 {
				self[i][j] += rhs[i][j];
			}
		}
		self
	}
}

impl Mul for Mat3x3 {
	type Output = Mat3x3;

	fn mul(self, rhs: Self) -> Self::Output {
		let mut result = Mat3x3::new(0);

		for i in 0..3 {
			for j in 0..3 {
				result[i][j] = (0..3)
					.map(|k| self[i][k] * rhs[k][j])
					.sum();
			}
		}

		result
	}
}

impl Mul<Vec3<i32>> for Mat3x3 {
	type Output = Vec3<i32>;

	fn mul(self, rhs: Vec3<i32>) -> Self::Output {
		let mut result = [0i32, 0i32, 0i32];

		for i in 0..3 {
			result[i] = (0..3)
				.map(|k| self[i][k] * rhs[k])
				.sum();
		}

		Vec3::new(result[0], result[1], result[2])
	}
}

impl Sub for Mat3x3 {
	type Output = Mat3x3;

	fn sub(mut self, rhs: Self) -> Self::Output {
		for i in 0..3 {
			for j in 0..3 {
				self[i][j] -= rhs[i][j];
			}
		}
		self
	}
}


fn quarter_sin(ang: i32) -> i32 {
	let ang = ((ang % 4) + 4) % 4;
	let deg = (ang * 90) as f32;
	deg.to_radians().sin().round() as i32
}

fn quarter_cos(ang: i32) -> i32 {
	let ang = ((ang % 4) + 4) % 4;
	let deg = (ang * 90) as f32;
	deg.to_radians().cos().round() as i32
}