pub mod vanilla;

use std::fmt::Debug;
use dyn_clone::DynClone;
use json::{JsonValue, object};
use crate::scheme::{DEFAULT_SLOT, Scheme};
use crate::slot::Slot;

use crate::util::{Map3D, Point};
use crate::util::Rot;
use crate::util::Bounds;

// TODO: check actual xaxis zaxis
/// This trait describes all in-game blocks and parts.
///
/// -------------------
/// `size` method should return physical size of the part in the rotation of: `"xaxis": 0, "zaxis": 0`.
///
/// _`has_input`_ method should return if a part can have incoming connections plugged in (if other parts can be connected _INTO_ this one).
///
/// _`has_output`_ method should return if a other parts can have connections incoming from this one.
///
/// _`build`_ method should convert [`ShapeBase`] instance into `JsonValue`.
/// Examples: [`vanilla::Gate`], [`vanilla:Timer`], [`vanilla::BlockBody`]
pub trait ShapeBase: DynClone + Debug {
	fn build(&self, data: ShapeBuildData) -> JsonValue;

	fn size(&self) -> Bounds;
	fn has_input(&self) -> bool;
	fn has_output(&self) -> bool;
}
dyn_clone::clone_trait_object!(ShapeBase);

/// This struct is used to pass all data required to create JsonValue
/// from [`Shape`]
pub struct ShapeBuildData<'a> {
	/// All shapes ids ("controller ids"), to which shape is connected
	pub out_conns: &'a Vec<usize>,

	/// Shape can have set color. By default, `&Some(String)` means given
	/// color and `&None` means default Shape color.
	pub color: &'a Option<String>,

	/// Physical position of the [`Shape`]
	pub pos: Point,

	/// Physical rotation of the [`Shape`]
	pub rot: Rot,

	/// `"controller id"` of the Shape.
	pub id: usize,
}

/// Represents in-game blocks and parts. Can be connected to other
/// shapes (`out_conns`). Can be painted (`color`).
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

	/// Adds connection from this shape to given controller id
	pub fn push_conn(&mut self, controller_id: usize) {
		self.out_conns.push(controller_id);
	}

	/// Adds multiple connections. Is not meant to be used without
	/// context of other shapes with their own unique ids.
	///
	/// # Example
	/// ```
	/// # use sm_logic::shape::vanilla::GateMode;
	/// # use crate::sm_logic::shape::Shape;
	/// # use crate::sm_logic::shape::vanilla::Gate;
	/// let mut shape = Gate::new(GateMode::AND);
	/// // These 1, 2, 3 should represent other shapes
	/// shape.extend_conn([1, 2, 3]);
	///
	/// let other_conns: Vec<usize> = vec![4, 5, 6];
	/// shape.extend_conn(other_conns)
	/// ```
	pub fn extend_conn<I>(&mut self, controller_ids: I)
		where I: IntoIterator<Item = usize>
	{
		self.out_conns.extend(controller_ids);
	}

	/// Forces the color of the shape.
	pub fn set_color<S: Into<String>>(&mut self, color: S) {
		self.color = Some(color.into());
	}

	/// Mutable getter.
	pub fn connections_mut(&mut self) -> &mut Vec<usize> {
		&mut self.out_conns
	}

	/// Returns physical bounds of the shape.
	pub fn bounds(&self) -> Bounds {
		self.base.size()
	}

	pub fn has_input(&self) -> bool {
		self.base.has_input()
	}

	pub fn has_output(&self) -> bool {
		self.base.has_output()
	}

	/// Compiles shape to JSON
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

impl Into<Scheme> for Shape {
	fn into(self) -> Scheme {
		// Since there is only one shape, slot should be 1 by 1 by 1
		// And the only point of this slot should reference the shape.
		let slot_map: Map3D<Vec<usize>> = Map3D::filled((1, 1, 1), vec![0_usize]);
		let slot = Slot::new(
			DEFAULT_SLOT.to_string(),
			"logic".to_string(),
			Bounds::new_ng(1, 1, 1),
			slot_map.clone()
		);

		let input = if self.has_input() { vec![slot.clone()] }  else { vec![] };
		let output = if self.has_input() { vec![slot.clone()] }  else { vec![] };

		Scheme::create(
			vec![(Point::new_ng(0, 0, 0), Rot::new(0, 0, 0), self)],
			input, output,
		)
	}
}

/// Converts [`Vec`] of usize-s to Scrap Mechanic blueprint's JSON connections
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