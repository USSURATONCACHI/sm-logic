use std::collections::HashMap;
use std::fmt::Debug;
use crate::positioner::ManualPosError::{SchemeHasNoPosition, SchemeIsNotPlaced};
use crate::scheme::Scheme;
use crate::util::{Point, Rot};

/// `Positioner` is an object, that gives each `Combiner`'s scheme a
/// position.
///
/// This is done using traits for customization possibilities.
///
/// Right now only [`ManualPos`] is implemented.
/// But I or you can create some logic to distribute `Scheme`s
/// automatically. Or pretty much any other position logic.
pub trait Positioner: Debug + Clone {
	type Error: Debug;

	/// This function is called by `Combiner` and the name of the last
	/// added `Scheme` is passed.
	fn set_last_scheme(&mut self, scheme_name: String);

	/// Converts HashMap<String, Scheme> to HashMap<String, (Point, Rot, Scheme)> -
	/// assigns physical positions and rotations to each of the schemes.
	fn arrange(self, schemes: HashMap<String, Scheme>) -> Result<HashMap<String, (Point, Rot, Scheme)>, Self::Error>;
}

/// [`Positioner`] with fully manual position management.
/// If any of the schemes is not positioned - error will be returned.
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

	/// Places scheme with equal name to the given position.
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

	/// Places multiple schemes to the given position.
	/// '`Pairs`' should be an iterable of pairs in format
	/// '`(name, position)`'
	pub fn place_iter<I, S, P>(&mut self, pairs: I)
		where S: Into<String>, P: Into<Point>, I: IntoIterator<Item = (S, P)>
	{
		for (scheme, pos) in pairs {
			self.place(scheme, pos);
		}
	}

	/// Places last added scheme to the given position. If no schemes
	/// were added before - panics.
	pub fn place_last<P>(&mut self, at: P)
		where P: Into<Point>
	{
		match self.last_scheme.clone() {
			None => panic!("No schemes were added to place (ManualPos::place_last)"),
			Some(name) => self.place(name, at),
		}
	}

	/// Rotates given scheme by given angle ([`Rot`])
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

	/// Applies self.rotate method to each pair of (name, rot) pairs.
	pub fn rotate_iter<I, S, R>(&mut self, pairs: I)
		where S: Into<String>, R: Into<Rot>, I: IntoIterator<Item = (S, R)>
	{
		for (scheme, rot) in pairs {
			self.rotate(scheme, rot);
		}
	}

	/// Rotates last added scheme by given angle ([`Rot`]). If no
	/// schemes were added - panics.
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

#[derive(Clone, Debug)]
pub enum ManualPosError {
	SchemeIsNotPlaced { name: String },
	SchemeHasNoPosition { name: String },
}

impl Positioner for ManualPos {
	type Error = ManualPosError;

	fn set_last_scheme(&mut self, scheme_name: String) {
		self.last_scheme = Some(scheme_name);
	}

	fn arrange(self, schemes: HashMap<String, Scheme>) -> Result<HashMap<String, (Point, Rot, Scheme)>, Self::Error> {
		let mut posed_schemes: HashMap<String, (Point, Rot, Scheme)> = HashMap::new();

		for (name, scheme) in schemes {
			match self.poses.get(&name) {
				None => return Err(SchemeIsNotPlaced { name }),

				Some((pos, rot)) =>
					match pos {
						None => return Err(SchemeHasNoPosition { name }),

						Some(pos) => {
							posed_schemes.insert(name, (pos.clone(), rot.clone(), scheme));
						},
					}
			}
		}

		Ok(posed_schemes)
	}
}