use std::fmt::Debug;
use dyn_clone::DynClone;
use json::JsonValue;
use crate::util::Bounds;

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
	pub fn new<B: Into<Box<dyn ShapeBase>>>(base: B) -> Shape {
		Shape {
			base: base.into(),
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