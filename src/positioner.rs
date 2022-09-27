use std::collections::HashMap;
use std::fmt::Debug;
use crate::scheme::Scheme;
use crate::util::{Point, Rot};

pub trait Positioner: Debug + Clone {
	fn set_last_scheme(&mut self, scheme_name: String);
	fn arrange(self, schemes: HashMap<String, Scheme>) -> Vec<(Point, Rot, Scheme)>;
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ManualPos {
	poses: HashMap<String, (Option<Point>, Rot)>,
	last_scheme: Option<String>,
}

impl ManualPos {
	pub fn new() -> Self {
		ManualPos {
			poses: HashMap::new(),
			last_scheme: None
		}
	}

	pub fn place<S, P>(&mut self, name: S, at: P)
		where S: Into<String>,
				P: Into<Point>
	{
		todo!()
	}

	pub fn place_last<P>(&mut self, at: P)
		where P: Into<Point>
	{
		todo!()
	}

	pub fn rotate<S, R>(&mut self, name: S, by: R)
		where S: Into<String>,
				R: Into<Rot>,
	{
		todo!()
	}

	pub fn rotate_last<R>(&mut self, by: R)
		where R: Into<Rot>,
	{
		todo!()
	}
}

impl Positioner for ManualPos {
	fn set_last_scheme(&mut self, scheme_name: String) {
		self.last_scheme = Some(scheme_name);
	}

	fn arrange(self, schemes: HashMap<String, Scheme>) -> Vec<(Point, Rot, Scheme)> {
		todo!()
	}
}