use json::{JsonValue, object};
use crate::shape::Shape;
use crate::slot::{Slot, SlotSector};
use crate::util;
use crate::util::Bounds;
use crate::util::split_first_token;
use crate::util::Rot;
use crate::util::Point;

pub const DEFAULT_SLOT: &str = "_";

/// Some structure/creation/blueprint made up of in-game
/// blocks and parts.
///
/// Can have inputs and outputs, which are [`Slot`]s.
///
/// Any [`Shape`] can be converted to Scheme.
///
/// Using `Combiner` schemes can be connected to each other and combined
/// into bigger scheme.
///
/// Every scheme has size/bounds. It can change, if scheme is rotated.
#[derive(Debug, Clone)]
pub struct Scheme {
	shapes: Vec<(Point, Rot, Shape)>,
	inputs: Vec<Slot>,
	outputs: Vec<Slot>,
	bounds: Bounds,
}

impl Scheme {
	/// Scheme constructor.
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

	/// Rotates whole Scheme / rotates every [`Shape`] of it.
	pub fn rotate(&mut self, rot: Rot) {
		let global_rot = rot;
		for (pos, rot, _) in &mut self.shapes {
			*pos = global_rot.apply(*pos);
			*rot = global_rot.apply_to_rot(rot.clone());
		}
		self.set_bounds();
	}

	/// Returns all the inputs of the Scheme.
	pub fn inputs(&self) -> &Vec<Slot> {
		&self.inputs
	}

	/// Returns all the outputs of the Scheme.
	pub fn outputs(&self) -> &Vec<Slot> {
		&self.outputs
	}

	/// Tries to find input slot/sector with given name.
	pub fn input<N>(&self, name: N) -> Option<(&Slot, &SlotSector)>
		where N: Into<String>
	{
		let (name, sector) = split_first_token(name.into());
		let sector = match sector {
			None => "".to_string(),
			Some(sector) => sector,
		};

		match find_slot(name, self.inputs()) {
			None => None,
			Some(slot) => {
				let sector = slot.get_sector(&sector);
				sector.map(|sector| (slot, sector))
			}
		}
	}

	/// Tries to find output slot/sector with given name.
	pub fn output<N>(&self, name: N) -> Option<(&Slot, &SlotSector)>
		where N: Into<String>
	{
		let (name, sector) = split_first_token(name.into());
		let sector = match sector {
			None => "".to_string(),
			Some(sector) => sector,
		};

		match find_slot(name, self.outputs()) {
			None => None,
			Some(slot) => {
				let sector = slot.get_sector(&sector);
				sector.map(|sector| (slot, sector))
			}
		}
	}

	// Do I need to add documentation to such methods?
	pub fn shapes_count(&self) -> usize {
		self.shapes.len()
	}

	pub fn shapes(&self) -> &Vec<(Point, Rot, Shape)> {
		&self.shapes
	}

	pub fn bounds(&self) -> Bounds {
		self.bounds.clone()
	}

	/// Shifts, rotates and offsets controller ids, then returns raw data:
	///
	/// shapes, inputs, outputs
	pub fn disassemble(mut self, start_shape: usize, pos: Point, rot: Rot) -> (Vec<(Point, Rot, Shape)>, Vec<Slot>, Vec<Slot>) {
		let (start, _) = self.calculate_bounds();

		for (shape_pos, shape_rot, shape) in &mut self.shapes {
			*shape_rot = rot.apply_to_rot(shape_rot.clone());
			*shape_pos = pos - start + rot.apply(shape_pos.clone());

			for connection in shape.connections_mut() {
				*connection += start_shape;
			}
		}

		(self.shapes, self.inputs, self.outputs)
	}

	/// Converts [`Scheme`] to JSON blueprint.
	pub fn to_json(mut self) -> JsonValue {
		let mut array: Vec<JsonValue> = Vec::new();

		for (i, bind) in self.inputs.into_iter().enumerate() {
			for vec in bind.shape_map().as_raw() {
				for id in vec {
					let (_, _, shape) = &mut self.shapes[*id];
					shape.set_color(util::get_input_color(i));
				}
			}
		}

		for (i, bind) in self.outputs.into_iter().enumerate() {
			for vec in bind.shape_map().as_raw() {
				for id in vec {
					let (_, _, shape) = &mut self.shapes[*id];
					shape.set_color(util::get_output_color(i));
				}
			}
		}

		for (i, (pos, rot, shape)) in self.shapes.into_iter().enumerate() {
			array.push(shape.build(pos, rot, i));
		}

		let array = JsonValue::Array(array);
		let mut obj = object!{
			"bodies": [
				{
				}
			],
			"version": 4_i32
		};
		obj["bodies"][0]["childs"] = array;
		obj
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

			// Shapes are being rotated around BLOCK at (0, 0, 0) position.
			// Not around corner of the block. And so, this "*2-1" is needed to
			// rotate bounds around center of the first block.
			let bounds = shape.bounds().cast::<i32>() * 2 - 1;
			let bounds = (rot.apply(bounds) + 1) / 2;
			let end = pos.clone() + bounds;

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

pub fn find_slot<N: Into<String>>(name: N, slots: &Vec<Slot>) -> Option<&Slot> {
	let name = name.into();
	let search_for = if name.len() == 0 {
		DEFAULT_SLOT
	} else {
		&name
	};

	for slot in slots {
		if slot.name().eq(search_for) {
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