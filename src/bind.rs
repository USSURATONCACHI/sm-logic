use std::collections::HashMap;
use crate::combiner::SlotSide;
use crate::connection::{ConnDim, Connection, ConnStraight};
use crate::scheme;
use crate::slot::{Slot, SlotSector};
use crate::util::{Bounds, is_point_in_bounds, Map3D, Point, split_first_token};

/// Invalid connection wrapper.
#[derive(Debug, Clone)]
pub enum InvalidConn {
	TargetSchemeDoesNotExists {
		sector: BasicBind,
		target_scheme: String,
	},

	TargetSlotDoesNotExists {
		sector: BasicBind,
		target_slot: String,
	},

	TargetSlotSectorDoesNotExists {
		sector: BasicBind,
		target_slot: String,
		target_slot_sector: String,
	}
}

#[derive(Debug, Clone)]
pub enum SectorError {
	NameIsAlreadyTaken {
		taken_name: String,
	},

	SectorIsOutOfSlotBounds {
		sector_name: String,
		sector_pos: Point,
		sector_bounds: Bounds,
		slot_bounds: Bounds,
	}
}

/// Bind is just [`Slot`] builder.
///
/// It is used to create Slots conveniently.
///
/// # Example
/// ```
/// # use crate::sm_logic::bind::Bind;
/// # use crate::sm_logic::combiner::Combiner;
/// # let mut combiner = Combiner::pos_manual();
/// let mut bind = Bind::new("slot name", "slot kind", (10, 15, 20));
/// // Connects the whole slot to some other slot
/// bind.connect_full("some scheme/some other slot");
/// // Connect one point of the slot to some specific slot
/// bind.connect(((0, 0, 0), (1, 1, 1)), "specific_logic_gate");
/// // Create sector of the slot
/// bind.add_sector("specific_trigger", (0, 0, 0), (1, 1, 1), "logic").unwrap();
///
/// combiner.bind_input(bind).unwrap();
/// // or bind_output
/// ```
#[derive(Debug, Clone)]
pub struct Bind {
	name: String,
	kind: String,
	size: Bounds,

	sectors: Vec<(String, Point, Bounds, String)>,
	maps: Vec<BasicBind>,
}

impl Bind {
	/// Creates new empty [`Bind`]
	/// # Example
	/// ```
	/// # use crate::sm_logic::bind::Bind;
	/// # use crate::sm_logic::combiner::Combiner;
	/// # let mut combiner = Combiner::pos_manual();
	/// let mut bind = Bind::new("slot name", "slot kind", (10, 15, 20));
	/// combiner.bind_input(bind).unwrap();
	/// // or bind_output
	/// ```
	pub fn new<S1, S2, B>(slot_name: S1, slot_kind: S2, bounds: B) -> Self
		where S1: Into<String>,
			  S2: Into<String>,
			  B: Into<Bounds>
	{
		Bind {
			name: slot_name.into(),
			kind: slot_kind.into(),
			size: bounds.into(),

			sectors: vec![],
			maps: vec![],
		}
	}

	pub fn name(&self) -> &String {
		&self.name
	}

	pub fn kind(&self) -> &String {
		&self.kind
	}

	pub fn bounds(&self) -> Bounds {
		self.size.clone()
	}

	/// Adds sector to the Bind (Slot)
	///
	/// # Example
	/// ```
	/// # use crate::sm_logic::bind::Bind;
	/// # let mut bind = Bind::new("slot name", "slot kind", (10, 15, 20));
	/// // Create sector of the slot
	/// bind.add_sector("specific_trigger", (0, 0, 0), (1, 1, 1), "logic").unwrap();
	/// ```
	pub fn add_sector<S1, P, B, S2>(&mut self, name: S1, corner: P, bounds: B, kind: S2) -> Result<(), SectorError>
		where S1: Into<String>, S2: Into<String>,
				P: Into<Point>, B: Into<Bounds>,
	{
		let name = name.into();
		for (added_name, _, _, _) in &self.sectors {
			if added_name.eq(&name) {
				return Err(
					SectorError::NameIsAlreadyTaken {
						taken_name: name,
					}
				)
			}
		}

		let corner = corner.into();
		let bounds = bounds.into();

		let start = corner;
		let end: Point = start + bounds.cast() - Point::new_ng(1_i32, 1, 1);

		if !is_point_in_bounds(start, self.bounds()) ||
			!is_point_in_bounds(end, self.bounds()) {
			return Err(
				SectorError::SectorIsOutOfSlotBounds {
					sector_name: name,
					sector_pos: corner,
					sector_bounds: bounds,
					slot_bounds: self.bounds(),
				}
			)
		}

		self.sectors.push((name, corner, bounds, kind.into()));
		Ok(())
	}

