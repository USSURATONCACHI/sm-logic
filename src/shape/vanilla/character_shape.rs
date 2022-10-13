use json::{JsonValue, object};
use crate::scheme::Scheme;
use crate::shape::{out_conns_to_controller, Shape, ShapeBase, ShapeBuildData};
use crate::util::Bounds;

pub const DEFAULT_CHARACTER_SHAPE_COLOR: &str = "df7f00";
pub const CHARACTER_SHAPE_UUID: &str = "9f0f56e8-2c31-4d83-996c-d00a9b296c3f";

#[derive(Debug, Clone)]
pub struct CharacterShape {}

impl CharacterShape {
	pub fn new() -> CharacterShape {
		CharacterShape {}
	}
}

impl ShapeBase for CharacterShape {
	fn build(&self, data: ShapeBuildData) -> JsonValue {
		let (xaxis, zaxis, offset) = data.rot.to_sm_data();
		let (x, y, z) = (data.pos + offset).tuple();

		object!{
			"color": match data.color {
				None => DEFAULT_CHARACTER_SHAPE_COLOR,
				Some(color) => color,
			},
			"shapeId": CHARACTER_SHAPE_UUID,
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
			}
		}
	}

	fn size(&self) -> Bounds {
		Bounds::new_ng(1, 1, 1)
	}

	fn has_input(&self) -> bool {
		false
	}

	fn has_output(&self) -> bool {
		false
	}
}

impl Into<Scheme> for CharacterShape {
	fn into(self) -> Scheme {
		let shape: Shape = Shape::new(Box::new(self));
		shape.into()
	}
}