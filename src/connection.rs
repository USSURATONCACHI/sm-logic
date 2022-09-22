use std::fmt::{Debug, Formatter};
use dyn_clone::DynClone;

use crate::util::Bounds;
use crate::util::Point;



/// `Connection` is an object that describes connection between two slots.
/// `Connection` creates a `Vec` of point-to-point connections between
/// two slots, based on their sizes.
pub trait Connection: DynClone + Debug {
	fn connect(&self, start: Bounds, end: Bounds) -> Vec<(Point, Point)>;

	fn chain(self: Box<Self>, virtual_slot: Option<Bounds>, other: Box<dyn Connection>) -> Box<dyn Connection> {
		todo!()
	}
}

dyn_clone::clone_trait_object!(Connection);