	/// Generates a sector for each point of slot with given names.
	///
	/// # Example
	/// ```
	/// # use crate::sm_logic::bind::Bind;
	/// # let mut bind = Bind::new("slot name", "slot kind", (10, 10, 1));
	/// bind.gen_point_sectors("logic", |x, y, z| format!("{}_{}", x, y)).unwrap();
	/// ```
	pub fn gen_point_sectors<S1, S2, F>(&mut self, kind: S1, names: F) -> Result<(), Vec<SectorError>>
		where S1: Into<String>,
			  S2: Into<String>,
			  F: Fn(u32, u32, u32) -> S2,
	{
		let (size_x, size_y, size_z) = self.bounds().tuple();
		let kind = kind.into();
		let mut errors: Vec<SectorError> = vec![];

		for x in 0..size_x {
			for y in 0..size_y {
				for z in 0..size_z {
					let res = self.add_sector(
						names(x, y, z),
						(x as i32, y as i32, z as i32),
						(1, 1, 1),
						kind.clone()
					);

					match res {
						Ok(()) => {}
						Err(e) => errors.push(e),
					}
				}
			}
		}

		if errors.len() == 0 {
			Ok(())
		} else {
			Err(errors)
		}
	}
}

impl Bind {
	/// Connects some part (sector) of the slot with custom connection (`conn` argument)
	///
	/// # Example
	/// ```
	/// # use sm_logic::connection::ConnStraight;
	/// # use crate::sm_logic::bind::Bind;
	/// # let mut bind = Bind::new("slot name", "slot kind", (10, 15, 20));
	/// bind.custom(((0, 0, 0), (1, 1, 1)), "path/to/slot or sector", ConnStraight::new());
	/// ```
	pub fn custom<Pt, B, P>(&mut self, sector: (Pt, B), target: P, conn: Box<dyn Connection>) -> &mut Self
		where P: Into<String>, Pt: Into<Point>, B: Into<Bounds>
	{
		self.maps.push(BasicBind {
			sector_corner: sector.0.into(),
			sector_size: sector.1.into(),
			target: target.into(),
			conn
		});
		self
	}
	/// Connects some part (sector) of the slot with straight connection
	/// (ConnStraight)
	///
	/// # Example
	/// ```
	/// # use sm_logic::connection::ConnStraight;
	/// # use crate::sm_logic::bind::Bind;
	/// # let mut bind = Bind::new("slot name", "slot kind", (10, 15, 20));
	/// bind.custom(((0, 0, 0), (1, 1, 1)), "path/to/slot or sector", ConnStraight::new());
	/// // these two lines are equal
	/// bind.connect(((0, 0, 0), (1, 1, 1)), "path/to/slot or sector");
	/// ```
	pub fn connect<P, Pt, B>(&mut self, sector: (Pt, B), target: P) -> &mut Self
		where P: Into<String>, Pt: Into<Point>, B: Into<Bounds>
	{
		let sector = (sector.0.into(), sector.1.into());
		self.custom(sector, target, ConnStraight::new())
	}

