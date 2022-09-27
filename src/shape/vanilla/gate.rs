use json::{JsonValue, object};
use crate::shape::{out_conns_to_controller, Shape, ShapeBase, ShapeBuildData};
use crate::util::{Bounds, GateMode};

pub const DEFAULT_GATE_COLOR: &str = "df7f00";
pub const GATE_UUID: &str = "9f0f56e8-2c31-4d83-996c-d00a9b296c3f";

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
		// Opinion: Scrap mechanic xaxis/zaxis rotation system works awful
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