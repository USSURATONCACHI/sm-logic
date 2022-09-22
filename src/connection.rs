use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::ops::Range;
use std::sync::Arc;
use dyn_clone::DynClone;

use crate::util::{Bounds, is_point_in_bounds};
use crate::util::Point;

/// `Connection` is an object that describes connection between two slots.
/// `Connection` creates a `Vec` of point-to-point connections between
/// two slots, based on their sizes.
pub trait Connection: DynClone + Debug {
	fn connect(&self, start: Bounds, end: Bounds) -> Vec<(Point, Point)>;

	fn chain(self: Box<Self>, virtual_slot: Option<Bounds>, other: Box<dyn Connection>) -> Box<dyn Connection>;
}

dyn_clone::clone_trait_object!(Connection);

/// Creates simple one-to-one points matching connection
/// between two slots. If slots' sizes differs it chooses smallest ones.
/// # Example
/// ```
/// let connection = ConnStraight::new();
///	let slot_size = Bounds::new(5, 5, 5);
/// let vectors = connection.connect(slot_size, slot_size);
/// ```
#[derive(Debug, Clone)]
pub struct ConnStraight {}

impl ConnStraight {
	pub fn new() -> Box<ConnStraight> {
		Box::new(ConnStraight {})
	}
}

impl Connection for ConnStraight {
	fn connect(&self, start: Bounds, end: Bounds) -> Vec<(Point, Point)> {
		let size_x = if start.x() < end.x() { *start.x() } else { *end.x() };
		let size_y = if start.y() < end.y() { *start.y() } else { *end.y() };
		let size_z = if start.z() < end.z() { *start.z() } else { *end.z() };

		let mut connections: Vec<(Point, Point)> = Vec::new();

		// Just straight one-to-one mapping
		for x in 0..size_x {
			for y in 0..size_y {
				for z in 0..size_z {
					let point = Point::new(x as i32, y as i32, z as i32);
					connections.push((point, point));
				}
			}
		}

		connections
	}

	fn chain(self: Box<Self>, virtual_slot: Option<Bounds>, other: Box<dyn Connection>) -> Box<dyn Connection> {
		ConnJoint::new(self).chain(virtual_slot, other)
	}
}

/// Joints other `Connection`s into a chain with a possibility
/// of changing `Slot` bounds in between. Behaves just as normal
/// `Connection`.
/// # Example
/// ```
/// let conn_1 = ConnStraight::new();	// you can put any connections
/// let conn_2 = ConnMap::new(|(point, _), _| Some(point * 2));
/// let slot_between_2_3 = Bounds::new_ng(5, 10, 15);
/// let conn_3 = ConnDim::new((false, true, false));
///
/// // This united connection will now behave just as three
/// // applied one after another
/// let united = ConnJoint::new(conn_1)
/// 	.chain(None, conn_2)
/// 	.chain(slot_between_2_3, conn_3);
/// ```
#[derive(Debug, Clone)]
pub struct ConnJoint {
	connections: Vec<(Option<Bounds>, Box<dyn Connection>)>,
}

impl ConnJoint {
	pub fn new(connection: Box<dyn Connection>) -> Box<ConnJoint> {
		Box::new(
			ConnJoint {
				connections: vec![(None, connection)],
			}
		)
	}
}

impl Connection for ConnJoint {
	fn connect(&self, start: Bounds, end: Bounds) -> Vec<(Point, Point)> {
		let mut start_bounds = start;
		let mut end_bounds: Bounds;

		// INITIALIZING START VECTORS
		let mut prev_vectors = ConnStraight::new().connect(start, start);

		for i in 0..self.connections.len() {
			// CONNECTION START/END BOUNDS CHECKING
			let (bounds, connection) = &self.connections[i];

			if bounds.is_some() {
				start_bounds = bounds.clone().unwrap();
			}

			let next = self.connections.get(i + 1);
			end_bounds = match next {
				Some((virt_slot, _)) => match virt_slot {
					Some(bounds) => bounds.clone(),
					None => start_bounds,
				},
				None => end,
			};

			// PROCESSING CONNECTION
			let vectors = connection.connect(start_bounds, end_bounds);
			let mut new_vectors: Vec<(Point, Point)> = Vec::new();

			for prev_vec in prev_vectors {
				for vec in &vectors {
					if prev_vec.1 == vec.0 && is_point_in_bounds(vec.1, end_bounds){
						new_vectors.push((prev_vec.0, vec.1));
					}
				}
			}

			prev_vectors = new_vectors;
			start_bounds = end_bounds;
		}

		// REMOVING DUPLICATES
		prev_vectors.sort_by(compare_two_vec_pairs);
		prev_vectors.dedup();

		prev_vectors
	}

