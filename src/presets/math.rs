use crate::bind::Bind;
use crate::combiner::Combiner;
use crate::connection::{ConnDim, Connection, ConnMap};
use crate::scheme::Scheme;
use crate::shape::vanilla::{BlockBody, BlockType, GateMode};
use crate::shape::vanilla::GateMode::{AND, NOR, OR, XOR};
use crate::util::{Facing, Point, Rot};

pub fn multiplier(bits_before_point: u32, bits_after_point: u32) -> Scheme {
	let size = bits_before_point + bits_after_point;

	let mut combiner = Combiner::pos_manual();

	let slots_kind = format!("binary[{}.{}]", bits_before_point, bits_after_point);

	let line = gates_line(OR, size, &slots_kind, Facing::NegX.to_rot());
	combiner.add("a", line.clone()).unwrap();
	combiner.pos().place_last((0, 0, 0));
	combiner.pass_input("a", "a", None as Option<String>).unwrap();

	combiner.add("b", line.clone()).unwrap();
	combiner.pos().place_last((0, 0, 1));
	combiner.pass_input("b", "b", None as Option<String>).unwrap();

	let mut prev_step: Vec<(i32, i32, String)> = vec![];

	for i in 0..(size as i32) {
		let offset = (i as i32) - (bits_after_point as i32);
		let line = line_segment(
			offset, offset + (size as i32), size,
			AND, &slots_kind, Facing::NegX.to_rot()
		);

		let name = format!("table_{}", i);
		combiner.add(&name, line).unwrap();
		combiner.pos().place_last((1, 0, i as i32));

		combiner.custom("a", &name, ConnMap::new(
			move |(point, _), _| Some(point + Point::new_ng(offset, 0, 0))
		));

		combiner.dim(format!("b/_/{}", i), &name, (true, true, true));
		prev_step.push((offset, offset + (size as i32), name));
	}


	let mut iteration = 0;
	while prev_step.len() > 1 {
		let mut i = 0;
		let mut step: Vec<(i32, i32, String)> = vec![];
		let mut collection = prev_step.into_iter();

		'iteration: loop {
			let slot_a = collection.next();
			let slot_b = collection.next();

			if slot_a.is_none() {
				break 'iteration;
			}
			let slot_a = slot_a.unwrap();

			match slot_b {
				Some(slot_b) => {
					let (min_a, max_a, slot_a) = slot_a;
					let (min_b, max_b, slot_b) = slot_b;
					let min = min_a.min(min_b);
					let max = max_a.max(max_b);

					let adder = adder_segment(min, max, size);
					let adder_size = adder.bounds().cast::<i32>().tuple();

					let name = format!("adder_{}_{}", iteration, i);
					combiner.add(&name, adder.clone()).unwrap();
					combiner.pos().place_last((2 + iteration * adder_size.0, 0, i * adder_size.2));


					combiner.connect(slot_a, format!("{}/a", name));
					combiner.connect(slot_b, format!("{}/b", name));
					step.push((min, max, name));
				}

				None => step.push(slot_a)
			}
			i += 1;
		}
		prev_step = step;
		iteration += 1;
	}

	match prev_step.len() {
		1 => {
			let (_, _, path) = prev_step.get(0).unwrap();
			combiner.pass_output("_", path, Some(slots_kind)).unwrap();
		}
		0 => {
			combiner.bind_output(Bind::new("_", slots_kind, (size, 1, 1))).unwrap();
		}
		_ => panic!("Something went wrong"),
	}

	let (scheme, _) = combiner.compile().unwrap();
	scheme
}

fn adder_segment(from: i32, to: i32, max_size: u32) -> Scheme {
	let size = (to - from.max(0)).max(0).min((max_size as i32) - from.max(0));
	let adder = adder(size as u32);

	let mut combiner = Combiner::pos_manual();
	combiner.add("adder", adder).unwrap();
	combiner.pos().place_last((0, from.max(0), 0));

	let glass_prev = from.max(0).min(max_size as i32);

	if glass_prev > 0 {
		combiner.add("glass_pre", BlockBody::new(BlockType::Glass, (1, 1, 1))).unwrap();
		combiner.pos().place_last((0, 0, 0));
	}

	let offset = from.max(0);
	let mut inp_a = Bind::new("a", "binary", (max_size, 1, 1));
	inp_a.connect_func(|x, _, _| Some(format!("adder/a/{}", (x as i32) - offset)));

	let mut inp_b = Bind::new("b", "binary", (max_size, 1, 1));
	inp_b.connect_func(|x, _, _| Some(format!("adder/b/{}", (x as i32) - offset)));

	let mut output = Bind::new("_", "binary", (max_size, 1, 1));
	output.connect_func(|x, _, _| Some(format!("adder/_/{}", (x as i32) - offset)));

	combiner.bind_input(inp_a).unwrap();
	combiner.bind_input(inp_b).unwrap();
	combiner.bind_output(output).unwrap();

	let (scheme, _) = combiner.compile().unwrap();
	scheme
}

