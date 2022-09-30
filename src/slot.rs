use std::collections::HashMap;
use crate::util::Bounds;
use crate::util::Map3D;
use crate::util::Point;

#[derive(Debug, Clone)]
pub struct BaseSlotData {
	pub name: String,
	pub kind: String,
	pub bounds: Bounds,
}

#[derive(Debug, Clone)]
pub enum SlotError {
	NameIsAlreadyTaken {
		main_slot_name: String,
		subject_name: String,
		comment: String,
	},

	OutOfBounds {
		main_slot_name: String,
		subject_name: String,
		subject_size: Bounds,
		subject_pos: Point,
		comment: String,
	},
}

#[derive(Debug, Clone)]
pub struct Slot {
	name: String,
	#[allow(dead_code)]		// Feature with Slot kinds is planned
	kind: String,
	bounds: Bounds,

	shape_map: Map3D<Vec<usize>>,
	sectors: HashMap<String, (Point, Bounds)>,
}

impl Slot {
	pub fn name(&self) -> &String {
		&self.name
	}

	pub fn bounds(&self) -> Bounds {
		self.bounds.clone()
	}

	pub fn shape_map(&self) -> &Map3D<Vec<usize>> {
		&self.shape_map
	}

	pub fn sectors(&self) -> &HashMap<String, (Point, Bounds)> {
		&self.sectors
	}

	pub fn sectors_mut(&mut self) -> &mut HashMap<String, (Point, Bounds)> {
		&mut self.sectors
	}

	pub fn get_point(&self, pos: Point) -> Option<&Vec<usize>> {
		match pos.try_cast::<usize>() {
			Ok(pos) => self.shape_map.get(pos.tuple()),
			Err(_) => None,
		}
	}

	pub fn base_data(&self) -> BaseSlotData {
		BaseSlotData {
			name: self.name.clone(),
			kind: self.kind.clone(),
			bounds: self.bounds.clone(),
		}
	}

	pub fn new(name: String, kind: String, bounds: Bounds, shape_map: Map3D<Vec<usize>>) -> Self {
		Slot {
			name,
			kind,
			bounds,
			shape_map,
			sectors: HashMap::new()
		}
	}

	pub fn get_sector(&self, path: &String) -> Option<(Point, Bounds)> {
		if path.len() == 0 {
			Some((Point::new_ng(0, 0, 0), self.bounds()))
		} else {
			self.sectors().get(path)
				.map(|x| x.clone())
		}
	}

	pub fn bind_sector(&mut self, name: String, pos: Point, bounds: Bounds) -> Result<(), SlotError> {
		// Check that there is no already existing slot with such name.
		if self.sectors().get(&name).is_some() {
			return Err(SlotError::NameIsAlreadyTaken {
				main_slot_name: self.name.clone(),
				subject_name: name,
				comment: "Sector with such name was added before".to_string(),
			});
		}

		if name.len() == 0 {
			return Err(SlotError::NameIsAlreadyTaken {
				main_slot_name: self.name.clone(),
				subject_name: name,
				comment: "Slot sector name cannot be zero-sized, \
					because zero sized name is already taken by the \
					slot itself".to_string(),
			})
		}

		self.sectors_mut().insert(name, (pos, bounds));
		Ok(())
	}
}