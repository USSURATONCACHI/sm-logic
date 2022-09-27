use json::{JsonValue, object};
use crate::shape::{out_conns_to_controller, Shape, ShapeBase, ShapeBuildData};
use crate::util::{Bounds, Point, Rot, TICKS_PER_SECOND};


pub const DEFAULT_TIMER_COLOR: &str = "df7f00";
pub const TIMER_UUID: &str = "8f7fd0e7-c46e-4944-a414-7ce2437bb30f";


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
		// Opinion: Scrap mechanic xaxis/zaxis rotation system works awful
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