fn line_segment<R: Into<Rot>, S: Into<String>>(
	from: i32, to: i32, max_size: u32,
	mode: GateMode, slot_kind: S, rot: R
) -> Scheme {
	let mut combiner = Combiner::pos_manual();
	let rot = rot.into();
	let slot_kind = slot_kind.into();

	let mut input = Bind::new("_", &slot_kind, (max_size, 1, 1));
	input.connect_func(|x, _, _| Some(format!("{}", x)));

	let mut output = Bind::new("_", slot_kind, (max_size, 1, 1));
	output.connect_func(|x, _, _| Some(format!("{}", x)));

	for i in 0..max_size {
		let i = i as i32;

		if i >= from && i < to {
			let name = format!("{}", i);
			combiner.add(&name, mode).unwrap();

			input.add_sector(&name, (i as i32, 0, 0), (1, 1, 1), "bit").unwrap();
			output.add_sector(name, (i as i32, 0, 0), (1, 1, 1), "bit").unwrap();
		} else {
			let name = format!("_{}", i);
			combiner.add(&name, BlockBody::new(BlockType::Glass, (1, 1, 1))).unwrap();
		}
		combiner.pos().place_last((0, i as i32, 0));
		combiner.pos().rotate_last(rot.clone());
	}

	combiner.bind_input(input).unwrap();
	combiner.bind_output(output).unwrap();

	let (scheme, _) = combiner.compile().unwrap();
	scheme
}

fn gates_line<R: Into<Rot>, S: Into<String>>(mode: GateMode, size: u32, slot_kind: S, rot: R) -> Scheme {
	let mut combiner = Combiner::pos_manual();
	let rot = rot.into();
	let slot_kind = slot_kind.into();

	let mut input = Bind::new("_", &slot_kind, (size, 1, 1));
	input.connect_func(|x, _, _| Some(format!("{}", x)));

	let mut output = Bind::new("_", slot_kind, (size, 1, 1));
	output.connect_func(|x, _, _| Some(format!("{}", x)));

	for i in 0..size {
		let name = format!("{}", i);
		combiner.add(&name, mode).unwrap();
		combiner.pos().place_last((0, i as i32, 0));
		combiner.pos().rotate_last(rot.clone());

		input.add_sector(&name, (i as i32, 0, 0), (1, 1, 1), "bit").unwrap();
		output.add_sector(name, (i as i32, 0, 0), (1, 1, 1), "bit").unwrap();
	}

	combiner.bind_input(input).unwrap();
	combiner.bind_output(output).unwrap();

	let (scheme, _) = combiner.compile().unwrap();
	scheme
}

#[allow(dead_code)]
pub fn inverter(word_size: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	combiner.add_iter([
		("-1_next", 	NOR),
		("no_inputs", 	AND),
	]).unwrap();

	combiner.pos().place_iter([
		("-1_next", 	(2, -1, 0)),
		("no_inputs", 	(1, -1, 0)),
	]);

	combiner.connect("no_inputs", "-1_next");

	for i in 0..word_size {
		combiner.add_iter([
			(format!("{}_inp", i), 	OR),
			(format!("{}_nor", i), 	NOR),
			(format!("{}_next", i), AND),
			(format!("{}_out", i), 	XOR),
		]).unwrap();

		combiner.connect(format!("{}_inp", i), format!("{}_nor", i));
		combiner.connect(format!("{}_nor", i), format!("{}_out", i));
		combiner.connect(format!("{}_nor", i), format!("{}_next", i));

		combiner.connect(format!("{}_next", (i as i32) - 1), format!("{}_next", i));
		combiner.connect(format!("{}_next", (i as i32) - 1), format!("{}_out", i));

		combiner.pos().place_iter([
			(format!("{}_inp", i), 	(0, i as i32, 0)),
			(format!("{}_nor", i), 	(1, i as i32, 0)),
			(format!("{}_next", i), (2, i as i32, 0)),
			(format!("{}_out", i), 	(3, i as i32, 0)),
		]);

		combiner.pos().rotate(format!("{}_inp", i), Facing::NegX.to_rot());
		combiner.pos().rotate(format!("{}_out", i), Facing::PosX.to_rot());
	}

	let mut bind = Bind::new("_", "binary", (word_size, 1, 1));
	bind.connect_func(|x, _, _| Some(format!("{}_inp", x)));
	bind.gen_point_sectors("bit", |x, _, _| format!("{}", x)).unwrap();
	combiner.bind_input(bind).unwrap();

	let mut bind = Bind::new("_", "binary", (word_size, 1, 1));
	bind.connect_func(|x, _, _| Some(format!("{}_out", x)));
	bind.gen_point_sectors("bit", |x, _, _| format!("{}", x)).unwrap();
	combiner.bind_output(bind).unwrap();

	let (scheme, _) = combiner.compile().unwrap();
	scheme
}

