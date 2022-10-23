use crate::bind::Bind;
use crate::combiner::Combiner;
use crate::positioner::ManualPos;
use crate::presets::{binary_selector_compact, Scheme};
use crate::shape::vanilla::{BlockBody, BlockType};
use crate::shape::vanilla::GateMode::*;
use crate::util::{Facing, MAX_CONNECTIONS, Point};

/// ***Inputs***: data, write.
///
/// ***Outputs***: _ (memory).

///
/// Simple and fast memory cell.
///
/// Data is available on default output.
/// To write data you need to send 1-tick logic signal 'write' and
/// the data itself in the same tick to 'data'. Data in memory cannot
/// be changed, until 'write' input is activated.
pub fn xor_mem_cell(size: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	combiner.add("cell", incomplete_xor_mem_cell(size, 1)).unwrap();
	combiner.pos().place_last((0, 0, 0));

	combiner.add("write", OR).unwrap();
	combiner.pos().place_last((0, size as i32, 1));
	combiner.dim("write", "cell/write_0", (true, true, true));

	combiner.pass_input("write", "write", Some("logic")).unwrap();
	combiner.pass_input("data", "cell/data_0", None as Option<String>).unwrap();
	combiner.pass_output("_", "cell", None as Option<String>).unwrap();

	let (scheme, _invalid) = combiner.compile().unwrap();
	scheme
}

/// ***Inputs***: data_0, write_0, data_1, write_1, data_2, write_2, etc...
///
/// (ATTENTION! 'write' is (size, 1, 1) slot,
/// not (1, 1, 1)).
///
/// ***Outputs***: _ (memory).

///
/// Simply `xor_mem_cell`, but without 'write' OR gate. Also there is
/// variable amount of write modules.
pub fn incomplete_xor_mem_cell(size: u32, write_modules_count: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	combiner.add_shapes_cube("memory", (size, 1, 1), XOR, Facing::PosZ.to_rot()).unwrap();
	combiner.connect("memory", "memory");
	combiner.pos().place_last((0, 0, (write_modules_count as i32) * 2));
	combiner.pos().rotate_last((0, 0, 1));

	for i in 0..write_modules_count {
		let compare = format!("compare_{}", i);
		let approve = format!("approve_{}", i);
		combiner.add_shapes_cube(&compare, (size, 1, 1), XOR, Facing::PosY.to_rot()).unwrap();
		combiner.add_shapes_cube(&approve, (size, 1, 1), AND, Facing::PosY.to_rot()).unwrap();

		combiner.connect("memory", &compare);
		combiner.connect(&compare, &approve);
		combiner.connect(&approve, "memory");

		let mut write = Bind::new(format!("write_{}", i), "logic", (size, 1, 1));
		write.connect_full(&approve);
		write.gen_point_sectors("logic", |x, _, _| x.to_string()).unwrap();
		combiner.bind_input(write).unwrap();

		combiner.pos().place_iter([
			(&compare, (0, 0, (i as i32) * 2)),
			(&approve, (0, 0, (i as i32) * 2 + 1)),
		]);
		combiner.pos().rotate_iter([
			(&compare, (0, 0, 1)),
			(&approve, (0, 0, 1)),
		]);

		let mut input = Bind::new(format!("data_{}", i), "_", (size, 1, 1));
		input.connect_full(&compare);
		input.gen_point_sectors("_", |x, _, _| x.to_string()).unwrap();
		combiner.bind_input(input).unwrap();
	}

	let mut output = Bind::new("_", "_", (size, 1, 1));
	output.connect_full("memory");
	output.gen_point_sectors("_", |x, _, _| x.to_string()).unwrap();

	combiner.bind_output(output).unwrap();

	let (scheme, _invalid) = combiner.compile().unwrap();
	scheme
}

/// ***Inputs***: address, write. Possibly direct memory inputs
/// ('0', '1', '2'...) that lead right into memory gates.
///
/// ***Outputs***: _ (read). Possibly direct memory outputs
/// ('0', '1', '2'...) that lead right into memory gates.

