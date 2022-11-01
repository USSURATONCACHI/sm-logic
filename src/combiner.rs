use std::collections::HashMap;
use std::fmt::Debug;
use crate::bind::{Bind, InvalidConn};
use crate::combiner::Error::{InvalidName, NameWasAlreadyTaken};
use crate::connection::{ConnDim, Connection, ConnStraight};
use crate::positioner::{ManualPos, Positioner};
use crate::presets::shapes_cube;
use crate::scheme;
use crate::scheme::Scheme;
use crate::shape::Shape;
use crate::slot::{Slot, SlotSector};
use crate::util::{Bounds, is_point_in_bounds, MAX_CONNECTIONS, Point, Rot, split_first_token};

/// Container for all invalid actions performed on the Combiner.
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

	NoSuchScheme {
		name: String,
	}
}

#[derive(Debug, Clone)]
pub enum CompileError<P> {
	PositionerError(P),
	ConnectionsOverflow {
		affected_inputs: Vec<String>,
		affected_outputs: Vec<String>,
		tip: String,
	},
}

/// Container for single connection with all of its parameters
#[derive(Debug, Clone)]
pub struct ConnCase {
	pub from: String,
	pub to: String,
	pub connection: Box<dyn Connection>,
}

/// The [`Scheme`] builder.
/// Can contain schemes, interconnect them and combine/compile into
/// new bigger scheme.
///
/// # Adding schemes
///
/// You can add schemes into combiner. Every scheme has to have unique
/// name in the Combiner:
///
/// ```
/// # use crate::sm_logic::combiner::Combiner;
/// # use crate::sm_logic::shape::vanilla::GateMode;
/// # use crate::sm_logic::shape::vanilla::Timer;
/// # use crate::sm_logic::shape::vanilla::BlockBody;
/// # use crate::sm_logic::shape::vanilla::BlockType;
/// let mut combiner = Combiner::pos_manual();
///
/// // GateMode::XOR implements Into<Scheme>, so it is automatically converted into scheme
/// let res = combiner.add("first_gate", GateMode::XOR);
///	assert!(res.is_ok());
///
/// // These two name are unique
/// let res = combiner.add("timer", Timer::new(42));
///	assert!(res.is_ok());
///
/// // 'timer' name is already taken
/// let plate = BlockBody::new(BlockType::Cardboard, (10, 10, 1));	// `Shape` also implements Into<Scheme>
/// let res = combiner.add("timer", plate);
///	assert!(res.is_err());
/// ```
///
/// # Connecting schemes
///
/// Schemes have *slots* - inputs and outputs.
///
/// Outputs can be connected to inputs and only this way (output ->
/// input).
///
/// Connection from input TO output (input -> output) is invalid and
/// impossible.<br><br>
///
/// Slots are specified to the combined via slot *paths*. Path is a
/// string in following format:<br>
/// **`"<scheme name>/<slot_name>/<optional - sector name>"`**<br><br>
///
/// If you do not specify the sector, the whole slot will be connected.
/// Next two formats are the same:<br>
/// **`"<scheme name>/<slot_name>"`**<br>
/// **`"<scheme name>/<slot_name>/"`**<br><br>
///
/// Examples:
/// ```
/// # use crate::sm_logic::combiner::Combiner;
/// # let mut combiner = Combiner::pos_manual();
/// // "from path" -> "to path"
/// // scheme - 'adder_1', slot - 'a', sector - default
/// combiner.connect("adder_1/a", "adder_2/a");
/// // scheme - 'encryptor', slot - 'data', sector - 'first_number'
/// combiner.connect("encryptor/data/first_number", "adder_2/b");
/// ```
///
/// If you do not specify slot name, slot name is set to default. Next
/// two examples are equal:<br>
/// ```
/// # use crate::sm_logic::combiner::Combiner;
/// # let mut combiner = Combiner::pos_manual();
/// // "from path" -> "to path"
/// // connect to default slot of 'and_gate'
/// combiner.connect("signal", "and_gate");
/// // connect to '_' (default) slot of 'and_gate'
/// combiner.connect("signal", "and_gate/_");
/// ```
///
/// # Setting position and rotation of schemes
///
/// `Combiner` have very flexible and simple system for assigning
/// physical positions and rotations to schemes. It is [`Positioner`]
/// trait. `Combiner<P: Positioner>` allows for any logic of positioning
/// its schemes.
///
/// <br>
///
/// You can get mutable reference to positioner by calling
/// `combiner.pos()`
///
/// <br>
///
/// Default implementation of [`Positioner`] is [`ManualPos`].
/// [`ManualPos`] requires for you to set positions of all the added
/// schemes. Rotations by default are (0, 0, 0), but you can rotate
/// schemes too.
///
/// If at least one scheme's position is not set - error will be
/// returned at compilation stage.
///
/// Example:
/// ```
/// # use crate::sm_logic::combiner::Combiner;
/// # use crate::sm_logic::shape::vanilla::GateMode;
/// # use crate::sm_logic::shape::vanilla::Timer;
/// # use crate::sm_logic::shape::vanilla::BlockBody;
/// # use crate::sm_logic::shape::vanilla::BlockType;
/// // Combiner::pos_manual() returns default Combiner<ManualPos>
/// let mut combiner = Combiner::pos_manual();
///
/// let res = combiner.add("first_gate", GateMode::XOR).unwrap();
/// // Last added scheme will be placed in the specified position
/// combiner.pos().place_last((0, 0, 0));
/// // Rotation is not necessary, but possible.
/// // 'first_gate' will be rotated around CENTER of BLOCK (0, 0, 0)
/// // relative to the scheme itself. Since Logic Gate has size (1, 1, 1)
/// // Its position won't change, only orientation. But bigger schemes
/// // will rotate around center of corner block.
///
/// // But there is a hack: if we `scheme.rotate((rx, ry, rz))` before
/// // adding it to the combiner its corner block will change.
/// combiner.pos().rotate_last((0, 0, 2));
///
/// let res = combiner.add("timer", Timer::new(42)).unwrap();
/// // We will set the position later
///
/// let res = combiner.add("plate", BlockBody::new(BlockType::Cardboard, (10, 10, 1)));
///
/// combiner.pos().place("timer", (0, 0, 1)); // Right on top of the gate
/// combiner.pos().place("plate", (0, 0, -1)); // Right under the gate
///
/// // All schemes have their positions, so there would be no error
/// // at compilation
/// ```
///
/// # Binding inputs and outputs
///
/// After adding all the components (schemes) and connecting them, it is
/// usual to add some inputs and/or outputs to the scheme, that is being
/// constructed.
///
/// For example: scheme, that controls the door from buttons for sure
/// should have inputs for control buttons, and output for the door
/// controller (open/close).
///
/// For example: binary adder should have two inputs for binary numbers,
/// one output for their sum, and possibly output/input for carry bit.
///
/// # Example: lets create one bit adder
/// ```
/// # use crate::sm_logic::combiner::Combiner;
/// # use crate::sm_logic::shape::vanilla::GateMode::*;
/// # use crate::sm_logic::util::Facing;
/// # use crate::sm_logic::bind::Bind;
/// let mut s = Combiner::pos_manual();
///
/// // Add gates for inputs - bit a, bit b, carry bit
/// s.add_mul(["a", "b", "carry"], OR).unwrap();
/// // Add gates for adder logic
/// s.add_mul(["and_1", "and_2", "and_3"], AND).unwrap();
/// // Gate with result
/// s.add("res", XOR).unwrap();
///
/// // Place all the gates to their positions
/// s.pos().place_iter([
/// 	("a", (0, 0, 0)),
/// 	("b", (0, 0, 1)),
/// 	("carry", (2, 0, 1)),
/// 	("and_1", (1, 0, 0)),
/// 	("and_2", (1, 0, 1)),
/// 	("and_3", (2, 0, 0)),
/// 	("res", (3, 0, 0)),
/// ]);
///
/// s.pos().rotate("a", Facing::NegX.to_rot());
/// s.pos().rotate("b", Facing::NegX.to_rot());
/// s.pos().rotate("res", Facing::PosX.to_rot());
///
/// // ==== THE BINDING PART ====
/// // Bind input 'a'.
/// let mut bind = Bind::new("a", "bit", (1, 1, 1));
/// bind.connect_full("a");	// Connect this slot to gate 'a', default slot.
/// s.bind_input(bind).unwrap(); // Add the bind to the combiner.
///
/// // The same way bind inputs 'b' and 'carry'.
/// let mut bind = Bind::new("b", "bit", (1, 1, 1));
/// bind.connect_full("b");
/// s.bind_input(bind).unwrap();
///
/// let mut bind = Bind::new("carry", "bit", (1, 1, 1));
/// bind.connect_full("carry");
/// s.bind_input(bind).unwrap();
///
/// // Connecting each on the left to each on the right.
/// s.connect_iter(["a", "b", "_"], ["res"]);
///
/// s.connect_iter(["a"], ["and_1", "and_2"]);
/// s.connect_iter(["b"], ["and_2", "and_3"]);
/// s.connect_iter(["_"], ["and_3", "and_1"]);
///
/// // ==== THE BINDING PART ====
/// // Bind output 'carry'. Note: input 'carry' and output 'carry' have
/// // the same name, but won't produce error, since those do not interfere
/// let mut bind = Bind::new("carry", "bit", (1, 1, 1));
/// bind.connect_full("and_1")  // Slot point is being connected to THREE
/// 	.connect_full("and_2")  // shapes simultaneously.
/// 	.connect_full("and_3");
/// s.bind_output(bind).unwrap();///
///
/// // Bind output 'res' - for result. Just as inputs
/// let mut bind = Bind::new("res", "bit", (1, 1, 1));
/// bind.connect_full("res");
/// s.bind_output(bind).unwrap();
///
/// assert!(s.compile().is_ok());
/// ```
#[derive(Debug, Clone)]
pub struct Combiner<P: Positioner> {
	schemes: HashMap<String, Scheme>,
	last_scheme: Option<String>,

