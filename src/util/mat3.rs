use std::iter::Sum;
use std::ops::{Add, Index, Mul, Sub};
use std::process::Output;

pub const SIZE: usize = 3;

#[derive(Clone, Debug)]
pub struct Mat3x3<T> {
	values: [T; SIZE * SIZE],
}

impl<T: Clone> Mat3x3<T> {
	pub fn new(fill_with: T) -> Self {
		let values: Vec<T> = [fill_with].into_iter().cycle()
			.take(SIZE * SIZE)
			.collect();

		Mat3x3 {
			values: match values.try_into() {
				Ok(values) => values,
				Err(_) => panic!("Mat3x3::new error"),
			}
		}
	}

	pub fn from_raw(values: [T; SIZE * SIZE]) -> Self {
		Mat3x3 {
			values
		}
	}
}

impl<T> Index<usize> for Mat3x3<T> {
	type Output = [T];

	fn index(&self, index: usize) -> &Self::Output {
		let start = index * SIZE;
		let end = start + SIZE;
		&self.values[start..end]
	}
}

impl<O, T: Add<Output = O>> Add for Mat3x3<T> {
	type Output = Mat3x3<O>;

	fn add(self, rhs: Self) -> Self::Output {
		let new_values: Vec<O> = self.values.into_iter()
			.zip(rhs.values.into_iter())
			.map(|(a, b)| a + b)
			.collect();

		Mat3x3 {
			values: match new_values.try_into() {
				Ok(values) => values,
				Err(_) => panic!("Mat3x3::add error"),
			}
		}
	}
}

impl<MulRes, T> Mul for Mat3x3<T>
	where MulRes: Sum,
		  T: Mul<Output = MulRes> + Clone
{
	type Output = Mat3x3<MulRes>;

	fn mul(self, rhs: Self) -> Self::Output {
		let mut new_values: Vec<MulRes> = vec![];

		for i in 0..SIZE {
			for j in 0..SIZE {
				//let mut sum = self[i][0].clone() * rhs[0][j].clone();
				let sum: MulRes = (0..SIZE)
					.map(|k| self[i][k].clone() * rhs[k][j].clone())
					.sum();

				new_values.push(sum);
			}
		}

		Mat3x3 {
			values: match new_values.try_into() {
				Ok(values) => values,
				Err(_) => panic!("Mat3x3::mul error")
			}
		}
	}
}

impl<O, T: Sub<Output = O>> Sub for Mat3x3<T> {
	type Output = Mat3x3<O>;

	fn sub(self, rhs: Self) -> Self::Output {
		let new_values: Vec<O> = self.values.into_iter()
			.zip(rhs.values.into_iter())
			.map(|(a, b)| a - b)
			.collect();

		Mat3x3 {
			values: match new_values.try_into() {
				Ok(values) => values,
				Err(_) => panic!("Mat3x3::add error"),
			}
		}
	}
}