///
/// A bunch of connected memory cells (not `xor_memory_cell`,
/// but `smallest_rw_cell`). Amount of cells is product of each
/// coordinate of `size` argument.
///
/// 'address' input is binary word to select some specific cell.
/// Size of address is determined as `cells_count.log2().ceil()`;
///
/// 'write' input bus will just pass data to selected cell.
///
/// Default output just shows data in selected cell (3 ticks delay
/// between input address signal and output data signal).
/// Memory block allows for 1-tick threaded reading with 3 ticks delay.
/// That means that you can send different address each tick and exactly
/// 3 ticks later corresponding data will be available each tick.
///
/// `make_direct_inputs` and `make_direct_outputs` will add inputs and
/// outputs that lead right to memory gates. Those slots are named as
/// '0', '1', '2' and so on for each cell.
///
/// Also: will cause connections overflow if there is more than 65 025
/// memory cells (`MAX_CONNECTIONS.pow(2)`). I assume you won't need so
/// much, since Scrap Mechanic won't preform very well with such amount
/// of gates.
pub fn raw_memory_block(word_size: u32, size: (u32, u32, u32), make_direct_inputs: bool, make_direct_outputs: bool) -> Scheme {
	let mut combiner = Combiner::pos_manual();
	let cell = smallest_rw_cell(word_size);

	let cells_count = size.0 * size.1 * size.2;
	let address_size = (cells_count as f64).log2().ceil() as u32;

	// Add all memory cells to the combiner
	let all_cells: Vec<String> = add_cells(&mut combiner, cell, size)
		.into_iter().map(|(cell, _pos)| cell).collect();

	// Create cell selector
	let cell_selector = binary_selector_compact(address_size);
	combiner.add("address", cell_selector).unwrap();

	let mut address_bind = Bind::new("address", "binary", (address_size, 1, 1));
	address_bind.gen_point_sectors("bit", |x, _, _| x.to_string()).unwrap();
	address_bind.connect_full("address");
	combiner.bind_input(address_bind).unwrap();

	// Add read and write data buses
	let mut input = Bind::new("write", "_", (word_size, 1, 1));
	let mut output = Bind::new("_", "_", (word_size, 1, 1)); //read

	input.gen_point_sectors("_", |x, _, _| x.to_string()).unwrap();
	output.gen_point_sectors("_", |x, _, _| x.to_string()).unwrap();

	combiner.pos().place_iter([
		("address", (0, -2, 0)),
		("write", 	(-2, 0, 0)),
		("read", 	(-2, 1, 0)),
	]);

	combiner.pos().rotate_iter([
		("address", (1, 0, 1)),
		("write", (0, -1, 0)),
		("read", (0, -1, 0)),
	]);

	let mut read_name = format!("read_none");
	let mut write_name = format!("write_none");

	// Connect selector to each cell
	for (i, cell) in all_cells.iter().enumerate() {
		combiner.connect(format!("address/{}", i), format!("{}/activate", cell));

		if make_direct_inputs {
			combiner.pass_input(i.to_string(), format!("{}/xor_gates", cell), Some("_")).unwrap();
		}
		if make_direct_outputs {
			combiner.pass_output(i.to_string(), format!("{}/xor_gates", cell), Some("_")).unwrap();
		}

		let bus_branch_id = (i as u32) / MAX_CONNECTIONS;

		if (i as u32) % MAX_CONNECTIONS == 0 {
			// If new bus branch is needed, add it
			read_name = format!("read_{}", bus_branch_id);
			write_name = format!("write_{}", bus_branch_id);
			combiner.add_shapes_cube(&read_name, (word_size, 1, 1), OR, Facing::PosZ.to_rot()).unwrap();
			combiner.add_shapes_cube(&write_name, (word_size, 1, 1), OR, Facing::PosZ.to_rot()).unwrap();
			output.connect_full(&read_name);
			input.connect_full(&write_name);

			combiner.pos().place(&read_name, (-1, (bus_branch_id as i32) * 2, 0));
			combiner.pos().place(&write_name, (-1, (bus_branch_id as i32) * 2 + 1, 0));
			combiner.pos().rotate(&read_name, (0, -1, 0));
			combiner.pos().rotate(&write_name, (0, -1, 0));
		}

		combiner.connect(cell, &read_name);
		combiner.connect(&write_name, format!("{}/data", cell));
	}

	combiner.bind_input(input).unwrap();
	combiner.bind_output(output).unwrap();

	let (scheme, _invalid) = combiner.compile().unwrap();
	// println!("Invalid conns: {:?}", invalid.connections);
	scheme
}

/// ***Inputs***: address, write, apply.Possibly direct memory inputs
/// ('0', '1', '2'...) that lead right into memory gates.
///
/// ***Outputs***: _ (read). Possibly direct memory outputs
/// ('0', '1', '2'...) that lead right into memory gates.