	connections: Vec<ConnCase>,
	positioner: P,

	inputs: Vec<Bind>,
	outputs: Vec<Bind>,

	conns_overflow_allowed: bool,
	debug_name: Option<String>,
}

impl Combiner<ManualPos> {
	/// Default Combiner - with positioner of ManualPos
	pub fn pos_manual() -> Self {
		Combiner::new(ManualPos::new())
	}
}

impl<P: Positioner> Combiner<P> {
	/// Creates new Combiner with custom positioner
	pub fn new(positioner: P) -> Self {
		Combiner {
			schemes: HashMap::new(),
			last_scheme: None,
			connections: vec![],
			positioner,
			inputs: vec![],
			outputs: vec![],
			conns_overflow_allowed: false,
			debug_name: None,
		}
	}

	pub fn set_debug_name<S: Into<String>>(&mut self, name: S) {
		self.debug_name = Some(name.into());
	}

	/// Returns mutable reference to positioner
	///
	/// # Example
	/// ```
	/// # use crate::sm_logic::combiner::Combiner;
	/// // Example with ManualPos
	/// let mut combiner = Combiner::pos_manual();
	///
	/// // combiner.pos() returns &mut ManualPos
	/// combiner.pos().place("scheme", (1, 2 ,3));
	/// ```
	pub fn pos(&mut self) -> &mut P {
		&mut self.positioner
	}

