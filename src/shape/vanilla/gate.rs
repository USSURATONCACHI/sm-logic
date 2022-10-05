use json::{JsonValue, object};
use crate::scheme::Scheme;
use crate::shape::{out_conns_to_controller, Shape, ShapeBase, ShapeBuildData};
use crate::util::Bounds;

pub const DEFAULT_GATE_COLOR: &str = "df7f00";
pub const GATE_UUID: &str = "9f0f56e8-2c31-4d83-996c-d00a9b296c3f";

/// Represents all possible states of Logic Gate in Scrap Mechanic
#[derive(Debug, Clone, Copy)]
pub enum GateMode {
	AND,
	OR,
	XOR,
	NAND,
	NOR,
	XNOR,
}

impl GateMode {
	/// In JSON Logic Gate state is contained as number, this method
	/// returns corresponding number.
	///
	/// # Example
	/// ```
	/// # use crate::sm_logic::shape::vanilla::GateMode;
	/// assert_eq!(GateMode::AND.to_number(), 0);
	/// assert_eq!(GateMode::OR.to_number(), 1);
	/// assert_eq!(GateMode::XOR.to_number(), 2);
	/// assert_eq!(GateMode::NAND.to_number(), 3);
	/// assert_eq!(GateMode::NOR.to_number(), 4);
	/// assert_eq!(GateMode::XNOR.to_number(), 5);
	/// ```
	pub fn to_number(self) -> usize {
		match self {
			GateMode::AND => 	0,
			GateMode::OR => 	1,
			GateMode::XOR => 	2,
			GateMode::NAND => 	3,
			GateMode::NOR => 	4,
			GateMode::XNOR => 	5,
		}
	}
}

impl Into<Shape> for GateMode {
	fn into(self) -> Shape {
		Gate::new(self)
	}
}

impl Into<Scheme> for GateMode {
	fn into(self) -> Scheme {
		let shape: Shape = self.into();
		shape.into()
	}
}

/// Represents "Logic Gate" from scrap mechanic.
///
/// # Example
/// ```
/// # use crate::sm_logic::shape::vanilla::Gate;
/// # use crate::sm_logic::shape::vanilla::GateMode;
/// let and_gate = Gate::new(GateMode::AND);
/// let or_gate = Gate::new(GateMode::OR);
/// let xor_gate = Gate::new(GateMode::XOR);
/// let nand_gate = Gate::new(GateMode::NAND);
/// let nor_gate = Gate::new(GateMode::NOR);
/// let xnor_gate = Gate::new(GateMode::XNOR);
/// ```
#[derive(Debug, Clone)]
pub struct Gate {
	mode: GateMode,
}

impl Gate {
	pub fn new(mode: GateMode) -> Shape {
		Shape::new(
			Box::new(
				Gate {
					mode
				}
			)
		)
	}
}

impl ShapeBase for Gate {
	fn build(&self, data: ShapeBuildData) -> JsonValue
	{
		// Opinion: Scrap mechanic xaxis/zaxis rotation system works really weird

		// This offset is needed because of weird behavior of xaxis-zaxis
		// rotation system of Scrap Mechanic.
		// Each xaxis-zaxis pair offsets part's position a little.
		// You can think that this is due to the fact that part is being
		// rotated around the CORNER of the block, but actually not.

		// I could not determine what xaxis, zaxis values do actually
		// represent. I cannot see any clear patterns of this and so
		// I just added all the possible values of xaxis, zaxis and
		// corresponding part position offsets to the table. this offset
		// is counteracted here

		// Also, shapes in this library are being rotated not around the
		// corner, but around the center of the block with position (0, 0, 0)
		let (xaxis, zaxis, offset) = data.rot.to_sm_data();
		let (x, y, z) = (data.pos + offset).tuple();

		object!{
			"color": match data.color {
				None => DEFAULT_GATE_COLOR,
				Some(color) => color,
			},
			"shapeId": GATE_UUID,
			"xaxis": xaxis,
			"zaxis": zaxis,
			"pos": {
				"x": x,
				"y": y,
				"z": z,
			},
			"controller": {
				"active": false,
				"id": data.id,
				"joints": null,
				"controllers": out_conns_to_controller(data.out_conns),
				"mode": self.mode.to_number()
			}
		}
	}

	fn size(&self) -> Bounds {
		Bounds::new_ng(1, 1, 1)
	}

	fn has_input(&self) -> bool {
		true
	}

	fn has_output(&self) -> bool {
		true
	}
}