///
/// A simple add-on to `raw_memory_block` to allow for more convenient
/// usage of memory. Amount of memory cells is product of each coordinate
/// of `size` argument.
///
/// To read data, you need to select memory cell address
/// ('address' input, binary number). Delay between address input signal
/// and output data read signal is 5 ticks (read data is on default
/// output).
///
/// `array` allows for 1-tick threaded reading with 5 ticks delay.
/// That means that you can send different address each tick and exactly
/// 5 ticks later corresponding data will be available each tick.
///
/// Process of ***writing*** is way slower and does not allow for
/// threading. To write you need to select the memory cell and wait for
/// 5 ticks until it is read (You need to keep address signal constant
/// for the whole time of writing!). After 5 ticks you can send 1-tick
/// (not multiple ticks long) signal to 'apply'. When the 'apply' signal
/// hits, it will use data from 'write' input to write. Also, writing is
/// 2-ticks long (+5 ticks to select in first place). It is allowed to
/// send synchronized 'write' and 'apply' as 1-tick data, that will
/// work.
///
/// Also: will cause connections overflow if there is more than 65 025
/// memory cells (`MAX_CONNECTIONS.pow(2)`). I assume you won't need so
/// much, since Scrap Mechanic won't preform very well with such amount
/// of gates.
pub fn array(word_size: u32, size: (u32, u32, u32), make_direct_inputs: bool, make_direct_outputs: bool) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	combiner.add("mem", raw_memory_block(word_size, size, make_direct_inputs, make_direct_outputs)).unwrap();

	let cells_count = size.0 * size.1 * size.2;
	let address_size = (cells_count as f64).log2().ceil() as u32;

	if make_direct_outputs || make_direct_inputs {
		for i in 0..cells_count {
			if make_direct_inputs {
				combiner.pass_input(i.to_string(), format!("mem/{}", i), None as Option<String>).unwrap();
			}
			if make_direct_outputs {
				combiner.pass_output(i.to_string(), format!("mem/{}", i), None as Option<String>).unwrap();
			}
		}
	}

	combiner.add_shapes_cube("address", (address_size, 1, 1), OR, Facing::PosZ.to_rot()).unwrap();
	combiner.add_shapes_cube("write", (address_size, 1, 1), OR, Facing::PosZ.to_rot()).unwrap();
	combiner.add_shapes_cube("read", (address_size, 1, 1), OR, Facing::PosZ.to_rot()).unwrap();
	combiner.add("apply", OR).unwrap();

	combiner.add_shapes_cube("compare", (address_size, 1, 1), XOR, Facing::PosZ.to_rot()).unwrap();
	combiner.add_shapes_cube("pass_data", (address_size, 1, 1), AND, Facing::PosZ.to_rot()).unwrap();
	combiner.add("apply_delay", OR).unwrap();

	combiner.connect("address", "mem/address");
	combiner.connect("write", "compare");
	combiner.connect("compare", "pass_data");
	combiner.connect("pass_data", "mem/write");
	combiner.connect_iter(["mem"], ["read", "compare"]);
	combiner.connect("apply", "apply_delay");
	combiner.dim("apply_delay", "pass_data", (true, true, true));

	combiner.pass_input("address", "address", Some("binary")).unwrap();
	combiner.pass_input("write", "write", Some("_")).unwrap();
	combiner.pass_input("apply", "apply", Some("logic")).unwrap();
	combiner.pass_output("_", "read", Some("_")).unwrap();

	combiner.pos().place_iter([
		("mem", 		(1, 0, 0)),
		("address", 	(0, 0, 0)),
		("write", 		(0, 1, 0)),
		("read", 		(0, 2, 0)),
		("apply", 		(0, 3, 0)),
		("apply_delay", (0, 3, 1)),
		("compare",		(0, 4, 0)),
		("pass_data",	(0, 5, 0)),
	]);

	combiner.pos().rotate_iter([
		("address", 	(0, -1, 0)),
		("write", 		(0, -1, 0)),
		("read", 		(0, -1, 0)),
		("compare",		(0, -1, 0)),
		("pass_data",	(0, -1, 0)),
		("apply", 		(0, -1, 0)),
		("apply_delay", (0, -1, 0)),
	]);

	let (scheme, _invalid_conns) = combiner.compile().unwrap();
	scheme
}

/// ***Inputs***: activate, data, xor_gates.
///
/// ***Outputs***: _ (data), xor_gates.