	pub fn last_scheme(&self) -> Option<&Scheme> {
		match &self.last_scheme {
			None => None,
			Some(name) => self.schemes.get(name),
		}
	}

	pub fn last_scheme_mut(&mut self) -> Option<&mut Scheme> {
		match &self.last_scheme {
			None => None,
			Some(name) => self.schemes.get_mut(name),
		}
	}

	pub fn allow_conns_overflow(&mut self) {
		self.conns_overflow_allowed = true;
	}
}

impl<P: Positioner> Combiner<P> {
	pub fn set_forcibly_used<N>(&mut self, name: N) -> Result<(), Error>
		where N: Into<String>
	{
		let name = name.into();

		match self.schemes.get_mut(&name) {
			Some(scheme) => {
				scheme.set_forcibly_used();
				Ok(())
			}

			None => Err(Error::NoSuchScheme { name })
		}
	}

	pub fn unset_forcibly_used<N>(&mut self, name: N) -> Result<(), Error>
		where N: Into<String>
	{
		let name = name.into();

		match self.schemes.get_mut(&name) {
			Some(scheme) => {
				scheme.unset_forcibly_used();
				Ok(())
			}

			None => Err(Error::NoSuchScheme { name })
		}
	}
}

impl<P: Positioner> Combiner<P> {
	/// Adds scheme with its unique name to the combiner.
	///
	/// # Example
	/// ```
	/// # use sm_logic::scheme::Scheme;
	/// # use sm_logic::shape::vanilla::GateMode;
	/// # use crate::sm_logic::combiner::Combiner;
	/// let mut combiner = Combiner::pos_manual();
	///
	/// combiner.add("and_gate", GateMode::AND).unwrap();
	/// // Do not forget to set its position later, if you use ManualPos
	/// ```
	pub fn add<N, S>(&mut self, name: N, scheme: S) -> Result<(), Error>
		where N: Into<String>,
			  S: Into<Scheme>
	{
		let name = name.into();

		if name.contains("/") {
			return Err(InvalidName {
				tip: match &self.debug_name {
					None => "Scheme name cannot contain '/' (slash) symbol".to_string(),
					Some(name) => format!("Scheme name cannot contain '/' (slash) symbol ('{}')", name),
				},
				invalid_name: name,
			});
		}

		if self.schemes.get(&name).is_none() {
			self.schemes.insert(name.clone(), scheme.into());
			self.last_scheme = Some(name.clone());
			self.pos().set_last_scheme(name);
			Ok(())
		} else {
			Err(NameWasAlreadyTaken {
				tip: match &self.debug_name {
					None => "Scheme with such name was already added".to_string(),
					Some(name) => format!("Scheme with such name was already added to '{}'", name),
				},
				taken_name: name,
			})
		}
	}

