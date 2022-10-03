use std::collections::HashMap;
use crate::combiner::SlotSide;
use crate::connection::{ConnDim, Connection, ConnStraight};
use crate::scheme;
use crate::slot::{Slot, SlotSector};
use crate::util::{Bounds, is_point_in_bounds, Map3D, Point, split_first_token};


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
pub struct Bind {
	name: String,
	kind: String,
	size: Bounds,

	maps: Vec<BasicBind>,
}

impl Bind {
	pub fn new<S1, S2, B>(slot_name: S1, slot_kind: S2, bounds: B) -> Self
		where S1: Into<String>,
			  S2: Into<String>,
			  B: Into<Bounds>
	{
		Bind {
			name: slot_name.into(),
			kind: slot_kind.into(),
			size: bounds.into(),

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
}

impl Bind {
	pub fn custom<P>(&mut self, sector: (Point, Bounds), target: P, conn: Box<dyn Connection>) -> &mut Self
		where P: Into<String>
	{
		self.maps.push(BasicBind {
			sector_corner: sector.0,
			sector_size: sector.1,
			target: target.into(),
			conn
		});
		self
	}

	pub fn connect<P, Pt, B>(&mut self, sector: (Pt, B), target: P) -> &mut Self
		where P: Into<String>, Pt: Into<Point>, B: Into<Bounds>
	{
		let sector = (sector.0.into(), sector.1.into());
		self.custom(sector, target, ConnStraight::new())
	}

	pub fn dim<P>(&mut self, sector: (Point, Bounds), target: P, adapt_axed: (bool, bool, bool)) -> &mut Self
		where P: Into<String>
	{
		self.custom(
			sector,
			target,
			ConnDim::new(adapt_axed)
		)
	}

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

	pub fn connect_full<P>(&mut self, target: P) -> &mut Self
		where P: Into<String>
	{
		let bounds = self.size.clone();
		self.connect(
			(Point::new_ng(0, 0, 0), bounds),
			target
		)
	}

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
					.map(|(from, to)| (from + sector.sector_corner, to  + slot_sector.pos))
					.collect(),

				SlotSide::Output => sector.conn
					.connect(slot_sector.bounds, sector.sector_size)
					.into_iter()
					.map(|(from, to)| (to + slot_sector.pos, from + sector.sector_corner))
					.collect(),
			};

			for (from_this, to_slot) in p2p_conns {
				if !is_point_in_bounds(from_this, sector.sector_size) ||
					!is_point_in_bounds(to_slot, slot_sector.bounds) {
					continue;
				}

				let to_slot_shapes = slot.get_point(to_slot)
					.unwrap()
					.iter()
					.map(|controller_id| controller_id + start_shape);

				map.get_mut(from_this.cast().tuple())
					.unwrap()
					.extend(to_slot_shapes)
			}
		}

		let slot = Slot::new(self.name, self.kind, self.size, map);
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