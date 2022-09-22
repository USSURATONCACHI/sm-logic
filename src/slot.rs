use crate::util::{Bounds, is_point_in_bounds, split_first_token};
use crate::util::Map3D;
use crate::util::Point;

#[derive(Debug, Clone)]
pub struct BaseSlotData {
	pub name: String,
	pub kind: String,
	pub size: Bounds,
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
	pos: Point,

	shape_map: Map3D<Vec<usize>>,
	sub_slots: Vec<Slot>,
}

impl Slot {
	pub fn name(&self) -> &String {
		&self.name
	}

	pub fn size(&self) -> Bounds {
		self.size.clone()
	}

	pub fn pos(&self) -> Point {
		self.pos.clone()
	}

	pub fn shape_map(&self) -> &Map3D<Vec<usize>> {
		&self.shape_map
	}

	pub fn sub_slots(&self) -> &Vec<Slot> {
		&self.sub_slots
	}

	pub fn sub_slots_mut(&mut self) -> &mut Vec<Slot> {
		&mut self.sub_slots
	}

	pub fn get_point(&self, pos: Point) -> Option<&Vec<usize>> {
		match pos.try_cast::<usize>() {
			Ok(pos) => self.shape_map.get(pos.tuple()),
			Err(_) => None,
		}
	}

	pub fn new(name: String, kind: String, size: Bounds, pos: Point, shape_map: Map3D<Vec<usize>>, sub_slots: Vec<Slot>) -> Self {
		Slot {
			name,
			kind,
			size,
			pos,
			shape_map,
			sub_slots
		}
	}

	pub fn get(&self, path: String) -> Option<&Slot> {
		get_slot_by_name(path, self.sub_slots())
	}

	pub fn get_mut(&mut self, path: String) -> Option<&mut Slot> {
		get_slot_by_name_mut(path, self.sub_slots_mut())
	}

	pub fn add_sub_slot(&mut self, path: String, pos: Point, slot: Slot) -> Result<(), SlotError> {
		// Check that there is no already existing slot with such name.
		let check_path = format!("{}/{}", path, slot.name());
		if self.get(check_path.clone()).is_some() {
			return Err(SlotError::NameIsAlreadyTaken {
				main_slot_name: self.name().to_string(),
				subject_name: check_path,
			});
		}

		// Split path into first token and the other path
		let (sub_slot_name, tail) = split_first_token(path);
		let tail = match tail {
			Some(tail) => tail,
			None => "".to_string(),
		};

		if sub_slot_name.len() == 0 {
			// Empty path = add slot right here
			let start = pos;
			let end = pos + slot.size().cast::<i32>() - 1;

			if !is_point_in_bounds(start, self.size()) ||
				!is_point_in_bounds(end, self.size())
			{
				return Err(SlotError::SlotIsOutOfBounds {
					main_slot_name: self.name().to_string(),
					subject_name: slot.name,
					subject_size: slot.size,
					subject_pos: pos,
				})
			}

			self.sub_slots.push(slot);
			return Ok(());
		} else {
			// Non-empty path = follow the path
			for sub_slot in self.sub_slots_mut() {
				if sub_slot.name.eq(&sub_slot_name) {
					let sub_slot_pos = sub_slot.pos();
					return sub_slot.add_sub_slot(tail, pos - sub_slot_pos, slot);
				}
			}

			// If there is nowhere to follow - create new subslot just to fit the one being added.
			// Then follow the path
			// CREATE A NEW SLOT HERE
			let base_data = BaseSlotData {
				name: sub_slot_name.clone(),
				kind: "none".to_string(),
				size: slot.size(),
			};
			self.set_sub_slot("".to_string(), pos, base_data)?;
			return self.get_mut(sub_slot_name)
				.unwrap()
				.add_sub_slot(tail, Point::new(0, 0, 0), slot);
		}
	}

	pub fn set_sub_slot(&mut self, path: String, pos: Point, slot: BaseSlotData) -> Result<(), SlotError> {
		todo!()
	}
}

pub fn get_slot_by_name(path: String, all_slots: &Vec<Slot>) -> Option<&Slot> {
	let (slot_name, tail_path) = split_first_token(path);

	for slot in all_slots {
		if slot.name().eq(&slot_name) {
			return match tail_path {
				None => Some(slot),
				Some(tail_path) =>
					get_slot_by_name(tail_path, slot.sub_slots())
			}
		}
	}

	None
}

pub fn get_slot_by_name_mut(path: String, all_slots: &mut Vec<Slot>) -> Option<&mut Slot> {
	let (slot_name, tail_path) = split_first_token(path);

	for slot in all_slots {
		if slot.name().eq(&slot_name) {
			return match tail_path {
				None => Some(slot),
				Some(tail_path) =>
					get_slot_by_name_mut(tail_path, slot.sub_slots_mut())
			}
		}
	}

	None
}