	pub fn add_pass_all<N, S, I, O>(&mut self, name: N, scheme: S, inputs_names: I, outputs_names: O) -> Result<(), Error>
		where N: Into<String>,
			  S: Into<Scheme>,
				I: Fn(&String) -> String,
				O: Fn(&String) -> String,
	{
		let name = name.into();
		let scheme = scheme.into();

		// bind name, target name
		let inputs: Vec<(String, String)> = scheme.inputs().iter()
			.map(|slot| (inputs_names(slot.name()), slot.name().clone()) ).collect();

		let outputs: Vec<(String, String)> = scheme.outputs().iter()
			.map(|slot| (outputs_names(slot.name()), slot.name().clone()) ).collect();

		self.add(&name, scheme)?;

		for (bind_name, target_name) in inputs {
			self.pass_input(bind_name, format!("{}/{}", &name, target_name), None as Option<String>)?;
		}

		for (bind_name, target_name) in outputs {
			self.pass_output(bind_name, format!("{}/{}", &name, target_name), None as Option<String>)?;
		}

		Ok(())
	}

	/// Adds all the (name, scheme) pairs passed to the combiner.
	///
	/// # Example
	/// ```
	/// # use sm_logic::shape::vanilla::GateMode::*;
	/// # use sm_logic::shape::vanilla::Timer;
	/// # use crate::sm_logic::combiner::Combiner;
	/// let mut combiner = Combiner::pos_manual();
	///
	/// combiner.add_iter([
	/// 	("and_gate", AND),
	/// 	("or_gate", OR),
	/// 	("xor_gate", XOR),
	/// 	("nand_gate", NAND),
	/// 	("nor_gate", NOR),
	/// 	("xnor_gate", XNOR),
	/// ]).unwrap();
	/// // Do not forget to set all positions later, if you use ManualPos
	/// ```
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

	/// Adds multiple copies of the same scheme but with different names.
	///
	/// # Example
	/// ```
	/// # use sm_logic::shape::vanilla::Timer;
	/// # use crate::sm_logic::combiner::Combiner;
	/// let mut combiner = Combiner::pos_manual();
	///
	/// combiner.add_mul(["timer_1", "timer_2", "timer_3"], Timer::new(42)).unwrap();
	/// // Do not forget to set all positions later, if you use ManualPos
	/// ```
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
	/// Connects two slots with given connection (`conn` arg).
	/// 'custom' - is for 'custom connection'
	///
	/// # Example
	/// ```
	/// # use sm_logic::connection::ConnMap;
	/// # use crate::sm_logic::combiner::Combiner;
	/// # let mut combiner = Combiner::pos_manual();
	/// let connection = ConnMap::new(|(point, _), _| Some(point * 2));
	/// combiner.custom("scheme1/slot1", "scheme2/slot2", connection);
	/// ```
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

