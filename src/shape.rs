use std::fmt::Debug;
use dyn_clone::DynClone;
use json::JsonValue;
use crate::util::{Bounds, GateMode, TICKS_PER_SECOND};

pub trait ShapeBase: DynClone + Debug {
	fn build(&self, out_conns: &Vec<usize>, color: &Option<String>) -> JsonValue;

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

	pub fn build(&self) -> JsonValue {
		self.base.build(&self.out_conns, &self.color)
	}
}

#[derive(Debug, Clone)]
pub struct Gate {
	mode: GateMode,
}

impl Gate {
	pub fn new(mode: GateMode) -> Shape {
		Shape::new(
			Box::new(
				Gate {
					mode
				}
			)
		)
	}
}

impl ShapeBase for Gate {
	fn build(&self, out_conns: &Vec<usize>, color: &Option<String>) -> JsonValue {
		todo!()
	}

	fn size(&self) -> Bounds {
		Bounds::new_ng(1, 1, 1)
	}

	fn has_input(&self) -> bool {
		true
	}

	fn has_output(&self) -> bool {
		true
	}
}


#[derive(Debug, Clone)]
pub struct Timer {
	seconds: u32,
	ticks: u32,
}

impl Timer {
	pub fn new(ticks: u32) -> Shape {
		Shape::new(
			Box::new(
				Timer {
					seconds: ticks / TICKS_PER_SECOND,
					ticks: ticks / TICKS_PER_SECOND,
				}
			)
		)
	}

	pub fn from_time(seconds: u32, ticks: u32) -> Shape {
		Shape::new(
			Box::new(
				Timer {
					seconds,
					ticks
				}
			)
		)
	}
}

impl ShapeBase for Timer {
	fn build(&self, out_conns: &Vec<usize>, color: &Option<String>) -> JsonValue {
		todo!()
	}

	fn size(&self) -> Bounds {
		Bounds::new_ng(1, 1, 2)
	}

	fn has_input(&self) -> bool {
		true
	}

	fn has_output(&self) -> bool {
		true
	}
}