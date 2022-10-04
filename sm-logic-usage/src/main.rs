#![allow(dead_code)]

use sm_logic::bind::Bind;
use sm_logic::combiner::*;
use sm_logic::scheme::Scheme;
use sm_logic::positioner::ManualPos;
use sm_logic::util::GateMode::{AND, OR, XOR};
use sm_logic::util::Facing;

fn main() {
	match test().compile() {
		Err(e) => println!("Fail: {:?}", e),
		Ok((scheme, invalid_acts)) => {
			println!("\nInvalid conns:");
			for conn in invalid_acts.connections {
				println!("\t{:?}", conn);
			}
			println!("\nInvalid inputs:");
			for (name, conn) in invalid_acts.inp_bind_conns {
				println!("\t{} - {:?}", name, conn);
			}
			println!("\nInvalid outputs:");
			for (name, conn) in invalid_acts.out_bind_conns {
				println!("\t{} - {:?}", name, conn);
			}

			println!("Success.");
			println!("Shapes count: {}", scheme.shapes_count());
			println!("Inputs:");
			for inp in scheme.inputs() {
				println!("\t{} - {:?}", inp.name(), inp.bounds().tuple());
			}

			println!("\nOutputs:");
			for inp in scheme.outputs() {
				println!("\t{} - {:?}", inp.name(), inp.bounds().tuple());
			}

			println!("Writing to json...");
			let json = scheme.to_json().to_string();
			std::fs::write(r#"C:\Users\redch\AppData\Roaming\Axolot Games\Scrap Mechanic\User\User_76561198288016737\Blueprints\fd551543-b05e-487f-9db5-1583ffc826d0\blueprint.json"#, json).unwrap();
			println!("Done");
		}
	}
}

fn test() -> Combiner<ManualPos> {
	let adder = adder(8);
	let mut combiner = Combiner::pos_manual();

	for i in 0..4 {
		combiner.add(format!("{}", i), adder.clone()).unwrap();
		combiner.pos().place_last((0, 0, i * 2));
		combiner.pos().rotate_last((0, 0, i));
		combiner.pass_input(format!("{}_a", i), format!("{}/a", i), None as Option<String>).unwrap();
		combiner.pass_input(format!("{}_b", i), format!("{}/b", i), None as Option<String>).unwrap();
		combiner.pass_output(format!("{}", i), format!("{}", i), None as Option<String>).unwrap();
	}

	combiner
}
/*
fn memory(word_size: u32, size: (u32, u32, u32)) -> Combiner<ManualPos> {
	let cell = memory_cell(word_size);
	let cell_size = cell.bounds().cast::<i32>();

	//let cells_count = size.0 * size.1 * size.2;

	let mut combiner = Combiner::pos_manual();

	// TODO: combiner.expand_slot(path, amount_of_slots, placer: Fn(id) -> position);

	combiner.create_slot_scheme("write_data_inner", "binary", size, AND, Facing::NegX.to_rot()).unwrap();
	combiner.pos().place_last((-(size.0 as i32), 0, size.2 as i32 + 1));

	combiner.create_slot_scheme("write_data", "binary", size, OR, Facing::NegX.to_rot()).unwrap();
	combiner.pos().place_last((-(size.0 as i32), 0, 0));

	combiner.create_slot_scheme("read_data", "binary", size, OR, Facing::NegX.to_rot()).unwrap();
	combiner.pos().place_last((-(size.0 as i32), (size.1 as i32) + 1, 0));

	combiner.add("write", OR).unwrap();
	combiner.pos().place_last((-(size.0 as i32), (size.1 as i32) * 2 + 2, 0));

	for x in 0..size.0 {
		for y in 0..size.1 {
			for z in 0..size.2 {
				let id = x + y * size.1 + z * size.2 * size.1;

				let name = format!("{}", id);
				combiner.add(&name, cell.clone()).unwrap();
				combiner.pos().place_last(cell_size * Point::new(x as i32, y as i32, z as i32));
			}
		}
	}

	combiner
}*/

fn memory_cell(word_size: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	let mut input_bind = Bind::new("data", "binary", (word_size, 1, 1));
	let mut output_bind = Bind::new("_", "binary", (word_size, 1, 1));

	for i in 0..word_size {
		let input_name = format!("{}_input", i);
		combiner.add(&input_name, AND).unwrap();
		combiner.pos().place_last((0, 0, i as i32));
		combiner.pos().rotate_last(Facing::NegY.to_rot());
		combiner.connect("activate", &input_name);

		input_bind.connect(((i as i32, 0, 0), (1, 1, 1)), &input_name);

		let output_name = format!("{}_output", i);
		combiner.add(&output_name, AND).unwrap();
		combiner.pos().place_last((1, 0, i as i32));
		combiner.pos().rotate_last(Facing::NegY.to_rot());
		combiner.connect("activate", &output_name);

		output_bind.connect(((i as i32, 0, 0), (1, 1, 1)), &output_name);

		let xor_name = format!("{}_xor", i);
		combiner.add(&xor_name, XOR).unwrap();
		combiner.pos().place_last((2, 0, i as i32));
		combiner.pos().rotate_last(Facing::NegY.to_rot());
		combiner.connect(&xor_name, &xor_name);
		combiner.connect(&input_name, &xor_name);
		combiner.connect(&xor_name, &output_name);
	}

	combiner.add("activate", AND).unwrap();
	combiner.pos().place_last((1, 0, word_size as i32));
	combiner.pos().rotate_last(Facing::NegY.to_rot());
	combiner.pass_input("activate", "activate", Some("logic")).unwrap();

	combiner.bind_input(input_bind).unwrap();
	combiner.bind_output(output_bind).unwrap();

	let (scheme, _) = combiner.compile().unwrap();
	scheme
}

fn adder(word_size: u32) -> Scheme {
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
	}
	adder.bind_input(bind).unwrap();

	let mut bind = Bind::new("b", "binary", (word_size, 1u32, 1u32));
	bind.connect_func(|x, _, _| Some(format!("{}/b", x)));
	adder.bind_input(bind).unwrap();

	let mut bind = Bind::new("_", "binary", (word_size, 1u32, 1u32));
	bind.connect_func(|x, _, _| Some(format!("{}/res", x)));
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

/*
	let mut combiner = Combiner::pos_manual();

	combiner.add("main", AND).unwrap();
	combiner.pos().place_last((0, 0, 10));
	combiner.pos().rotate_last(Facing::NegZ.to_rot());

	for x in 0..128 {
		for y in 0..128 {
			let name = format!("{}_{}", x, y);
			combiner.add(&name, OR).unwrap();
			combiner.pos().place_last((x, y, 0));
			combiner.pos().rotate_last(Facing::PosZ.to_rot());
			combiner.connect("main", name);
		}
	}

	combiner

*/