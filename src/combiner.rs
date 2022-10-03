use std::collections::HashMap;
use std::fmt::Debug;
use crate::bind::{Bind, InvalidConn};
use crate::combiner::Error::{InvalidName, NameWasAlreadyTaken};
use crate::connection::{ConnDim, Connection, ConnStraight};
use crate::positioner::{ManualPos, Positioner};
use crate::scheme;
use crate::scheme::Scheme;
use crate::shape::Shape;
use crate::slot::{Slot, SlotSector};
use crate::util::{is_point_in_bounds, Point, Rot, split_first_token};

#[derive(Debug, Clone)]
pub struct InvalidActs {
	pub connections: Vec<ConnCase>,
	pub inp_bind_conns: Vec<(String, InvalidConn)>,
	pub out_bind_conns: Vec<(String, InvalidConn)>,
}

impl InvalidActs {
	pub fn new() -> Self {
		InvalidActs {
			connections: vec![],
			inp_bind_conns: vec![],
			out_bind_conns: vec![],
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum SlotSide {
	Input, Output
}

#[derive(Debug, Clone)]
pub enum Error {
	InvalidName {
		invalid_name: String,
		tip: String,
	},

	NameWasAlreadyTaken {
		taken_name: String,
		tip: String,
	},

	PassHasInvalidTarget {
		pass_name: String,
		pass_side: SlotSide,
		tip: String,
	},
}

#[derive(Debug, Clone)]
pub struct ConnCase {
	pub from: String,
	pub to: String,
	pub connection: Box<dyn Connection>,
}

#[derive(Debug, Clone)]
pub struct Combiner<P: Positioner> {
	schemes: HashMap<String, Scheme>,
	connections: Vec<ConnCase>,
	positioner: P,

	inputs: Vec<Bind>,
	outputs: Vec<Bind>,

	// invalid_acts: InvalidActs,
}

impl Combiner<ManualPos> {
	pub fn pos_manual() -> Self {
		Combiner::new(ManualPos::new())
	}
}

impl<P: Positioner> Combiner<P> {
	pub fn new(positioner: P) -> Self {
		Combiner {
			schemes: HashMap::new(),
			connections: vec![],
			positioner,
			inputs: vec![],
			outputs: vec![],
			// invalid_acts: InvalidActs::new(),
		}
	}

	pub fn pos(&mut self) -> &mut P {
		&mut self.positioner
	}
}

impl<P: Positioner> Combiner<P> {
	pub fn add<N, S>(&mut self, name: N, scheme: S) -> Result<(), Error>
		where N: Into<String>,
			  S: Into<Scheme>
	{
		let name = name.into();

		if name.contains("/") {
			return Err(InvalidName {
				tip: "Scheme name cannot contain '/' (slash) symbol".to_string(),
				invalid_name: name,
			});
		}

		if self.schemes.get(&name).is_none() {
			self.schemes.insert(name.clone(), scheme.into());
			self.pos().set_last_scheme(name);
			Ok(())
		} else {
			Err(NameWasAlreadyTaken {
				tip: "Scheme with such name was already added".to_string(),
				taken_name: name,
			})
		}
	}

	pub fn add_iter<N, S, I>(&mut self, pairs: I) -> Result<(), Vec<Error>>
		where N: Into<String>,
			  S: Into<Scheme>,
			  I: IntoIterator<Item = (N, S)>
	{
		let mut errors: Vec<Error> = vec![];
		for (name, scheme) in pairs {
			match self.add(name, scheme) {
				Ok(()) => {},
				Err(e) => errors.push(e),
			}
		}

		if errors.len() > 0 {
			return Err(errors)
		} else {
			Ok(())
		}
	}

	pub fn add_mul<S, N, I>(&mut self, names: I, scheme: S) -> Result<(), Vec<Error>>
		where S: Into<Scheme>,
			  N: Into<String>,
			  I: IntoIterator<Item = N>,
	{
		let scheme = scheme.into();
		let mut errors: Vec<Error> = vec![];

		for name in names {
			match self.add(name, scheme.clone()) {
				Ok(()) => {},
				Err(e) => errors.push(e),
			}
		}

		if errors.len() > 0 {
			return Err(errors)
		} else {
			Ok(())
		}
	}
}

impl<P: Positioner> Combiner<P> {
	pub fn custom<P1, P2>(&mut self, from: P1, to: P2, conn: Box<dyn Connection>)
		where P1: Into<String>,
			  P2: Into<String>
	{
		self.connections.push(
			ConnCase {
				from: from.into(),
				to: to.into(),
				connection: conn,
			}
		);
	}

	pub fn connect<P1, P2>(&mut self, from: P1, to: P2)
		where P1: Into<String>,
			  P2: Into<String>
	{
		self.custom(from, to, ConnStraight::new())
	}

	pub fn dim<P1, P2>(&mut self, from: P1, to: P2, adapt_axes: (bool, bool, bool))
		where P1: Into<String>,
				P2: Into<String>,
	{
		self.custom(from, to, ConnDim::new(adapt_axes))
	}

	pub fn custom_iter<I1, I2, P1, P2>(&mut self, from: I1, to: I2, conn: Box<dyn Connection>)
		where P1: Into<String>, I1: IntoIterator<Item = P1>,
			  P2: Into<String>, I2: IntoIterator<Item = P2>,
	{
		let to: Vec<String> = to.into_iter()
			.map(|x| x.into())
			.collect();

		for from_path in from {
			let from_path = from_path.into();
			for to_path in &to {
				self.custom(from_path.clone(), to_path, conn.clone())
			}
		}
	}

	pub fn connect_iter<I1, I2, P1, P2>(&mut self, from: I1, to: I2)
		where P1: Into<String>, I1: IntoIterator<Item = P1>,
			  P2: Into<String>, I2: IntoIterator<Item = P2>,
	{
		self.custom_iter(from, to, ConnStraight::new())
	}

	pub fn dim_iter<I1, I2, P1, P2>(&mut self, from: I1, to: I2, adapt_axes: (bool, bool, bool))
		where P1: Into<String>, I1: IntoIterator<Item = P1>,
			  P2: Into<String>, I2: IntoIterator<Item = P2>,
	{
		self.custom_iter(from, to, ConnDim::new(adapt_axes))
	}
}

impl<P: Positioner> Combiner<P> {
	pub fn bind_input<B>(&mut self, bind: B) -> Result<(), Error>
		where B: Into<Bind>
	{
		let bind = bind.into();

		if bind.name().contains("/") {
			return Err(InvalidName {
				invalid_name: bind.name().clone(),
				tip: "Bind name cannot contain '/' (slash) symbol".to_string()
			})
		}

		for check in &self.inputs {
			if check.name().eq(bind.name()) {
				return Err(NameWasAlreadyTaken {
					taken_name: bind.name().clone(),
					tip: format!("Input bind with such name was already added"),
				})
			}
		}

		self.inputs.push(bind);
		Ok(())
	}

	pub fn bind_output<B>(&mut self, bind: B) -> Result<(), Error>
		where B: Into<Bind>
	{
		let bind = bind.into();

		if bind.name().contains("/") {
			return Err(InvalidName {
				invalid_name: bind.name().clone(),
				tip: "Bind name cannot contain '/' (slash) symbol".to_string()
			})
		}

		for check in &self.outputs {
			if check.name().eq(bind.name()) {
				return Err(NameWasAlreadyTaken {
					taken_name: bind.name().clone(),
					tip: format!("Output bind with such name was already added"),
				})
			}
		}

		self.outputs.push(bind);
		Ok(())
	}

	pub fn pass_input<S, Pt, K>(&mut self, name: S, path: Pt, new_kind: Option<K>) -> Result<(), Error>
		where S: Into<String>,
				Pt: Into<String>,
			  K: Into<String>
	{
		let name = name.into();
		let path = path.into();
		let new_kind = new_kind.map(|k| k.into());

		check_name_validity(&name)?;

		let bind = self.parse_pass_data(name, path, new_kind, SlotSide::Input)?;
		self.bind_input(bind)
	}

	pub fn pass_output<S, Pt, K>(&mut self, name: S, path: Pt, new_kind: Option<K>) -> Result<(), Error>
		where S: Into<String>,
			  Pt: Into<String>,
			  K: Into<String>
	{
		let name = name.into();
		let path = path.into();
		let new_kind = new_kind.map(|k| k.into());

		check_name_validity(&name)?;

		let bind = self.parse_pass_data(name, path, new_kind, SlotSide::Output)?;
		self.bind_output(bind)
	}

	fn parse_pass_data(&self, name: String, path: String, new_kind: Option<String>, side: SlotSide) -> Result<Bind, Error> {
		let (scheme_name, slot_name) = split_first_token(path.clone());
		let slot_name = match slot_name {
			None => "".to_string(),
			Some(name) => name,
		};

		let scheme = match self.schemes.get(&scheme_name) {
			None => return Err(Error::PassHasInvalidTarget {
				pass_name: name,
				pass_side: side,
				tip: format!("Scheme '{}' was not found.", scheme_name),
			}),

			Some(scheme) => scheme,
		};

		let slot = match side {
			SlotSide::Input => scheme.input(&slot_name),
			SlotSide::Output => scheme.output(&slot_name),
		};

		let (slot, sector) = match slot {
			None => return Err(Error::PassHasInvalidTarget {
				pass_name: name,
				pass_side: side,
				tip: format!("Slot {}/{} was not found.", scheme_name, slot_name)
			}),

			Some(values) => values,
		};

		let kind = match new_kind {
			Some(new_kind) => new_kind,
			None => slot.kind().to_string(),
		};

		let mut bind = Bind::new(name, kind, sector.bounds);
		bind.connect_full(path);
		Ok(bind)
	}
}

fn check_name_validity(name: &String) -> Result<(), Error> {
	if name.contains("/") {
		return Err(InvalidName {
			invalid_name: name.to_string(),
			tip: "Name cannot contain '/' (slash) symbol.".to_string()
		});
	}

	Ok(())
}


impl<P: Positioner> Combiner<P> {
	pub fn compile(self) -> Result<(Scheme, InvalidActs), <P as Positioner>::Error> {
		// Placing schemes
		let schemes = self.positioner.arrange(self.schemes)?;

		let mut invalid_acts = InvalidActs::new();
		let mut inputs_map: HashMap<String, (usize, Vec<Slot>)> = HashMap::new();
		let mut outputs_map: HashMap<String, (usize, Vec<Slot>)> = HashMap::new();

		let mut shapes: Vec<(Point, Rot, Shape)> = Vec::new();

		// Combining all schemes into new one
		for (name, (pos, rot, scheme)) in schemes {
			let (scheme_shapes, scheme_inps, scheme_outps) = scheme.disassemble(pos, rot);
			let start_shape = shapes.len();
			inputs_map.insert(name.clone(), (start_shape, scheme_inps));
			outputs_map.insert(name.clone(), (start_shape, scheme_outps));
			shapes.extend(scheme_shapes)
		}

		// Compiling input binds
		let inputs: Vec<Slot> = self.inputs.into_iter()
			.map(|bind| bind.compile(&inputs_map, SlotSide::Input))
			.map(|(slot, invalid)| {
				let invalid = invalid.into_iter()
					.map(|x| (slot.name().clone(), x));

				invalid_acts.inp_bind_conns.extend(invalid);
				slot
			})
			.collect();

		// Compiling output binds
		let outputs: Vec<Slot> = self.outputs.into_iter()
			.map(|bind| bind.compile(&inputs_map, SlotSide::Output))
			.map(|(slot, invalid)| {
				let invalid = invalid.into_iter()
					.map(|x| (slot.name().clone(), x));

				invalid_acts.out_bind_conns.extend(invalid);
				slot
			})
			.collect();

		// Compiling all the connections
		for conn in self.connections {
			let slot_from = get_scheme_slot(&conn.from, &outputs_map);
			let slot_to = get_scheme_slot(&conn.to, &inputs_map);

			if slot_from.is_none() || slot_to.is_none() {
				invalid_acts.connections.push(conn);
				continue;
			}
			let slot_from = slot_from.unwrap();
			let slot_to = slot_to.unwrap();

			compile_connection(slot_from, slot_to, conn.connection, &mut shapes);
		}

		let scheme = Scheme::create(shapes, inputs, outputs);
		Ok((scheme, invalid_acts))
	}
}

fn compile_connection(from: (usize, &Slot, &SlotSector),
					  to: (usize, &Slot, &SlotSector),
					  with: Box<dyn Connection>,
					  shapes: &mut Vec<(Point, Rot, Shape)>)
{
	let p2p_conns = with.connect(from.2.bounds, to.2.bounds);
	let from_offset = from.2.pos;
	let to_offset = to.2.pos;

	for (start, end) in p2p_conns {
		if !is_point_in_bounds(start, from.2.bounds) ||
			!is_point_in_bounds(from_offset + start, from.1.bounds()) ||
			!is_point_in_bounds(end, from.2.bounds) ||
			!is_point_in_bounds(to_offset + end, to.1.bounds())
		{
			continue;
		}

		let from_shapes = from.1.get_point(from_offset + start).unwrap();
		let from_start_shape = from.0;

		let to_shapes = to.1.get_point(to_offset + end).unwrap();
		let to_start_shape = to.0;

		for f_shape_id in from_shapes {
			let f_shape: &mut Shape = &mut shapes[from_start_shape + *f_shape_id].2;
			f_shape.extend_conn(
				to_shapes.into_iter()
					.map(|shape_id| to_start_shape + *shape_id )
			);
		}
	}
}

fn get_scheme_slot<'a>(path: &String, slots: &'a HashMap<String, (usize, Vec<Slot>)>) -> Option<(usize, &'a Slot, &'a SlotSector)> {
	let (scheme_name, slot_name) = split_first_token(path.clone());
	let slot_name = match slot_name {
		None => "".to_string(),		// Default slot name
		Some(name) => name,
	};

	let (slot_name, slot_sector_name) = split_first_token(slot_name);
	let slot_sector_name = match slot_sector_name {
		None => "".to_string(),		// Default sector name
		Some(sector) => sector,
	};

	match slots.get(&scheme_name) {
		None => None,
		Some((start_shape, all_scheme_slots)) => {
			match scheme::find_slot(slot_name, all_scheme_slots) {
				None => None,
				Some(slot) =>
					slot.get_sector(&slot_sector_name)
						.map(|sector| (*start_shape, slot, sector))
			}
		}
	}
}

impl<P: Positioner> Combiner<P> {
	pub fn schemes_inputs(&self) -> HashMap<String, &Vec<Slot>> {
		let mut map: HashMap<String, &Vec<Slot>> = HashMap::new();

		for (name, scheme) in &self.schemes {
			map.insert(name.to_string(), scheme.inputs());
		}

		map
	}

	pub fn schemes_outputs(&self) -> HashMap<String, &Vec<Slot>> {
		let mut map: HashMap<String, &Vec<Slot>> = HashMap::new();

		for (name, scheme) in &self.schemes {
			map.insert(name.to_string(), scheme.outputs());
		}

		map
	}
}