///
/// Name means "Smallest read/write memory cell". Its purpose is to be
/// used in memory blocks.
///
/// Have 'activate' input. If cell is not activated it won't read data
/// from input or send data to output. When activated, memory content
/// is available on default output and it can be changed with 'data'
/// input.
///
/// 'xor_gates' output leads right to memory gates.
pub fn smallest_rw_cell(word_size: u32) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	combiner.add_shapes_cube("input", (word_size, 1, 1), AND, Facing::NegY.to_rot()).unwrap();
	combiner.add_shapes_cube("output", (word_size, 1, 1), AND, Facing::NegY.to_rot()).unwrap();
	combiner.add_shapes_cube("memory", (word_size, 1, 1), XOR, Facing::NegY.to_rot()).unwrap();

	combiner.connect("input", "memory");
	combiner.connect_iter(["memory"], ["output", "memory"]);
	combiner.dim_iter(["activate"], ["input", "output"], (true, true, true));

	combiner.add("activate", AND).unwrap();
	combiner.pos().rotate_last(Facing::NegY.to_rot());

	combiner.pass_input("activate", "activate", Some("logic")).unwrap();
	combiner.pass_input("data", "input", Some("_")).unwrap();
	combiner.pass_output("_", "output", Some("_")).unwrap();

	combiner.pass_input("xor_gates", "memory", Some("_")).unwrap();
	combiner.pass_output("xor_gates", "memory", Some("_")).unwrap();

	combiner.pos().place_iter([
		("activate", (1, 0, word_size as i32)),
		("input", (0, 0, 0)),
		("output", (1, 0, 0)),
		("memory", (2, 0, 0)),
	]);

	combiner.pos().rotate_iter([
		("input", (0, -1, 0)),
		("output", (0, -1, 0)),
		("memory", (0, -1, 0)),
	]);

	let (scheme, _invalid) = combiner.compile().unwrap();
	scheme
}

/// ***Inputs***: data, write.
///
/// ***Outputs***: first, last, 0, 1, 2, 3 etc...

///
/// Allows to write a word to the first memory cell. All other data will
/// be shifted to next cell on write. Only allows to shift in one
/// direction.
///
/// To write some data send data to 'data' and 1-tick logic signal to
/// 'write'.
///
/// Has outputs from first and last cell ('first', 'last' outputs). Also
/// has output from each cell it has. Those are named as '0', '1', '2'
/// and so on. '0' and 'first' actually lead to the same cell. And so do
/// 'last' and '{n - 1}', where n - amount of cells.
///
/// Amount of cells is determined as `size.0 * size.1 * size.2`.
///
/// Will panic if zero-sized.
pub fn shift_array(word_size: u32, size: (u32, u32, u32)) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	// Add all cells to the combiner
	let cell = incomplete_xor_mem_cell(word_size, 1);
	let cell_size = cell.bounds().tuple();
	let all_cells: Vec<String> = add_cells(&mut combiner, cell, size)
		.into_iter().map(|(cell, _pos)| cell).collect();

	let mut write = Bind::new("write", "logic", (1, 1, 1));

	let mut write_conns_used = 0_u32;
	let mut write_gate_name = "none".to_string();

	for i in 0..all_cells.len() {
		if i + 1 < all_cells.len() {
			combiner.connect(&all_cells[i], format!("{}/data_0", all_cells[i + 1]));
		}
		combiner.pass_output(i.to_string(), &all_cells[i], None as Option<String>).unwrap();

		for j in 0..word_size {
			if write_conns_used % MAX_CONNECTIONS == 0 {
				let id  = write_conns_used / MAX_CONNECTIONS;
				write_gate_name = format!("write_{}", id);

				combiner.add(&write_gate_name, OR).unwrap();
				combiner.pos().place_last((-2, (id / cell_size.2) as i32, (id % cell_size.2) as i32));
				write.connect_full(&write_gate_name);

				combiner.add(format!("write_{}_support", id), BlockBody::new(BlockType::Glass, (1, 1, 1))).unwrap();
				combiner.pos().place_last((-1, (id / cell_size.2) as i32, (id % cell_size.2) as i32));
			}

			combiner.connect(&write_gate_name, format!("{}/write_0/{}", all_cells[i], j));
			write_conns_used += 1;
		}
	}

	combiner.bind_input(write).unwrap();
	combiner.pass_input("data", format!("{}/data_0", all_cells[0]), None as Option<String>).unwrap();
	combiner.pass_output("first", &all_cells[0], None as Option<String>).unwrap();
	combiner.pass_output("last", all_cells.last().unwrap(), None as Option<String>).unwrap();

	let (scheme, _invalid_conns) = combiner.compile().unwrap();
	scheme
}

/// ***Inputs***: data_fwd, write_fwd, data_rev, write_rev.
///
/// ***Outputs***: first, last, 0, 1, 2, 3, etc...