	/// Connects two slots with straight connection ([`ConnStraight`]).
	/// 'Straight' means, that each point of output slot connects to the
	/// same point of input slot.
	///
	/// # Example
	/// ```
	/// # use sm_logic::connection::{ConnStraight};
	/// # use crate::sm_logic::combiner::Combiner;
	/// # let mut combiner = Combiner::pos_manual();
	///
	/// combiner.connect("scheme1/slot1", "scheme2/slot2");
	/// // These two lines do the same thing
	/// combiner.custom("scheme1/slot1", "scheme2/slot2", ConnStraight::new());
	/// ```
	pub fn connect<P1, P2>(&mut self, from: P1, to: P2)
		where P1: Into<String>,
			  P2: Into<String>
	{
		self.custom(from, to, ConnStraight::new())
	}

	/// Connects two slots with dimensional connection ([`ConnDim`]).
	/// 'Dim' is for 'dimensional' and it means, that specified dimensions
	/// of the slot will be ignored ("flattened").<br><br>
	///
	/// Explanation attempt:
	/// In the following example adapted axis is X (only first is true - (true, false, false)).
	/// Due to that, for example, output points (inp_x, 1, 2) for ***each*** 'inp_x' will be connected
	/// to input points (out_x, 1, 2) for ***each*** 'out_x'.<br><br>
	///
	/// Explanation attempt 2:
	/// It's like slots are being "flattened" on certain axes, then connected, and
	/// then "unflattened" back.
	///
	/// # Example
	/// ```
	/// # use sm_logic::connection::{ConnDim};
	/// # use crate::sm_logic::combiner::Combiner;
	/// # let mut combiner = Combiner::pos_manual();
	///
	/// combiner.dim("scheme1/slot1", "scheme2/slot2", (true, false, false));
	/// // These two lines do the same thing
	/// combiner.custom("scheme1/slot1", "scheme2/slot2", ConnDim::new((true, false, false)));
	/// ```
	pub fn dim<P1, P2>(&mut self, from: P1, to: P2, adapt_axes: (bool, bool, bool))
		where P1: Into<String>,
				P2: Into<String>,
	{
		self.custom(from, to, ConnDim::new(adapt_axes))
	}

	/// Just like 'custom', but for multiple targets. ***Each*** slot
	/// on the left will be connected to ***each*** slot on the right<br>
	/// (with given connection (`conn` arg)).
	/// 'custom' - is for 'custom connection'
	///
	/// # Example
	/// ```
	/// # use sm_logic::connection::{ConnStraight};
	/// # use crate::sm_logic::combiner::Combiner;
	/// # let mut combiner = Combiner::pos_manual();
	///
	/// combiner.custom_iter(["1", "2", "3"], ["4", "5", "6"], ConnStraight::new());
	/// // These two ways do the same thing
	///
	/// // Each to each
 	///	let conn = ConnStraight::new();
	/// combiner.custom("1", "4", conn.clone());
	/// combiner.custom("1", "5", conn.clone());
	/// combiner.custom("1", "6", conn.clone());
	/// combiner.custom("2", "4", conn.clone());
	/// combiner.custom("2", "5", conn.clone());
	/// combiner.custom("2", "6", conn.clone());
	/// combiner.custom("3", "4", conn.clone());
	/// combiner.custom("3", "5", conn.clone());
	/// combiner.custom("3", "6", conn.clone());
	/// ```
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

	/// Just like 'connect', but for multiple targets. ***Each*** slot
	/// on the left will be connected to ***each*** slot on the right
	/// with straight connection ([`ConnStraight`]).
	/// 'Straight' means, that each point of output slot connects to the
	/// same point of input slot.
	///
	/// # Example
	/// ```
	/// # use sm_logic::connection::{ConnStraight};
	/// # use crate::sm_logic::combiner::Combiner;
	/// # let mut combiner = Combiner::pos_manual();
	///
	/// combiner.connect_iter(["1", "2", "3"], ["4", "5", "6"]);
	/// // These two lines do the same thing
	/// combiner.custom_iter(["1", "2", "3"], ["4", "5", "6"], ConnStraight::new());
	/// ```
	pub fn connect_iter<I1, I2, P1, P2>(&mut self, from: I1, to: I2)
		where P1: Into<String>, I1: IntoIterator<Item = P1>,
			  P2: Into<String>, I2: IntoIterator<Item = P2>,
	{
		self.custom_iter(from, to, ConnStraight::new())
	}

