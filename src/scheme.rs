use json::{JsonValue, object};
use crate::shape::Shape;
use crate::slot::{Slot, SlotSector};
use crate::util::{Bounds};
use crate::util::palette::{input_color, output_color};
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

	/// Sets color of every shape to a given color.
	/// Basically just fills everything with color.
	pub fn full_paint<S: Into<String>>(&mut self, color: S) {
		let color = color.into();

		for (_, _, shape) in &mut self.shapes {
			shape.set_color(&color);
		}
	}

	/// Only paints shapes with default color. If a shape was painted
	/// before, its color won't change.
	pub fn soft_paint<S: Into<String>>(&mut self, color: S) {
		let color = color.into();

		for (_, _, shape) in &mut self.shapes {
			if shape.get_color().is_none() {
				shape.set_color(&color);
			}
		}
	}

	/// Shifts, rotates and offsets controller ids, then returns raw data:
	///
	/// (shapes, inputs, outputs)
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
	pub fn to_json(self) -> JsonValue {
		self.to_json_custom_colors(input_color, output_color)
	}

	/// Converts [`Scheme`] to JSON blueprint.
	pub fn to_json_custom_colors<P1, P2>(mut self, inputs_palette: P1, outputs_palette: P2) -> JsonValue
		where P1: Fn(u32, Point) -> String,
				P2: Fn(u32, Point) -> String,
	{
		let mut array: Vec<JsonValue> = Vec::new();

		// Slot
		for (i, bind) in self.inputs.into_iter().enumerate() {
			let map_size: (i32, i32, i32) = bind.shape_map().bounds().cast().tuple();

			// Point of slot
			for x in 0..map_size.0 {
				for y in 0..map_size.1 {
					for z in 0..map_size.2 {
						// All the connections of the point
						for vec in bind.shape_map().get((x as usize, y as usize, z as usize)) {
							// Connection of the point
							for id in vec {
								let (_, _, shape) = &mut self.shapes[*id];
								shape.set_color(inputs_palette(i as u32, (x, y, z).into()));
							}
						}
					}
				}
			}
		}

		for (i, bind) in self.outputs.into_iter().enumerate() {
			let map_size: (i32, i32, i32) = bind.shape_map().bounds().cast().tuple();

			// Point of slot
			for x in 0..map_size.0 {
				for y in 0..map_size.1 {
					for z in 0..map_size.2 {
						// All the connections of the point
						for vec in bind.shape_map().get((x as usize, y as usize, z as usize)) {
							// Connection of the point
							for id in vec {
								let (_, _, shape) = &mut self.shapes[*id];
								shape.set_color(outputs_palette(i as u32, (x, y, z).into()));
							}
						}
					}
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


	fn _nobounds_remove_shape(&mut self, id: usize) {
		let _ = self.shapes.remove(id);

		for (_, _, shape) in self.shapes.iter_mut() {
			let mut conns_count = shape.connections().len();
			let mut i = 0;

			while i < conns_count {
				let connection = shape.connections()[i];
				if connection == id {
					shape.connections_mut().remove(i);
					conns_count -= 1;
				} else if connection > id {
					shape.connections_mut()[i] -= 1;
					i += 1;
				} else {
					i += 1;
				}
			}
		}

		for input in &mut self.inputs {
			input.shape_was_removed(id);
		}

		for output in &mut self.outputs {
			output.shape_was_removed(id);
		}
	}

	pub fn remove_unused(&mut self) {
		// used = connected to output
		let mut is_used: Vec<bool> = self.shapes.iter().map(
			|(_, _, shape)| !shape.has_output()
		).collect();

		// in the first place, all shapes connected to output are used
		for slot in self.outputs.iter() {
			for point in slot.shape_map().as_raw() {
				for connection in point {
					if *connection < is_used.len() {
						is_used[*connection] = true;
					}
				}
			}
		}

		// Then "usefulness" spreads to other shapes in reverse iteratively
		let mut new_used = 0;
		loop {
			for (id, (_, _, shape)) in self.shapes.iter().enumerate() {
				if let Some(false) = is_used.get(id) {
					for connection in shape.connections() {
						// If the shape is connected to used shape, "usefulness" spreads
						if let Some(true) = is_used.get(*connection) {
							is_used[id] = true;
							new_used = 1;
						}
					}
				}
			}

			if new_used == 0 {
				break;
			}
			new_used = 0;
		}

		// Then all unused shapes get deleted
		for i in (0..is_used.len()).rev() {
			if is_used[i] == false {
				self._nobounds_remove_shape(i);
			}
		}

		// Check bounds, those might have been updated
		self.set_bounds();
	}

	pub fn set_force_used(&mut self) {

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