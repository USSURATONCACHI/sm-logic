use std::collections::HashMap;
use crate::connection::{ConnDim, Connection, ConnStraight};
use crate::slot::Slot;
use crate::util::{Bounds, Point};

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
	pub fn compile(self, schemes: HashMap<String, &Vec<Slot>>) -> Slot {
		todo!()
	}
}

#[derive(Debug, Clone)]
struct BasicBind {
	sector_corner: Point,
	sector_size: Bounds,
	target: String,
	conn: Box<dyn Connection>
}