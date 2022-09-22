use std::collections::HashMap;
use crate::connection::Connection;
use crate::scheme::Scheme;
use crate::util::Rotation;

type SlotPath = ();     // TODO
type Placement = ();    // TODO
type Bind = ();         // TODO
type Warn = ();         // TODO

#[derive(Debug, Clone)]
struct ConnCase {
	from: SlotPath,
	to: SlotPath,
	connection: Box<dyn Connection>,
}

#[derive(Debug, Clone)]
struct SchemeCase {
	scheme: Scheme,
	pos: Placement,
	rot: Rotation,
}

#[derive(Debug, Clone)]
pub struct Combiner {
	schemes: HashMap<String, SchemeCase>,
	connections: Vec<ConnCase>,

	inputs: Vec<Bind>,
	outputs: Vec<Bind>,

	warns: Vec<Warn>,
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
		todo!()
	}

	/*
	Also:
		place_iter(pairs: I),
		place_last(at: P)
	*/
	pub fn place<N, P>(&mut self, scheme_name: N, at: P)
		where N: Into<String>
	{
		todo!()
	}

	pub fn rotate_by<N, R>(&mut self, scheme_name: N, rot: R)
		where N: Into<String>,
			  R: Into<Rotation>,
	{
		todo!()
	}

	pub fn rotate_at<N, R>(&mut self, scheme_name: N, rot: R)
		where N: Into<String>,
			  R: Into<Rotation>,
	{
		todo!()
	}

	/*
	Also:
		custom,
		dim?,
		chain?,
		connect_func,
	*/
	pub fn connect<P1, P2>(&mut self, from: P1, to: P2)
		where P1: Into<SlotPath>,
			  P2: Into<SlotPath>
	{
		todo!()
	}

	// Also: bind_output
	pub fn bind_input<B>(&mut self, bind: B)
		where B: Into<Bind>
	{
		todo!()
	}

	// Also: pass_output
	pub fn pass_input<S, T>(&mut self, slot: S, new_type: Option<T>)
		where S: Into<SlotPath>,
			  T: Into<String>
	{
		todo!()
	}
}

impl Combiner {
	pub fn compile(self) -> (Scheme, Vec<Warn>) {
		todo!()
	}
}