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
		let name = name.into();
		let pos_at = at.into();
		self.create_if_n_exists(&name);

		let (pos, _) = self.poses.get_mut(&name)
			.unwrap();

		*pos = Some(pos_at);
	}

	pub fn place_last<P>(&mut self, at: P)
		where P: Into<Point>
	{
		match self.last_scheme.clone() {
			None => panic!("No schemes were added to place (ManualPos::place_last)"),
			Some(name) => self.place(name, at),
		}
	}

	pub fn rotate<S, R>(&mut self, name: S, by: R)
		where S: Into<String>,
				R: Into<Rot>,
	{
		let name = name.into();
		let rot_by = by.into();
		self.create_if_n_exists(&name);

		let (_, rot) = self.poses.get_mut(&name)
			.unwrap();

		*rot = rot_by.apply_to_rot(rot.clone());
	}

	pub fn rotate_last<R>(&mut self, by: R)
		where R: Into<Rot>,
	{
		match self.last_scheme.clone() {
			None => panic!("No schemes were added to place (ManualPos::place_last)"),
			Some(name) => self.rotate(name, by),
		}
	}

	fn create_if_n_exists(&mut self, name: &String) {
		if self.poses.get(name).is_none() {
			self.poses.insert(
				name.to_string(),
				(None, Rot::new(0, 0, 0))
			);
		}
	}
}

impl Positioner for ManualPos {
	fn set_last_scheme(&mut self, scheme_name: String) {
		self.last_scheme = Some(scheme_name);
	}

	fn arrange(self, _schemes: HashMap<String, Scheme>) -> Vec<(Point, Rot, Scheme)> {
		todo!()
	}
}