	/// Connects some part (sector) of the slot with [`ConnDim`] connection
	///
	/// # Example
	/// ```
	/// # use sm_logic::connection::ConnDim;
	/// # use crate::sm_logic::bind::Bind;
	/// # let mut bind = Bind::new("slot name", "slot kind", (10, 15, 20));
	/// bind.custom(((0, 0, 0), (1, 1, 1)), "path/to/slot or sector", ConnDim::new((true, true, false)));
	/// // these two lines are equal
	/// bind.dim(((0, 0, 0), (1, 1, 1)), "path/to/slot or sector", (true, true, false));
	/// ```
	pub fn dim<P, Pt, B>(&mut self, sector: (Pt, B), target: P, adapt_axed: (bool, bool, bool)) -> &mut Self
		where P: Into<String>, Pt: Into<Point>, B: Into<Bounds>
	{
		self.custom(
			sector,
			target,
			ConnDim::new(adapt_axed)
		)
	}

	/// Connects the whole slot with custom connection (`conn` argument)
	///
	/// # Example
	/// ```
	/// # use sm_logic::connection::ConnStraight;
	/// # use crate::sm_logic::bind::Bind;
	/// # let mut bind = Bind::new("slot name", "slot kind", (10, 15, 20));
	/// bind.custom(((0, 0, 0), bind.bounds()), "path/to/slot or sector", ConnStraight::new());
	///	// These two lines are equal
	/// bind.custom_full("path/to/slot or sector", ConnStraight::new());
	/// ```
	pub fn custom_full<P>(&mut self, target: P, conn: Box<dyn Connection>) -> &mut Self
		where P: Into<String>
	{
		let bounds = self.size.clone();
		self.custom(
			(Point::new_ng(0, 0, 0), bounds),
			target,
			conn
		)
	}
	/// Connects the whole slot with straight connection (ConnStraight)
	/// # Example
	/// ```
	/// # use sm_logic::connection::ConnStraight;
	/// # use crate::sm_logic::bind::Bind;
	/// # let mut bind = Bind::new("slot name", "slot kind", (10, 15, 20));
	/// bind.custom_full("path/to/slot or sector", ConnStraight::new());
	///	// These two lines are equal
	/// bind.connect_full("path/to/slot or sector");
	/// ```
	pub fn connect_full<P>(&mut self, target: P) -> &mut Self
		where P: Into<String>
	{
		let bounds = self.size.clone();
		self.connect(
			(Point::new_ng(0, 0, 0), bounds),
			target
		)
	}

	/// Connects the whole slot with [`ConnDim`] connection
	/// # Example
	/// ```
	/// # use sm_logic::connection::ConnDim;
	/// # use crate::sm_logic::bind::Bind;
	/// # let mut bind = Bind::new("slot name", "slot kind", (10, 15, 20));
	/// bind.custom_full("path/to/slot or sector", ConnDim::new((true, false, true)));
	///	// These two lines are equal
	/// bind.dim_full("path/to/slot or sector", (true, false, true));
	/// ```
	pub fn dim_full<P>(&mut self, target: P, axes: (bool, bool, bool)) -> &mut Self
		where P: Into<String>
	{
		let bounds = self.size.clone();
		self.dim(
			(Point::new_ng(0, 0, 0), bounds),
			target,
			axes
		)
	}

	/// Connects the whole slot using some map.
	/// Map must take slot abstract point position as input and return
	/// and option of other slot path, to connect that point into.
	///
	/// # Example
	/// ```
	/// # use sm_logic::connection::ConnDim;
	/// # use crate::sm_logic::bind::Bind;
	/// # let mut bind = Bind::new("slot name", "slot kind", (10, 15, 20));
	/// // Each point of the slot will be connected separately
	/// bind.connect_func( |x, y, z| Some(format!("gate_{}_{}_{}", x, y, z+1)) );
	/// ```
	pub fn connect_func<P, F>(&mut self, func: F) -> &mut Self
		where P: Into<String>,
			  F: Fn(usize, usize, usize) -> Option<P>
	{
		let bounds = self.size.try_cast::<usize>().unwrap().tuple();

		for x in 0..bounds.0 {
			for y in 0..bounds.1 {
				for z in 0..bounds.2 {
					let target = func(x, y, z);

					match target {
						None => continue,
						Some(target) => {
							let point: Point = Point::new(x as i32, y as i32, z as i32);
							self.connect(
								(point, Bounds::from_tuple((1u32, 1u32, 1u32))),
								target
							);
						}
					}
				}
			}
		};

		self
	}
}