	/// Just like 'connect', but for multiple targets. ***Each*** slot
	/// on the left will be connected to ***each*** slot on the right
	/// with dimensional connection ([`ConnDim`]).
	///
	/// 'Dim' is for 'dimensional' and it means, that specified dimensions
	/// of the slot will be ignored ("flattened").<br><br>
	///
	/// Explanation attempt:
	/// In the following example adapted axis is X (only first is true - (true, false, false)).
	/// Due to that, for example, output points (inp_x, 1, 2) for ***each*** 'inp_x' will be connected
	/// to input points (out_x, 1, 2) for ***each*** 'out_x'.<br><br>
	///
	/// Explanation attempt 2:
	/// It's like slots are being "flattened" on certain axes, then connected, and
	/// then "unflattened" back.
	///
	/// # Example
	/// ```
	/// # use sm_logic::connection::{ConnDim, ConnStraight};
	/// # use crate::sm_logic::combiner::Combiner;
	/// # let mut combiner = Combiner::pos_manual();
	///
	/// combiner.dim_iter(["1", "2", "3"], ["4", "5", "6"], (false, true, false));
	/// // These two lines do the same thing
	/// combiner.custom_iter(["1", "2", "3"], ["4", "5", "6"], ConnDim::new((false, true, false)));
	/// ```
	pub fn dim_iter<I1, I2, P1, P2>(&mut self, from: I1, to: I2, adapt_axes: (bool, bool, bool))
		where P1: Into<String>, I1: IntoIterator<Item = P1>,
			  P2: Into<String>, I2: IntoIterator<Item = P2>,
	{
		self.custom_iter(from, to, ConnDim::new(adapt_axes))
	}
}

impl<P: Positioner> Combiner<P> {
	/// Adds input bind to all binds list. Bind name must be unique.
	///
	/// # Example
	/// ```
	/// # use crate::sm_logic::combiner::Combiner;
	/// # use crate::sm_logic::bind::Bind;
	/// # let mut combiner = Combiner::pos_manual();
	///
	/// // Slot 2X by 1Y by 1Z
	/// let mut input = Bind::new("my input", "my input kind", (2, 1, 1));
	/// // Connect sector with size 1X by 1Y by 1Z to some slot
	/// input.connect(((0, 0, 0), (1, 1, 1)), "left_gate/_");
	/// // Connect another sector
	/// input.connect(((1, 0, 0), (1, 1, 1)), "right_gate/_");
	///
	/// combiner.bind_input(input).unwrap();
	/// ```
	pub fn bind_input<B>(&mut self, bind: B) -> Result<(), Error>
		where B: Into<Bind>
	{
		let bind = bind.into();

		if bind.name().contains("/") {
			return Err(InvalidName {
				invalid_name: bind.name().clone(),
				tip: match &self.debug_name {
					None => "Bind name cannot contain '/' (slash) symbol".to_string(),
					Some(name) => format!("Bind name cannot contain '/' (slash) symbol ('{}')", name),
				}
			})
		}

		for check in &self.inputs {
			if check.name().eq(bind.name()) {
				return Err(NameWasAlreadyTaken {
					taken_name: bind.name().clone(),
					tip: match &self.debug_name {
						None => format!("Input bind with such name was already added"),
						Some(name) => format!("Input bind with such name was already added to '{}'", name),
					},
				})
			}
		}

		self.inputs.push(bind);
		Ok(())
	}

	/// Adds input bind to all binds list. Bind name must be unique.
	///
	/// # Example
	/// ```
	/// # use crate::sm_logic::combiner::Combiner;
	/// # use crate::sm_logic::bind::Bind;
	/// # let mut combiner = Combiner::pos_manual();
	///
	/// // Slot 2X by 1Y by 1Z
	/// let mut output = Bind::new("my output", "my output kind", (2, 1, 1));
	/// // Connect sector with size 1X by 1Y by 1Z to some slot
	/// output.connect(((0, 0, 0), (1, 1, 1)), "left_gate/_");
	/// // Connect another sector
	/// output.connect(((1, 0, 0), (1, 1, 1)), "right_gate/_");
	///
	/// combiner.bind_output(output).unwrap();
	/// ```
	pub fn bind_output<B>(&mut self, bind: B) -> Result<(), Error>
		where B: Into<Bind>
	{
		let bind = bind.into();

		if bind.name().contains("/") {
			return Err(InvalidName {
				invalid_name: bind.name().clone(),
				tip: match &self.debug_name {
					None => "Bind name cannot contain '/' (slash) symbol".to_string(),
					Some(name) => format!("Bind name cannot contain '/' (slash) symbol ('{}')", name),
				},
			})
		}

		for check in &self.outputs {
			if check.name().eq(bind.name()) {
				return Err(NameWasAlreadyTaken {
					taken_name: bind.name().clone(),
					tip: match &self.debug_name {
						None => format!("Output bind with such name was already added"),
						Some(name) => format!("Output bind with such name was already added to ('{}')", name),
					},
				})
			}
		}

		self.outputs.push(bind);
		Ok(())
	}

