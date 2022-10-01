use std::collections::HashMap;
use crate::bind::Bind;
use crate::combiner::CombinerError::SchemeNameWasAlreadyTaken;
use crate::connection::{ConnDim, Connection, ConnStraight};
use crate::positioner::{ManualPos, Positioner};
use crate::scheme::Scheme;
use crate::slot::Slot;
use crate::util::split_first_token;

#[derive(Debug, Clone)]
pub struct Warns {
	pub invalid_conns: Vec<ConnCase>,
	// Input or Output, name, target path, new_kind
	pub non_existent_passes: Vec<(SlotSide, String, String, Option<String>)>
}

impl Warns {
	pub fn new() -> Self {
		Warns {
			invalid_conns: vec![],
			non_existent_passes: vec![],
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum SlotSide {
	Input, Output
}

pub enum CombinerError {
	InvalidSchemeName {
		name: String,
		scheme: Scheme,
		tip: String,
	},

	InvalidSlotName {
		name: String,
		tip: String
	},

	PassHasInvalidTarget {
		pass_name: String,
		new_kind: Option<String>,
		side: SlotSide,

		target: String,
		tip: String,
	},

	SchemeNameWasAlreadyTaken {
		name: String,
		failed_to_add: Scheme,
	},

	SlotNameWasAlreadyTaken {
		name: String,
		side: SlotSide,
		failed_to_add: Bind,
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

	warns: Warns,
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
			warns: Warns::new(),
		}
	}

	pub fn pos(&mut self) -> &mut P {
		&mut self.positioner
	}
}

impl<P: Positioner> Combiner<P> {
	pub fn add<N, S>(&mut self, name: N, scheme: S) -> Result<(), CombinerError>
		where N: Into<String>,
			  S: Into<Scheme>
	{
		let name = name.into();
		if self.schemes.get(&name).is_some() {
			self.schemes.insert(name.clone(), scheme.into());
			self.pos().set_last_scheme(name);
			Ok(())
		} else {
			Err(SchemeNameWasAlreadyTaken {
				name,
				failed_to_add: scheme.into(),
			})
		}
	}

	pub fn add_iter<N, S, I>(&mut self, pairs: I) -> Result<(), Vec<CombinerError>>
		where N: Into<String>,
			  S: Into<Scheme>,
			  I: IntoIterator<Item = (N, S)>
	{
		let mut errors: Vec<CombinerError> = vec![];
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

	pub fn add_mul<S, N, I>(&mut self, names: I, scheme: S) -> Result<(), Vec<CombinerError>>
		where S: Into<Scheme>,
			  N: Into<String>,
			  I: IntoIterator<Item = N>,
	{
		let scheme = scheme.into();
		let mut errors: Vec<CombinerError> = vec![];

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
}

impl<P: Positioner> Combiner<P> {
	pub fn bind_input<B>(&mut self, bind: B)
		where B: Into<Bind>
	{
		self.inputs.push(bind.into());
	}

	pub fn bind_output<B>(&mut self, bind: B)
		where B: Into<Bind>
	{
		self.outputs.push(bind.into());
	}

	pub fn pass_input<S, Pt, K>(&mut self, name: S, path: Pt, new_kind: Option<K>) -> Result<(), CombinerError>
		where S: Into<String>,
				Pt: Into<String>,
			  K: Into<String>
	{
		let new_kind = new_kind.map(|k| k.into());
		let path = path.into();
		let (name, target_scheme, target_slot) =
			parse_pass_data(name.into(), path.clone(), &new_kind)?;

		let scheme = match self.schemes.get(&target_scheme) {
			None => return Err(CombinerError::PassHasInvalidTarget {
				pass_name: name,
				new_kind,
				side: SlotSide::Input,
				target: target_scheme.clone(),
				tip: format!("Scheme '{}' was not found", target_scheme),
			}),

			Some(scheme) => scheme,
		};

		let slot = scheme.input(&target_slot);

		let (slot, sector) = match slot {
			None => return Err(CombinerError::PassHasInvalidTarget {
				pass_name: name,
				new_kind,
				side: SlotSide::Input,
				target: path,
				tip: format!("Slot {}/{} was not found", target_scheme, target_slot)
			}),

			Some(values) => values,
		};

		let kind = match new_kind {
			Some(new_kind) => new_kind,
			None => slot.kind().to_string(),
		};

		let mut bind = Bind::new(name, kind, sector.bounds);
		bind.connect_full(path);
		self.bind_input(bind);

		Ok(())
	}

	pub fn pass_output<S, Pt, K>(&mut self, name: S, path: Pt, new_kind: Option<K>) -> Result<(), CombinerError>
		where S: Into<String>,
			  Pt: Into<String>,
			  K: Into<String>
	{
		let new_kind = new_kind.map(|k| k.into());
		let path = path.into();
		let (name, target_scheme, target_slot) =
			parse_pass_data(name.into(), path.clone(), &new_kind)?;

		let scheme = match self.schemes.get(&target_scheme) {
			None => return Err(CombinerError::PassHasInvalidTarget {
				pass_name: name,
				new_kind,
				side: SlotSide::Output,
				target: target_scheme.clone(),
				tip: format!("Scheme '{}' was not found", target_scheme),
			}),

			Some(scheme) => scheme,
		};

		let slot = scheme.output(&target_slot);

		let (slot, sector) = match slot {
			None => return Err(CombinerError::PassHasInvalidTarget {
				pass_name: name,
				new_kind,
				side: SlotSide::Output,
				target: path,
				tip: format!("Slot {}/{} was not found", target_scheme, target_slot)
			}),

			Some(values) => values,
		};

		let kind = match new_kind {
			Some(new_kind) => new_kind,
			None => slot.kind().to_string(),
		};

		let mut bind = Bind::new(name, kind, sector.bounds);
		bind.connect_full(path);
		self.bind_output(bind);

		Ok(())
	}
}

fn parse_pass_data(name: String, path: String, new_kind: &Option<String>)
	-> Result<(String, String, String), CombinerError>
{
	if name.contains("/") {
		return Err(CombinerError::InvalidSlotName {
			name,
			tip: "Pass name cannot contain '/' (slash) symbol.".to_string()
		})
	}

	let (target_scheme, target_slot) = split_first_token(path.clone());

	if target_scheme.len() == 0 {
		return Err(CombinerError::PassHasInvalidTarget {
			pass_name: name,
			new_kind: new_kind.clone(),
			side: SlotSide::Input,
			target: path,
			tip: "No Scheme name is specified. Required format: <scheme>/<slot name>.".to_string()
		})
	}

	if target_slot.is_none() {
		return Err(CombinerError::PassHasInvalidTarget {
			pass_name: name,
			new_kind: new_kind.clone(),
			side: SlotSide::Input,
			target: path,
			tip: "No Slot name is specified. Required format: <scheme>/<slot name>.".to_string()
		})
	}

	let target_slot = target_slot.unwrap();
	if target_slot.contains("/") {
		return Err(CombinerError::InvalidSlotName {
			name: target_slot,
			tip: "Pass name cannot contain '/' (slash) symbol. Required format: <scheme>/<slot name>".to_string()
		});
	}

	Ok((name, target_scheme, target_slot))
}

impl<P: Positioner> Combiner<P> {
	pub fn compile(self) -> Result<(Scheme, Warns), Vec<CombinerError>> {
		todo!()
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