impl Bind {
	// 										name, start shape, slots
	pub fn compile(self, schemes: &HashMap<String, (usize, Vec<Slot>)>, side: SlotSide)
		-> (Slot, Vec<InvalidConn>)
	{
		let mut map: Map3D<Vec<usize>> = Map3D::filled(self.bounds().cast().tuple(), vec![]);
		let mut errors: Vec<InvalidConn> = vec![];

		for sector in self.maps {
			let (start_shape, slot, slot_sector) =
				match compile_get_slot(&sector, schemes) {
					Err(e) => {
						errors.push(e);
						continue;
					}

					Ok(values) => values,
				};

			// Point-to-point
			let p2p_conns: Vec<(Point, Point)> = match side {
				SlotSide::Input => sector.conn
					.connect(sector.sector_size, slot_sector.bounds)
					.into_iter()
					.map(|(from, to)| (from, to))
					.collect(),

				SlotSide::Output => sector.conn
					.connect(slot_sector.bounds, sector.sector_size)
					.into_iter()
					.map(|(from, to)| (to, from))
					.collect(),
			};

			for (from_this, to_slot) in p2p_conns {
				if !is_point_in_bounds(from_this, sector.sector_size) ||
					!is_point_in_bounds(sector.sector_corner + from_this, self.size) ||
					!is_point_in_bounds(to_slot, slot_sector.bounds) ||
					!is_point_in_bounds(slot_sector.pos + to_slot, slot.bounds())
				{
					continue;
				}
				let from_this = from_this + sector.sector_corner;
				let to_slot = to_slot + slot_sector.pos;

				let to_slot_shapes = slot.get_point(to_slot)
					.unwrap()
					.iter()
					.map(|controller_id| controller_id + start_shape);

				map.get_mut(from_this.cast().tuple())
					.unwrap()
					.extend(to_slot_shapes)
			}
		}

		let mut slot = Slot::new(self.name, self.kind, self.size, map);

		for (name, pos, bounds, kind) in self.sectors {
			let sector = SlotSector { pos, bounds, kind };
			slot.bind_sector(name, sector).unwrap();
		}

		(slot, errors)
	}
}

fn compile_get_slot<'a>(sector: &BasicBind, schemes: &'a HashMap<String, (usize, Vec<Slot>)>)
	-> Result<(usize, &'a Slot, &'a SlotSector), InvalidConn>
{

	let target = sector.target.clone();
	let (target_scheme, slot) = split_first_token(target);
	let slot = match slot {
		None => "".to_string(),
		Some(slot) => slot,
	};

	let (slot_name, slot_sector) = split_first_token(slot);
	let slot_sector = match slot_sector {
		None => "".to_string(),
		Some(slot_sector) => slot_sector,
	};

	//println!("Target: '{}', Scheme: '{}', Slot: '{}', Sector: '{}'", target, target_scheme, slot_name, slot_sector);

	let (start_shape, slots) = match schemes.get(&target_scheme) {
		None => {
			return Err(InvalidConn::TargetSchemeDoesNotExists {
				sector: sector.clone(),
				target_scheme
			});
		},

		Some((start_shape, slots)) =>
			(*start_shape, slots)
	};

	let slot = match scheme::find_slot(&slot_name, slots) {
		None => {
			return Err(InvalidConn::TargetSlotDoesNotExists {
				sector: sector.clone(),
				target_slot: slot_name
			});
		}
		Some(slot) => slot,
	};

	match slot.get_sector(&slot_sector) {
		None => {
			Err(InvalidConn::TargetSlotSectorDoesNotExists {
				sector: sector.clone(),
				target_slot: slot_name,
				target_slot_sector: slot_sector,
			})
		}

		Some(sector) => {
			Ok((start_shape, slot, sector))
		}
	}
}

#[derive(Debug, Clone)]
pub struct BasicBind {
	sector_corner: Point,
	sector_size: Bounds,
	target: String,
	conn: Box<dyn Connection>
}