#[allow(dead_code)]
pub fn adder(word_size: u32) -> Scheme {
	let section = adder_section();

	let mut adder = Combiner::pos_manual();

	for i in 0..word_size {
		adder.add(format!("{}", i), section.clone()).unwrap();
		adder.pos().place_last((0, i as i32, 0));

		adder.connect(format!("{}", i), format!("{}", i+1));
	}

	let mut bind = Bind::new("a", "binary", (word_size, 1u32, 1u32));
	for x in 0..(word_size as i32) {
		bind.connect(((x, 0, 0), (1, 1, 1)), format!("{}/a", x));
		bind.add_sector(format!("{}", x), (x, 0, 0), (1, 1, 1), "binary").unwrap();
	}
	adder.bind_input(bind).unwrap();

	let mut bind = Bind::new("b", "binary", (word_size, 1u32, 1u32));
	for x in 0..(word_size as i32) {
		bind.connect(((x, 0, 0), (1, 1, 1)), format!("{}/b", x));
		bind.add_sector(format!("{}", x), (x, 0, 0), (1, 1, 1), "binary").unwrap();
	}
	adder.bind_input(bind).unwrap();

	let mut bind = Bind::new("_", "binary", (word_size, 1u32, 1u32));
	bind.connect_func(|x, _, _| Some(format!("{}/res", x)));
	bind.gen_point_sectors("bit", |x, _, _| format!("{}", x)).unwrap();
	adder.bind_output(bind).unwrap();

	let (scheme, _) = adder.compile().unwrap();
	scheme
}

fn adder_section() -> Scheme {
	let mut s = Combiner::pos_manual();

	s.add_mul(["a", "b", "_"], OR).unwrap();
	s.add_mul(["and_1", "and_2", "and_3"], AND).unwrap();
	s.add("res", XOR).unwrap();

	s.pos().place_iter([
		("a", (0, 0, 0)),
		("b", (0, 0, 1)),
		("_", (2, 0, 1)),
		("and_1", (1, 0, 0)),
		("and_2", (1, 0, 1)),
		("and_3", (2, 0, 0)),
		("res", (3, 0, 0)),
	]);

	s.pos().rotate("a", Facing::NegX.to_rot());
	s.pos().rotate("b", Facing::NegX.to_rot());
	s.pos().rotate("res", Facing::PosX.to_rot());

	let mut bind = Bind::new("a", "bit", (1, 1, 1));
	bind.connect_full("a");
	s.bind_input(bind).unwrap();

	let mut bind = Bind::new("b", "bit", (1, 1, 1));
	bind.connect_full("b");
	s.bind_input(bind).unwrap();

	let mut bind = Bind::new("_", "bit", (1, 1, 1));
	bind.connect_full("_");
	s.bind_input(bind).unwrap();

	s.connect_iter(["a", "b", "_"], ["res"]);

	s.connect_iter(["a"], ["and_1", "and_2"]);
	s.connect_iter(["b"], ["and_2", "and_3"]);
	s.connect_iter(["_"], ["and_3", "and_1"]);

	let mut bind = Bind::new("_", "bit", (1, 1, 1));
	bind.connect_full("and_1")
		.connect_full("and_2")
		.connect_full("and_3");
	s.bind_output(bind).unwrap();

	let mut bind = Bind::new("res", "bit", (1, 1, 1));
	bind.connect_full("res");
	s.bind_output(bind).unwrap();

	let (scheme, _) = s.compile().unwrap();
	scheme
}