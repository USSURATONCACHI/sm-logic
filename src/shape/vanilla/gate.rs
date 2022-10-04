use json::{JsonValue, object};
use crate::shape::{out_conns_to_controller, Shape, ShapeBase, ShapeBuildData};
use crate::util::{Bounds, GateMode};

pub const DEFAULT_GATE_COLOR: &str = "df7f00";
pub const GATE_UUID: &str = "9f0f56e8-2c31-4d83-996c-d00a9b296c3f";

/// Represents "Logic Gate" from scrap mechanic.
///
/// # Example
/// ```
/// # use crate::sm_logic::shape::vanilla::Gate;
/// # use crate::sm_logic::util::GateMode;
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

		// This offset is needed due to weird behavior of xaxis-zaxis
		// rotation system of Scrap Mechanic.
		// Each xaxis-zaxis pair offsets part's position a little.
		// You can think that this is due to the fact that part is being
		// rotated around the CORNER of the block, but actually not.

		// I could not determine what xaxis, zaxis values do actually
		// represent. I cannot see any clear patterns of this and so
		// I just added all the possible values of xaxis, zaxis and
		// corresponding part position offsets to the table. this offset
		// is counteracted here
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