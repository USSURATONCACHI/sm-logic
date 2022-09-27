use std::collections::HashMap;
use crate::connection::{ConnDim, Connection, ConnStraight};
use crate::positioner::{ManualPos, Positioner};
use crate::scheme::Scheme;
use crate::util::Rot;

type Bind = ();         // TODO

#[derive(Debug, Clone)]
pub struct Warns {
	pub invalid_conns: Vec<ConnCase>
}

impl Warns {
	pub fn new() -> Self {
		Warns {
			invalid_conns: vec![]
		}
	}
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
	pub fn add<N, S>(&mut self, name: N, scheme: S)
		where N: Into<String>,
			  S: Into<Scheme>
	{
		let name = name.into();
		self.schemes.insert(name.clone(), scheme.into());
		self.pos().set_last_scheme(name);
	}

	pub fn add_iter<N, S, I>(&mut self, pairs: I)
		where N: Into<String>,
			  S: Into<Scheme>,
			  I: IntoIterator<Item = (N, S)>
	{
		for (name, scheme) in pairs {
			self.add(name, scheme);
		}
	}

	pub fn add_mul<S, N, I>(&mut self, names: I, scheme: S)
		where S: Into<Scheme>,
			  N: Into<String>,
			  I: IntoIterator<Item = N>,
	{
		let scheme = scheme.into();

		for name in names {
			self.add(name, scheme.clone());
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

	// Also: bind_output
	#[allow(unused_variables)]
	pub fn bind_input<B>(&mut self, bind: B)
		where B: Into<Bind>
	{
		todo!()
	}

	// Also: pass_output
	#[allow(unused_variables)]
	pub fn pass_input<S, T>(&mut self, slot: S, new_type: Option<T>)
		where S: Into<String>,
			  T: Into<String>
	{
		todo!()
	}
}

impl<P: Positioner> Combiner<P> {
	pub fn compile(self) -> (Scheme, Warns) {
		todo!()
	}
}