	fn chain(mut self: Box<Self>, virtual_slot: Option<Bounds>, other: Box<dyn Connection>) -> Box<dyn Connection> {
		self.connections.push((virtual_slot, other));
		self
	}
}

fn compare_two_vec_pairs(a: &(Point, Point), b: &(Point, Point)) -> Ordering {
	// I hate this function so much
	let (a_start, a_end) = a;
	let (b_start, b_end) = b;

	if a_start.x() < b_start.x() {
		return Ordering::Less;
	} else if a_start.x() > b_start.x() {
		return Ordering::Greater;
	}

	if a_start.y() < b_start.y() {
		return Ordering::Less;
	} else if a_start.y() > b_start.y() {
		return Ordering::Greater;
	}

	if a_start.z() < b_start.z() {
		return Ordering::Less;
	} else if a_start.z() > b_start.z() {
		return Ordering::Greater;
	}

	if a_end.x() < b_end.x() {
		return Ordering::Less;
	} else if a_end.x() > b_end.x() {
		return Ordering::Greater;
	}

	if a_end.y() < b_end.y() {
		return Ordering::Less;
	} else if a_end.y() > b_end.y() {
		return Ordering::Greater;
	}

	if a_end.z() < b_end.z() {
		return Ordering::Less;
	} else if a_end.z() > b_end.z() {
		return Ordering::Greater;
	}

	return Ordering::Equal;
}

/// Connection that "ignores" specified dimensions of end `Slot`.
/// All `Slot`'s points that are laid on ignored/adapted axis will be
/// treated as the equal points and so, all of them will have the same
/// output/input vectors (depends on `Slot`)
///
/// # Example
/// If one of our slots is, for example, 3D: 15x by 15y by 30z points
/// size;
///
/// and the other one is 2D: 15x by 1y by 30z points size. (Is flat by
/// Y axis)
///
/// We can "adapt"/"ignore" Y axis using `ConnDim` type of connections.
/// That will connect each of 15 points of 3D `Slot` to the only
/// corresponding point on 2D `Slot`.
/// ```
/// let start = Bounds::new_ng(15, 15, 30);
/// let end = Bounds::new_ng(15, 1, 30);
/// let conn = ConnDim::new((false, true, false));
/// // conn.connect(start, end) - put where needed
/// ```
///
/// # What, if both slots are not flat at ignored axis
/// Then EACH point on ignored axis on one slot will be connected
/// to EACH point on another slot on that axis.
#[derive(Debug, Clone)]
pub struct ConnDim {
	adapt_x: bool,
	adapt_y: bool,
	adapt_z: bool,
}

impl ConnDim {
	#[allow(dead_code)]		// TODO add usage
	pub fn new(adapt_axes: (bool, bool, bool)) -> Box<ConnDim> {
		Box::new(
			ConnDim {
				adapt_x: adapt_axes.0,
				adapt_y: adapt_axes.1,
				adapt_z: adapt_axes.2,
			}
		)
	}

	fn ranges(&self, start: Point, end: Bounds) -> (Range<i32>, Range<i32>, Range<i32>) {
		let (x, y, z) = start.tuple();

		let x_range = if self.adapt_x {
			0..(*end.x() as i32)
		} else {
			x..(x + 1)
		};

		let y_range = if self.adapt_y {
			0..(*end.y() as i32)
		} else {
			y..(y + 1)
		};

		let z_range = if self.adapt_z {
			0..(*end.z() as i32)
		} else {
			z..(z + 1)
		};

		(x_range, y_range, z_range)
	}
}

impl Connection for ConnDim {
	fn connect(&self, start: Bounds, end: Bounds) -> Vec<(Point, Point)> {
		let mut vectors: Vec<(Point, Point)> = Vec::new();

		for x_start in 0..(*start.x() as i32) {
			for y_start in 0..(*start.y() as i32) {
				for z_start in 0..(*start.z() as i32) {
					let (x_range, y_range, z_range) = self
						.ranges(Point::new(x_start, y_start, z_start), end);

					for x_end in x_range {
						for y_end in y_range.clone() {
							for z_end in z_range.clone() {
								vectors.push((
									Point::new(
										x_start,
										y_start,
										z_start
									),
									Point::new(
										x_end,
										y_end,
										z_end
									)
								));
							}
						}
					}
				}
			}
		}

		vectors
	}