	/// Copies input from inner scheme, but name and kind might be replaced.
	///
	/// # Example
	/// ```
	/// # use sm_logic::shape::vanilla::GateMode;
	/// # use crate::sm_logic::combiner::Combiner;
	/// # use crate::sm_logic::bind::Bind;
	/// let mut combiner = Combiner::pos_manual();
	/// // Could be any scheme
	/// combiner.add("signal_gate", GateMode::XOR).unwrap();
	///
	/// // New slot with name "my input pass" will be created,
	/// // connected to "signal_gate/_" slot.
	/// // None means slot kind will be the same with target's
	/// // `"signal_gate"` is path to the slot to be copied
	/// combiner.pass_input("my input pass", "signal_gate", None as Option<String>).unwrap();
	/// ```
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

	/// Copies output from inner scheme, but name and kind might be replaced.
	///
	/// # Example
	/// ```
	/// # use sm_logic::shape::vanilla::GateMode;
	/// # use crate::sm_logic::combiner::Combiner;
	/// # use crate::sm_logic::bind::Bind;
	/// let mut combiner = Combiner::pos_manual();
	/// // Could be any scheme
	/// combiner.add("signal_gate", GateMode::XOR).unwrap();
	///
	/// // New slot with name "my output pass" will be created,
	/// // connected to "signal_gate/_" slot.
	/// // None means slot kind will be the same with target's
	/// // `"signal_gate"` is path to the slot to be copied
	/// combiner.pass_output("my output pass", "signal_gate", None as Option<String>).unwrap();
	/// ```
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
				tip: match &self.debug_name {
					None => format!("Scheme '{}' was not found.", scheme_name),
					Some(name) => format!("Scheme '{}' was not found in '{}'.", scheme_name, name),
				},
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
				tip: match &self.debug_name {
					None => format!("Slot {}/{} was not found (Scheme exists, but not the slot).", scheme_name, slot_name),
					Some(name) => format!("Slot {}/{} was not found in '{}' (Scheme exists, but not the slot).", scheme_name, slot_name, name),
				},
			}),

			Some(values) => values,
		};

		let kind = match new_kind {
			Some(new_kind) => new_kind,
			None => slot.kind().to_string(),
		};

		let mut bind = Bind::new(name, kind, sector.bounds);

		if sector.pos.eq(&Point::new(0, 0, 0)) && sector.bounds == slot.bounds() {
			for (sec_name, sector) in slot.sectors() {
				if sec_name.len() == 0 {
					continue;
				}
				bind.add_sector(sec_name.clone(), sector.pos.clone(), sector.bounds.clone(), sector.kind.clone()).unwrap();
			}
		}

		bind.connect_full(path);

		Ok(bind)
	}
}

