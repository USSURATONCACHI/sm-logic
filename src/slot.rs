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
pub struct SlotSector {
	pub pos: Point,
	pub bounds: Bounds,
	pub kind: String,
}

/// # About
///
/// Slot is `Scheme` interface to connect `Scheme` to other schemes.
/// It maps out abstract Slot space to actual shapes stored in `Scheme`.
///
/// # Abstract slot space
///
/// Slot has an abstract 3D space of its size. Each point can transfer
/// data. Using `Connection`, slot's space can be connected
/// to the space of another slot, point-to-point.
///
/// Each point can be connected to any amount of points of another Slot.
/// On the more real side, each abstract point can represent any count
/// of real shapes.
///
/// <br>
///
/// You can imagine it as abstract cube of logic gates, that can be
/// connected to another abstract cube of logic gates, gate-to-gate.
/// Output shapes are connected into output cube, output cube is
/// connected to input cube in some way, and input cube is connected
/// into real input shapes.
///
/// Then we delete this 'abstract wrapper' and the only thing left is
/// pure shape-to-shape in-game connections.
///
/// <br>
///
/// Connection is usually done in combiner this way:
/// ```
/// # use sm_logic::combiner::Combiner;
/// # let mut combiner = Combiner::pos_manual();
/// combiner.connect("<scheme name>/<output slot name>", "<scheme name>/<input_slot_name>");
/// // or
/// combiner.connect(
/// 	"<scheme>/<slot>/<optional: slot sector name>",
/// 	"<scheme>/<slot>/<optional: slot sector name>"
/// );
/// ```
///
/// # What if I connect slots with different sizes?
///
/// Well, nothing special. All point-to-point connections, which point
/// from nothing or to nothing will be discarded.
///
/// An so, if your Connection weirdly warps point-to-point connections,
/// only valid will remain.
/// <br>
///
/// # Sectors
///
/// Slot sector is the part of the slot area that can be used just as
/// the slot itself. Size of sector cannot be bigger, than the size
/// of the slot.
///
/// <br>
/// Why this can be useful (abstract example):
/// If slot, for example, carries two binary numbers and you want to
/// only connect the part with the first one. Instead of connecting the
/// whole slot with some mapping connection:
///
/// ```
/// # use crate::sm_logic::combiner::Combiner;
/// # use crate::sm_logic::connection::ConnMap;
/// # use crate::sm_logic::util::Point;
/// # let mut combiner = Combiner::pos_manual();
/// // Lets imagine slot has one number at points (0..number_size, 0, 0)
/// // and second number at points (0..number_size, 0, 1).
/// // With this data layout we only need points with z = 0:
/// let filter = ConnMap::new(
/// 	|(point, _), _| if *point.z() == 0 { Some(point) } else { None }
/// );
///
/// combiner.custom("out_slot_scheme/slot", "target/slot", filter);
/// ```
///
/// We can add sector of slot when creating it and just connect to slot:
///
/// ```
/// # use crate::sm_logic::combiner::Combiner;
/// # let mut combiner = Combiner::pos_manual();
/// combiner.connect("out_slot_scheme/slot/first_number", "target/slot");
/// ```
///
/// Sector creation would probably look like this:
/// ```
/// # use crate::sm_logic::bind::Bind;
/// # use crate::sm_logic::combiner::Combiner;
/// # let mut combiner = Combiner::pos_manual();
/// let mut bind = Bind::new("slot", "two_numbers", (16, 2, 1));
///
/// // ... bind to some actual data ...
///	// then add sector
///	bind.add_sector("first_number", (0, 0, 0), (16, 1, 1), "binary").unwrap();
///	bind.add_sector("second_number", (0, 1, 0), (16, 1, 1), "binary").unwrap();
/// combiner.bind_input(bind).unwrap();
/// // or bind_output
/// ```
#[derive(Debug, Clone)]
pub struct Slot {
	/// Slot name, obviously
	name: String,

	/// Meaning of the slot and its data
	#[allow(dead_code)]		// Feature with Slot kinds and adaptors is planned
	kind: String,

	/// Size of the slot
	bounds: Bounds,

	/// Map of the abstract shape space to real shapes.
	shape_map: Map3D<Vec<usize>>,

	/// List of all sectors of Slot
	sectors: HashMap<String, SlotSector>,
}

impl Slot {
	pub fn name(&self) -> &String {
		&self.name
	}

	pub fn kind(&self) -> &String {
		&self.kind
	}

	pub fn bounds(&self) -> Bounds {
		self.bounds.clone()
	}

	pub fn shape_map(&self) -> &Map3D<Vec<usize>> {
		&self.shape_map
	}

	pub fn shape_map_mut(&mut self) -> &mut Map3D<Vec<usize>> {
		&mut self.shape_map
	}

	pub fn sectors(&self) -> &HashMap<String, SlotSector> {
		&self.sectors
	}

	pub fn sectors_mut(&mut self) -> &mut HashMap<String, SlotSector> {
		&mut self.sectors
	}

	/// Returns reference to vec of shapes, connected to specific point
	/// of abstract slot space.
	pub fn get_point(&self, pos: Point) -> Option<&Vec<usize>> {
		match pos.try_cast::<usize>() {
			Ok(pos) => self.shape_map.get(pos.tuple()),
			Err(_) => None,
		}
	}

	/// Returns basic data about the slot.
	pub fn base_data(&self) -> BaseSlotData {
		BaseSlotData {
			name: self.name.clone(),
			kind: self.kind.clone(),
			bounds: self.bounds.clone(),
		}
	}

	/// Creates slot from given data.
	pub fn new(name: String, kind: String, bounds: Bounds, shape_map: Map3D<Vec<usize>>) -> Self {
		Slot {
			name,
			kind: kind.clone(),
			bounds,
			shape_map,
			sectors: {
				// Sector with empty name is the slot itself
				let mut map = HashMap::new();
				map.insert("".to_string(), SlotSector {
					pos: Point::new_ng(0, 0, 0),
					bounds,
					kind,
				});
				map
			}
		}
	}

	/// Returns sector with such name, if it exists.
	pub fn get_sector(&self, name: &String) -> Option<&SlotSector> {
		self.sectors().get(name)
	}

	/// Adds sector.
	pub fn bind_sector(&mut self, name: String, sector: SlotSector) -> Result<(), SlotError> {
		if name.len() == 0 {
			return Err(SlotError::NameIsAlreadyTaken {
				main_slot_name: self.name.clone(),
				subject_name: name,
				comment: "Slot sector name cannot be zero-sized, \
					because zero sized name is already taken by the \
					slot itself".to_string(),
			})
		}

		// Check that there is no already existing slot with such name.
		if self.sectors().get(&name).is_some() {
			return Err(SlotError::NameIsAlreadyTaken {
				main_slot_name: self.name.clone(),
				subject_name: name,
				comment: "Sector with such name was added before".to_string(),
			});
		}

		self.sectors_mut().insert(name, sector);
		Ok(())
	}

	pub fn shape_was_removed(&mut self, id: usize) {
		for point in self.shape_map_mut().as_raw_mut() {
			let mut len = point.len();
			let mut i = 0;

			while i < len {
				if point[i] == id {
					point.remove(i);
					len -= 1;
				} else if point[i] > id {
					point[i] -= 1;
					i += 1;
				} else {
					i += 1;
				}
			}
		}
	}
}