	fn chain(self: Box<Self>, virtual_slot: Option<Bounds>, other: Box<dyn Connection>) -> Box<dyn Connection> {
		ConnJoint::new(self).chain(virtual_slot, other)
	}
}


/// Filters point-to-point connections of other `Connection`.
///
/// # Example
/// ```
/// let old_conn = ConnStraight::new();
/// // Filters all connections that starts at odd x coordinate
/// let conn = ConnFilter::new(old_conn,
/// 		|start, end| (*start.x() % 2) == 0
/// 	);
/// ```
#[derive(Clone)]
pub struct ConnFilter {
	connection: Box<dyn Connection>,
	function: Arc<dyn Fn(&Point, &Point) -> bool>
}

impl ConnFilter {
	#[allow(dead_code)]		// TODO add usage
	pub fn new<F>(connection: Box<dyn Connection>, function: F) -> Box<ConnFilter>
		where F: Fn(&Point, &Point) -> bool + 'static
	{
		Box::new(
			ConnFilter {
				connection,
				function: Arc::new(function)
			}
		)
	}

	#[allow(dead_code)]		// TODO add usage
	pub fn from_arc(connection: Box<dyn Connection>, function: Arc<dyn Fn(&Point, &Point) -> bool>) -> Box<ConnFilter>
	{
		Box::new(
			ConnFilter {
				connection,
				function,
			}
		)
	}
}

impl Connection for ConnFilter {
	fn connect(&self, start: Bounds, end: Bounds) -> Vec<(Point, Point)> {
		let vectors = self.connection.connect(start, end);

		vectors.into_iter().filter(
			|(start, end)|
				(*self.function)(start, end)
		).collect()
	}

	fn chain(self: Box<Self>, virtual_slot: Option<Bounds>, other: Box<dyn Connection>) -> Box<dyn Connection> {
		ConnJoint::new(self).chain(virtual_slot, other)
	}
}

impl Debug for ConnFilter {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "ConnFilter {{ connection: {:?}, function: Arc {{?}} }}", self.connection)
	}
}

/// Maps each point of start `Slot` to points of end `Slot` via given
/// function.
///
/// # Example
/// ```
/// // Here each point of the start of connection will be connected
/// // to point with doubled coordinates
/// let conn = ConnMap::new(|(point, _), _| Some(point * 2));
/// ```
#[derive(Clone)]
pub struct ConnMap {
	function: Arc<dyn Fn((Point, Bounds), Bounds) -> Option<Point>>,
}

impl ConnMap {
	/// Argument is: Fn((start point, start bounds), end bounds) -> Option<end point>
	#[allow(dead_code)]		// TODO add usage
	pub fn new<F>(function: F) -> Box<ConnMap>
		where F: Fn((Point, Bounds), Bounds) -> Option<Point> + 'static
	{
		Box::new(
			ConnMap {
				function: Arc::new(function)
			}
		)
	}

	/// Argument is: Fn((start point, start bounds), end bounds) -> Option<end point>
	#[allow(dead_code)]		// TODO add usage
	pub fn from_arc(function: Arc<dyn Fn((Point, Bounds), Bounds) -> Option<Point>>) -> Box<ConnMap>
	{
		Box::new( ConnMap { function } )
	}
}

impl Connection for ConnMap {
	fn connect(&self, start: Bounds, end: Bounds) -> Vec<(Point, Point)> {
		let mut vectors: Vec<(Point, Point)> = Vec::new();

		for x in 0..(*start.x() as i32) {
			for y in 0..(*start.y() as i32) {
				for z in 0..(*start.z() as i32) {
					let start_point = Point::new(x, y, z);
					match (*self.function)((start_point, start), end) {
						Some(end_point) => vectors.push((start_point, end_point)),
						None => {}
					}
				}
			}
		}

		vectors
	}

	fn chain(self: Box<Self>, virtual_slot: Option<Bounds>, other: Box<dyn Connection>) -> Box<dyn Connection> {
		ConnJoint::new(self).chain(virtual_slot, other)
	}
}

impl Debug for ConnMap {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "ConnMap {{?}}")
	}
}