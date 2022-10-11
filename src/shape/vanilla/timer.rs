use json::{JsonValue, object};
use crate::scheme::Scheme;
use crate::shape::{out_conns_to_controller, Shape, ShapeBase, ShapeBuildData};
use crate::util::{Bounds, TICKS_PER_SECOND};


pub const DEFAULT_TIMER_COLOR: &str = "df7f00";
pub const TIMER_UUID: &str = "8f7fd0e7-c46e-4944-a414-7ce2437bb30f";

/// Represents "Timer" from scrap mechanic.
///
/// # Example
/// ```
/// # use crate::sm_logic::shape::vanilla::Timer;
/// // 10 seconds + 10 ticks
/// let timer_a = Timer::new(410);
/// let timer_b = Timer::from_time(10, 10);
/// ```
#[derive(Debug, Clone)]
pub struct Timer {
	seconds: u32,
	ticks: u32,
}

impl Timer {
	pub fn new(ticks: u32) -> Shape {
		Timer::from_time(
			ticks / TICKS_PER_SECOND,
			ticks % TICKS_PER_SECOND,
		)
	}

	pub fn from_time(seconds: u32, ticks: u32) -> Shape {
		Shape::new(
			Box::new(
				Timer {
					seconds,
					ticks
				}
			)
		)
	}
}

impl ShapeBase for Timer {
	fn build(&self, data: ShapeBuildData) -> JsonValue {
		let (xaxis, zaxis, offset) = data.rot.to_sm_data();
		let (x, y, z) = (data.pos + offset).tuple();

		object!{
			"color": match data.color {
				None => DEFAULT_TIMER_COLOR,
				Some(color) => color,
			},
			"shapeId": TIMER_UUID,
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
				"seconds": self.seconds,
				"ticks": self.ticks,
			}
		}
	}

	fn size(&self) -> Bounds {
		Bounds::new_ng(1, 1, 2)
	}

	fn has_input(&self) -> bool {
		true
	}

	fn has_output(&self) -> bool {
		true
	}
}

impl Into<Shape> for Timer {
	fn into(self) -> Shape {
		Shape::new(Box::new(self))
	}
}

impl Into<Scheme> for Timer {
	fn into(self) -> Scheme {
		let shape: Shape = self.into();
		shape.into()
	}
}