use crate::shape::Shape;
use crate::slot::Slot;
use crate::util::Bounds;
use crate::util::Rot;
use crate::util::Point;

#[derive(Debug, Clone)]
pub struct Scheme {
	shapes: Vec<(Point, Rot, Shape)>,
	inputs: Vec<Slot>,
	outputs: Vec<Slot>,
	bounds: Bounds,
}

impl Scheme {
	pub fn create(
		shapes: Vec<(Point, Rot, Shape)>,
		inputs: Vec<Slot>,
		outputs: Vec<Slot>
	) -> Self {
		let mut scheme = Scheme {
			shapes,
			inputs,
			outputs,
			bounds: (0, 0, 0).into(),
		};
		scheme.set_bounds();
		scheme
	}

	pub fn rotate(&mut self, rot: Rot) {
		let global_rot = rot;
		for (pos, rot, _) in &mut self.shapes {
			*pos = global_rot.apply(*pos);
			*rot = global_rot.apply_to_rot(rot.clone());
		}
		self.set_bounds();
	}

	pub fn inputs(&self) -> &Vec<Slot> {
		&self.inputs
	}

	pub fn outputs(&self) -> &Vec<Slot> {
		&self.outputs
	}

	pub fn input<N>(&self, name: N) -> Option<&Slot>
		where N: Into<String>
	{
		find_slot(name, self.inputs())
	}

	pub fn output<N>(&self, name: N) -> Option<&Slot>
		where N: Into<String>
	{
		find_slot(name, self.outputs())
	}

	pub fn shapes_count(&self) -> usize {
		self.shapes.len()
	}

	pub fn bounds(&self) -> Bounds {
		self.bounds.clone()
	}
}

impl Scheme {
	// start, size
	fn calculate_bounds(&self) -> (Point, Bounds) {
		if self.shapes.len() == 0 {
			return ((0, 0, 0).into(), (0, 0, 0).into());
		}

		let mut min: Point = Point::new(i32::MAX, i32::MAX, i32::MAX);
		let mut max: Point = Point::new(i32::MIN, i32::MIN, i32::MIN);

		for (pos, rot, shape) in self.shapes.iter() {
			let start = pos.clone();
			let end = pos.clone() + rot.apply(shape.bounds().cast());

			min = fold_coords(
				min,
				[start, end],
				|a, b| if a < b { a } else { b }
			);

			max = fold_coords(
				max,
				[start, end],
				|a, b| if a > b { a } else { b }
			);
		}

		(min, (max - min).cast())
	}

	fn set_bounds(&mut self) {
		let (_, bounds) = self.calculate_bounds();
		self.bounds = bounds;
	}
}

fn find_slot<N: Into<String>>(name: N, slots: &Vec<Slot>) -> Option<&Slot> {
	let name = name.into();

	for slot in slots {
		if slot.name().eq(&name) {
			return Some(slot);
		}
	}

	None
}

/// Folds coordinates of all points separately by `fold` function
fn fold_coords<P, I, F>(start_point: Point, points: I, fold: F) -> Point
	where P: Into<Point>,
		  I: IntoIterator<Item = P>,
		  F: Fn(i32, i32) -> i32
{
	let (mut x, mut y, mut z) = start_point.tuple();

	for point in points {
		let (px, py, pz) = point.into().tuple();
		x = fold(x, px);
		y = fold(y, py);
		z = fold(z, pz);
	}

	Point::new(x, y, z)
}