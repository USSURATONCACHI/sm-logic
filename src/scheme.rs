use crate::util::{Bounds, Map3D};
use crate::util::Point;
use crate::util::Rotation;

type Shape = ();

pub struct Slot {
	name: String,
	kind: String,
	shapes: Map3D<usize>,
}

pub struct Scheme {
	shapes: Vec<(Point, Rotation, Shape)>,
	inputs: Vec<Slot>,
	outputs: Vec<Slot>,
}

impl Scheme {
	pub fn create(
		shapes: Vec<(Point, Rotation, Shape)>,
		inputs: Vec<Slot>,
		outputs: Vec<Slot>
	) -> Self {
		Scheme {
			shapes,
			inputs,
			outputs,
		}
	}

	pub fn mirror_x(&mut self) {
		todo!()
	}

	pub fn mirror_y(&mut self) {
		todo!()
	}

	pub fn mirror_z(&mut self) {
		todo!()
	}

	pub fn rotate(&mut self, rot: Rotation) {
		todo!()
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
		todo!()
	}

	pub fn output<N>(&self, name: N) -> Option<&Slot>
		where N: Into<String>
	{
		todo!()
	}

	pub fn shapes_count(&self) -> usize {
		self.shapes.len()
	}

	pub fn bounds(&self) -> Bounds {
		todo!()
	}

	fn offset_to_origin(&mut self) {
		todo!()
	}
}