impl<P: Positioner> Combiner<P> {
	/// Creates Scheme which is just cube of copies of passed shape.
	/// - '`name`' is _scheme_ name.
	/// - '`slot_kind`' is _slots'_ kind - there will be '_' input and '_' output.
	/// - '`bounds`' is the size of the cube (count of shapes).
	///
	/// # Example
	/// ```
	/// # use sm_logic::shape::vanilla::GateMode;
	/// # use crate::sm_logic::combiner::Combiner;
	/// let mut combiner = Combiner::pos_manual();
	///
	/// let res = combiner.add_shapes_cube("inner", (5, 5, 5), GateMode::OR, (0, 0, 0));
	///	assert!(res.is_ok()); // Success
	///
	/// let res = combiner.add("inner", GateMode::OR);
	/// assert!(res.is_err()); // This name is already taken
	/// ```
	pub fn add_shapes_cube<N, B, S, R>(&mut self, name: N, bounds: B, from_shape: S, shape_rot: R)
		-> Result<(), Error>
		where N: Into<String>, B: Into<Bounds>, S: Into<Shape>, R: Into<Rot>
	{
		self.add(name, shapes_cube(bounds, from_shape, shape_rot))
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
	/// Compiles the [`Combiner`] to a [`Scheme`], and lists of all of
	/// invalid actions performed - invalid connections, invalid inputs
	/// and outputs.
	///
	/// # Example
	/// ```
	/// # use sm_logic::shape::vanilla::GateMode;
	/// # use crate::sm_logic::combiner::Combiner;
	/// let mut combiner = Combiner::pos_manual();
	///
	/// // Simple demo scheme - just single gate wrapper.
	/// combiner.add("test", GateMode::AND).unwrap();
	/// combiner.pos().place_last((0, 0, 0));
	///
	/// combiner.pass_input("_", "test", None as Option<String>).unwrap();
	/// combiner.pass_output("_", "test", None as Option<String>).unwrap();
	///
	/// let result = combiner.compile();
	/// assert!(result.is_ok());
	/// let (scheme, invalid_acts) = result.unwrap();
	///
	/// assert_eq!(invalid_acts.connections.len(), 0);
	/// assert_eq!(invalid_acts.inp_bind_conns.len(), 0);
	/// assert_eq!(invalid_acts.out_bind_conns.len(), 0);
	/// ```
	pub fn compile(self) -> Result<(Scheme, InvalidActs), CompileError<<P as Positioner>::Error>>
	{
		// Placing schemes
		let schemes = self.positioner.arrange(self.schemes)
			.map_err(|error| CompileError::PositionerError(error))?;

		let mut invalid_acts = InvalidActs::new();
		let mut inputs_map: HashMap<String, (usize, Vec<Slot>)> = HashMap::new();
		let mut outputs_map: HashMap<String, (usize, Vec<Slot>)> = HashMap::new();

		let mut shapes: Vec<(Point, Rot, Shape)> = Vec::new();

		// Combining all schemes into new one
		for (name, (pos, rot, scheme)) in schemes {
			let start_shape = shapes.len();
			let (scheme_shapes, scheme_inps, scheme_outps) = scheme.disassemble(start_shape, pos, rot);
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
			.map(|bind| bind.compile(&outputs_map, SlotSide::Output))
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

		if !self.conns_overflow_allowed {
			// Check if some shape contains more than 255 connections
			let ovf_shapes: Vec<bool> = shapes.iter()
				.map(|(_, _, shape)| shape.connections().len() > (MAX_CONNECTIONS as usize))
				.collect();

			for (i, is_ovf) in ovf_shapes.iter().enumerate() {
				if *is_ovf {
					println!("Affected {}: conns {}", i, shapes[i].2.connections().len());
				}
			}

			// if at least one shape has connections overflow
			if ovf_shapes.iter().any(|x| *x) {
				fn check_affected_slots(ovf_shapes: &Vec<bool>, slots_map: &HashMap<String, (usize, Vec<Slot>)>) -> Vec<String> {
					let mut affected: Vec<String> = vec![];

					for (scheme_name, (start_shape, scheme_inputs)) in slots_map {
						'input: for input in scheme_inputs {
							let input_name = input.name();
							for point in input.shape_map().as_raw() {
								for shape in point {
									if ovf_shapes[*start_shape + *shape] {
										affected.push(format!("{}/{}", scheme_name, input_name));
										continue 'input;
									}
								}
							}
						}
					}

					affected
				}

				return Err(CompileError::ConnectionsOverflow {
					affected_inputs: check_affected_slots(&ovf_shapes, &inputs_map),
					affected_outputs: check_affected_slots(&ovf_shapes, &outputs_map),
					tip: {
						let msg = format!("Some slots were connected with too much other slots. \
							That resulted in connection overflow. When some shape of the scheme gets \
							more than {} connections connected to it's input or output it is called \
							connections overflow. This is bad, because it is too buggy and, probably, \
							it is not the thing you want. If you want to intentionally allow connections \
							overflow, use `allow_conns_overflow` method before compilation.", MAX_CONNECTIONS);
						match &self.debug_name {
							None => msg,
							Some(name) => format!("Combiner '{}' compilation: {}", name, msg),
						}
					}
				});
			}
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
			!is_point_in_bounds(end, to.2.bounds) ||
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
			match scheme::find_slot(slot_name.clone(), all_scheme_slots) {
				None => None,
				Some(slot) => slot.get_sector(&slot_sector_name)
					.map(|sector| (*start_shape, slot, sector))

			}
		}
	}
}