///
/// `shift_array` analog that allows to write in both directions.
/// To write data with shift forward send data to 'data_fwd' and 1-tick
/// signal to 'write_fwd'. To write data from other side, use 'data_rev'
/// and 'write_rev'.
///
/// Will panic if zero-sized.
pub fn bidirectional_shift_array(word_size: u32, size: (u32, u32, u32)) -> Scheme {
	let mut combiner = Combiner::pos_manual();

	// Add all cells to the combiner
	let cell = incomplete_xor_mem_cell(word_size, 2);
	let (all_cells, all_poses): (Vec<String>, Vec<Point>) = add_cells(&mut combiner, cell, size)
		.into_iter().unzip();

	let mut write_fwd = Bind::new("write_fwd", "logic", (1, 1, 1));
	let mut write_rev = Bind::new("write_rev", "logic", (1, 1, 1));

	let mut write_conns_used = 0_u32;
	let mut write_fwd_gate_name = "none".to_string();
	let mut write_rev_gate_name = "none".to_string();

	for i in 0..all_cells.len() {
		if i + 1 < all_cells.len() {
			combiner.connect(&all_cells[i], format!("{}/data_0", all_cells[i + 1]));
		}
		if i > 0 {
			combiner.connect(&all_cells[i], format!("{}/data_1", all_cells[i - 1]));
		}
		combiner.pass_output(i.to_string(), &all_cells[i], None as Option<String>).unwrap();

		for j in 0..word_size {
			if write_conns_used % MAX_CONNECTIONS == 0 {
				let id  = write_conns_used / MAX_CONNECTIONS;
				write_fwd_gate_name = format!("write_fwd_{}", id);
				write_rev_gate_name = format!("write_rev_{}", id);

				combiner.add(&write_fwd_gate_name, OR).unwrap();
				combiner.pos().place_last((-2, (id / 2) as i32, (id % 2) as i32));
				write_fwd.connect_full(&write_fwd_gate_name);

				combiner.add(format!("write_fwd_{}_support", id), BlockBody::new(BlockType::Glass, (1, 1, 1))).unwrap();
				combiner.pos().place_last((-1, (id / 2) as i32, (id % 2) as i32));

				let last_pos = all_poses.last().unwrap();
				combiner.add(&write_rev_gate_name, OR).unwrap();
				combiner.pos().place_last(last_pos.clone() + Point::new_ng(2, (id / 2) as i32, (id % 2) as i32 + 2));
				write_rev.connect_full(&write_rev_gate_name);

				combiner.add(format!("write_rev_{}_support", id), BlockBody::new(BlockType::Glass, (1, 1, 1))).unwrap();
				combiner.pos().place_last(last_pos.clone() + Point::new_ng(1, (id / 2) as i32, (id % 2) as i32 + 2));
			}

			combiner.connect(&write_fwd_gate_name, format!("{}/write_0/{}", all_cells[i], j));
			combiner.connect(&write_rev_gate_name, format!("{}/write_1/{}", all_cells[i], j));
			write_conns_used += 1;
		}
	}

	combiner.bind_input(write_fwd).unwrap();
	combiner.bind_input(write_rev).unwrap();
	combiner.pass_input("data_fwd", format!("{}/data_0", all_cells[0]), None as Option<String>).unwrap();
	combiner.pass_input("data_rev", format!("{}/data_1", all_cells.last().unwrap()), None as Option<String>).unwrap();

	combiner.pass_output("first", &all_cells[0], None as Option<String>).unwrap();
	combiner.pass_output("last", all_cells.last().unwrap(), None as Option<String>).unwrap();

	let (scheme, _invalid_conns) = combiner.compile().unwrap();
	scheme
}

fn add_cells(combiner: &mut Combiner<ManualPos>, cell: Scheme, size: (u32, u32, u32)) -> Vec<(String, Point)> {
	let cell_size: (i32, i32, i32) = cell.bounds().cast().tuple();
	let mut all_cells: Vec<(String, Point)> = vec![];

	for x in 0..size.0 {
		for y in 0..size.1 {
			for z in 0..size.2 {
				let name = format!("{}_{}_{}", x, y, z);
				combiner.add(name.clone(), cell.clone()).unwrap();
				let pos: Point = (
					x as i32 * cell_size.0,
					y as i32 * cell_size.1,
					z as i32 * cell_size.2
				).into();
				combiner.pos().place_last(pos);

				all_cells.push((name, pos));
			}
		}
	}

	all_cells
}