use json::{JsonValue, object};
use crate::scheme::Scheme;
use crate::shape::{Shape, ShapeBase, ShapeBuildData};
use crate::util::Bounds;


pub const DEFAULT_TOTEBOT_CAP_COLOR: &str = "49642d";
pub const TOTEBOT_CAP_UUID: &str = "34d22fc5-0a45-4d71-9aaf-64df1355c272";

#[derive(Debug, Clone)]
pub struct TotebotCapsule {}

impl TotebotCapsule {
	pub fn new() -> Shape {
		Shape::new(Box::new(TotebotCapsule {}))
	}
}

impl ShapeBase for TotebotCapsule {
	fn build(&self, data: ShapeBuildData) -> JsonValue {
		let (xaxis, zaxis, offset) = data.rot.to_sm_data();
		let (x, y, z) = (data.pos + offset).tuple();

		object!{
			"color": match data.color {
				None => DEFAULT_TOTEBOT_CAP_COLOR,
				Some(color) => color,
			},
			"shapeId": TOTEBOT_CAP_UUID,
			"xaxis": xaxis,
			"zaxis": zaxis,
			"pos": {
				"x": x,
				"y": y,
				"z": z,
			},
			"controller": {
				"containers": null,
				"controllers": null,
				"joints": null,
				"id": data.id,
			}
		}
	}

	fn size(&self) -> Bounds {
		Bounds::new_ng(7, 7, 6)
	}

	fn has_input(&self) -> bool {
		false
	}

	fn has_output(&self) -> bool {
		false
	}
}

impl Into<Shape> for TotebotCapsule {
	fn into(self) -> Shape {
		Shape::new(Box::new(self))
	}
}

impl Into<Scheme> for TotebotCapsule {
	fn into(self) -> Scheme {
		let shape: Shape = self.into();
		shape.into()
	}
}