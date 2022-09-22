use crate::util::{Bounds, Vec3};
use crate::util::is_point_in_bounds;
use crate::util::split_first_token;
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
		self.check_slot_doesnt_exist(&path, slot.name())?;

		if path.len() == 0 {
			// Add right here
			let mut slot = slot;
			slot.pos = pos;
			self.sub_slots.push(slot);
			return Ok(());
			// TODO: UPSTREAM ADDING OF SHAPES
		} else {
			// Follow the path
			self.create_path(path.clone(), pos, slot.size.clone())
				.map_err(|_| SlotError::SlotIsOutOfBounds {
					main_slot_name: self.name().to_string(),
					subject_name: slot.name().to_string(),
					subject_size: slot.size(),
					subject_pos: pos,
				})?;

			return self.get_mut(path.clone())
				.unwrap()
				.add_sub_slot(path, pos, slot);
		}
	}

	pub fn set_sub_slot(&mut self, path: String, pos: Point, slot: BaseSlotData) -> Result<(), SlotError> {
		// Check that there is no already existing slot with such name.
		self.check_slot_doesnt_exist(&path, &slot.name)?;

		if path.len() == 0 {
			// Add right here
			self.create_sector_slot(slot.name, pos, slot.size, slot.kind);
			return Ok(());
		} else {
			// Follow the path
			self.create_path(path.clone(), pos, slot.size.clone())
				.map_err(|_| SlotError::SlotIsOutOfBounds {
					main_slot_name: self.name().to_string(),
					subject_name: slot.name.clone(),
					subject_size: slot.size.clone(),
					subject_pos: pos,
				})?;

			return self.get_mut(path.clone())
				.unwrap()
				.set_sub_slot(path, pos, slot);
		}
	}
}

impl Slot {
	fn check_slot_doesnt_exist(&self, path: &String, slot_name: &String) -> Result<(), SlotError> {
		let check_path = format!("{}/{}", path, slot_name);
		if self.get(check_path.clone()).is_some() {
			return Err(SlotError::NameIsAlreadyTaken {
				main_slot_name: self.name().to_string(),
				subject_name: check_path,
			});
		}
		Ok(())
	}

	/// Creates subslots (sectors) by provided path. If slot does
	/// already exists - function follows through. If some segment of
	/// path does not exist - function creates it.
	///
	/// Ok(()) means success and Err(()) means error of SlotIsOutOfBounds
	fn create_path(&mut self, path: String, pos: Point, size: Bounds) -> Result<(), ()> {
		if !self.is_path_available(path.clone(), pos, size) {
			return Err(());
		}
		// Split path into first token and the other path
		let (sub_slot_name, tail) = split_first_token(path);
		let tail = match tail {
			Some(tail) => tail,
			None => "".to_string(),
		};

		if sub_slot_name.len() > 0 {
			// FOLLOW THE PATH
			for sub_slot in self.sub_slots_mut() {
				if sub_slot.name.eq(&sub_slot_name) {
					let sub_slot_pos = sub_slot.pos();
					return sub_slot.create_path(tail, pos - sub_slot_pos, size);
				}
			}
			// PATH NOT FOUND = CREATE SUBSLOT
			self.create_sector_slot(sub_slot_name.clone(), pos, size, "none (sector)".to_string());
			// AND FOLLOW THE PATH

			// Since new sector slot is created at `pos` position we need to next follow the path
			// with pos of (0, 0, 0) - because is is relative to the slot.
			return self.get_mut(sub_slot_name)
				.unwrap()
				.create_path(tail, Point::new(0, 0, 0), size);
		} else {
			// Empty path = everything is already created
			return Ok(());
		}
	}

	fn create_sector_slot(&mut self, name: String, pos: Point, size: Bounds, kind: String) {
		if self.get(name.clone()).is_some() {
			panic!("Cannot create sector with already taken name");
		}
		let map_size = size.cast().tuple();
		let mut map: Map3D<Vec<usize>> = Map3D::new(map_size, Vec::new());

		for map_x in 0..map_size.0 {
			for map_y in 0..map_size.1 {
				for map_z in 0..map_size.2 {
					let map_point = Vec3::<usize>::new(map_x, map_y, map_z);
					let self_point = map_point + pos.cast();

					let map_point = map.get_mut(map_point.tuple()).unwrap();

					*map_point = self.shape_map()
						.get(self_point.tuple())
						.unwrap()
						.clone();
				}
			}
		}

		self.sub_slots.push(Slot {
			name,
			kind,
			size,
			pos,
			shape_map: map,
			sub_slots: vec![],
		})
	}

	fn is_path_available(&self, path: String, pos: Point, size: Bounds) -> bool {
		let (sub_slot_name, tail) = split_first_token(path);
		let tail = match tail {
			Some(tail) => tail,
			None => "".to_string(),
		};

		if sub_slot_name.len() > 0 {
			// FOLLOW THE PATH
			for sub_slot in self.sub_slots() {
				if sub_slot.name.eq(&sub_slot_name) {
					let sub_slot_pos = sub_slot.pos();
					return sub_slot.is_path_available(tail, pos - sub_slot_pos, size);
				}
			}
			// IF NO  => CHECK IF IN BOUNDS vvv
		}
		// If path destination is this => CHECK IF IN BOUNDS
		let start = pos;
		let end = pos + size.cast::<i32>() - 1;

		is_point_in_bounds(start, self.size()) &&
			is_point_in_bounds(end, self.size())
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