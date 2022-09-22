use crate::util::Bounds;
use crate::util::Map3D;
use crate::util::Point;

#[derive(Debug, Clone)]
pub struct BaseSlotData {
	name: String,
	kind: String,
	size: Bounds,
}

#[derive(Debug, Clone)]
pub enum SlotError {
	NameIsAlreadyTaken {
		main_slot_name: String,
		subject_name: String
	},

	SlotIsOutOfBounds {
		main_slot_name: String,
		subject_name: String,
		subject_size: Bounds,
		subject_pos: Point,
	}
}

#[derive(Debug, Clone)]
pub struct Slot {
	name: String,
	kind: String,
	size: Bounds,

	shape_map: Map3D<usize>,
	sub_slots: Vec<Slot>,
}

impl Slot {
	pub fn name(&self) -> &String {
		&self.name
	}

	pub fn size(&self) -> Bounds {
		self.size.clone()
	}

	pub fn shape_map(&self) -> &Map3D<usize> {
		&self.shape_map
	}

	pub fn new(name: String, kind: String, size: Bounds, shape_map: Map3D<usize>, sub_slots: Vec<Slot>) -> Self {
		Slot {
			name,
			kind,
			size,
			shape_map,
			sub_slots
		}
	}

	pub fn get(&self, path: String) -> Option<&Slot> {
		todo!()
	}

	pub fn add_sub_slot(&mut self, path: String, pos: Point, slot: Slot) -> Result<(), SlotError> {
		todo!()
	}

	pub fn set_sub_slot(&mut self, path: String, pos: Point, slot: BaseSlotData) -> Result<(), SlotError> {
		todo!()
	}
}


