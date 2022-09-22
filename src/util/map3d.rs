use std::fmt::{Debug, Formatter};
use crate::util::Bounds;

#[derive(Clone)]
pub struct Map3D<T> {
	x_size: usize,
	y_size: usize,
	z_size: usize,

	data: Vec<T>,
}

impl<T: Debug> Debug for Map3D<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "Map3D {{{:?}}}", self.data)
	}
}

impl<T: Clone> Map3D<T> {
	#[allow(dead_code)]
	pub fn filled(size: (usize, usize, usize), default: T) -> Self {
		Map3D {
			x_size: size.0,
			y_size: size.1,
			z_size: size.2,
			data: [default].into_iter()
				.cycle()
				.take(size.0 * size.1 * size.2)
				.collect()
		}
	}
}

impl<T> Map3D<T> {
	#[allow(dead_code)]
	pub fn new<I>(size: (usize, usize, usize), data: I) -> Self
		where I: IntoIterator<Item = T>
	{
		Map3D {
			x_size: size.0,
			y_size: size.1,
			z_size: size.2,
			data: data.into_iter().collect()
		}
	}

	#[allow(dead_code)]		// TODO: remove this
	pub fn as_raw(&self) -> &Vec<T> {
		&self.data
	}

	#[allow(dead_code)]		// TODO: remove this
	pub fn as_raw_mut(&mut self) -> &mut Vec<T> {
		&mut self.data
	}

	#[allow(dead_code)]		// TODO: remove this
	pub fn size(&self) -> (usize, usize, usize) {
		(self.x_size, self.y_size, self.z_size)
	}

	#[allow(dead_code)]		// TODO: remove this
	pub fn size_u32(&self) -> (u32, u32, u32) {
		(self.x_size as u32, self.y_size as u32, self.z_size as u32)
	}

	#[allow(dead_code)]		// TODO: remove this
	pub fn bounds(&self) -> Bounds {
		Bounds::from_tuple(self.size_u32())
	}

	#[allow(dead_code)]		// TODO: remove this
	pub fn get(&self, pos: (usize, usize, usize)) -> Option<&T> {
		match self.to_id(pos) {
			None => None,
			Some(id) => Some(&self.data[id]),
		}
	}

	#[allow(dead_code)]		// TODO: remove this
	pub fn get_mut(&mut self, pos: (usize, usize, usize)) -> Option<&mut T> {
		match self.to_id(pos) {
			None => None,
			Some(id) => Some(&mut self.data[id]),
		}
	}

	#[allow(dead_code)]		// TODO: remove this
	pub fn replace(&mut self, pos: (usize, usize, usize), item: T) -> Option<T> {
		match self.to_id(pos) {
			None => None,
			Some(id) => Some(
				std::mem::replace(&mut self.data[id], item)
			),
		}
	}

	pub fn to_id(&self, pos: (usize, usize, usize)) -> Option<usize> {
		if pos.0 >= self.x_size ||
			pos.1 >= self.y_size ||
			pos.2 >= self.z_size
		{
			None
		} else {
			Some (
				pos.0 +
					pos.1 * self.x_size +
					pos.2 * self.x_size * self.y_size
			)
		}
	}

	#[allow(dead_code)]
	pub fn from_nested<I1, I2, I3>(vecs: I3) -> Self
		where I1: IntoIterator<Item = T>,
				I2: IntoIterator<Item = I1>,
				I3: IntoIterator<Item = I2>
	{
		let mut x_size: Option<usize> = None;
		let mut y_size: Option<usize> = None;
		let mut z_size = 0usize;

		let mut data: Vec<T> = Vec::new();

		for item_yx in vecs.into_iter() {
			let mut y = 0usize;
			for item_x in item_yx.into_iter() {
				let mut x = 0usize;
				for item in item_x.into_iter() {
					data.push(item);
					x += 1;
				}

				if x_size.is_some() {
					if x_size.unwrap() != x {
						panic!("Failed to create Map3D from Vec<Vec<Vec<T>>> - inconsistent size of X axis vectors.");
					}
				} else {
					x_size = Some(x);
				}

				y += 1;
			}

			if y_size.is_some() {
				if y_size.unwrap() != y {
					panic!("Failed to create Map3D from Vec<Vec<Vec<T>>> - inconsistent size of Y axis vectors.");
				}
			} else {
				y_size = Some(y);
			}

			z_size += 1;
		}

		Map3D {
			x_size: match x_size {
				None => 0,
				Some(size) => size
			},
			y_size: match y_size {
				None => 0,
				Some(size) => size
			},
			z_size,
			data,
		}
	}
}