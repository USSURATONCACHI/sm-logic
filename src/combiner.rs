use std::collections::HashMap;
use crate::connection::{Connection, ConnStraight};
use crate::scheme::Scheme;
use crate::util::Rot;

type Bind = ();         // TODO

#[derive(Debug, Clone)]
pub struct Warns {
	pub invalid_conns: Vec<ConnCase>
}

#[derive(Debug, Clone)]
pub struct ConnCase {
	pub from: String,
	pub to: String,
	pub connection: Box<dyn Connection>,
}

#[derive(Debug, Clone)]
pub struct Combiner {
	schemes: HashMap<String, Scheme>,
	connections: Vec<ConnCase>,

	inputs: Vec<Bind>,
	outputs: Vec<Bind>,

	warns: Warns,
}

impl Combiner {
	pub fn new() -> Self {
		todo!()
	}

	/*
	Also:
		add_iter(schemes: IntoIter<(String, Scheme)>),
		add_func(iter: I, map: M),
		add_mul(scheme: Scheme, names: I),
		add_func_mul(scheme: S, iter: I, map_name: M),
	*/
	pub fn add<N, S>(&mut self, name: N, scheme: S)
		where N: Into<String>,
			  S: Into<Scheme>
	{
		self.schemes.insert(name.into(), scheme.into());
	}

	/*
	Also:
		custom,
		dim?,
		chain?,
		connect_func,
	*/
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

impl Combiner {
	pub fn compile(self) -> (Scheme, Warns) {
		todo!()
	}
}