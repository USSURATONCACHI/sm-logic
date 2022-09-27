pub mod vanilla;

use std::fmt::Debug;
use dyn_clone::DynClone;
use json::{JsonValue, object};

use crate::util::Point;
use crate::util::Rot;
use crate::util::Bounds;

pub struct ShapeBuildData<'a> {
	pub out_conns: &'a Vec<usize>,
	pub color: &'a Option<String>,
	pub pos: Point,
	pub rot: Rot,
	pub id: usize,
}

pub trait ShapeBase: DynClone + Debug {
	fn build(&self, data: ShapeBuildData) -> JsonValue;

	fn size(&self) -> Bounds;
	fn has_input(&self) -> bool;
	fn has_output(&self) -> bool;
}

dyn_clone::clone_trait_object!(ShapeBase);

#[derive(Debug, Clone)]
pub struct Shape {
	base: Box<dyn ShapeBase>,
	out_conns: Vec<usize>,
	color: Option<String>,
}

impl Shape {
	pub fn new(base: Box<dyn ShapeBase>) -> Shape {
		Shape {
			base,
			out_conns: Vec::new(),
			color: None,
		}
	}

	pub fn push_conn(&mut self, controller_id: usize) {
		self.out_conns.push(controller_id);
	}

	pub fn extend_conn<I>(&mut self, controller_ids: I)
		where I: IntoIterator<Item = usize>
	{
		self.out_conns.extend(controller_ids);
	}

	pub fn size(&self) -> Bounds {
		self.base.size()
	}

	pub fn has_input(&self) -> bool {
		self.base.has_input()
	}

	pub fn has_output(&self) -> bool {
		self.base.has_output()
	}

	pub fn build(&self, pos: Point, rot: Rot, id: usize) -> JsonValue {
		let data = ShapeBuildData {
			out_conns: &self.out_conns,
			color: &self.color,
			pos,
			rot,
			id
		};

		self.base.build(data)
	}
}

pub fn out_conns_to_controller(out_conns: &Vec<usize>) -> JsonValue {
	if out_conns.len() > 0 {
		let vals: Vec<JsonValue> = out_conns.iter()
			.map(|id| object!{ "id": *id })
			.collect();

		JsonValue::Array(vals)
	} else {
		JsonValue::Null
	}
}