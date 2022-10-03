use sm_logic::bind::Bind;
use sm_logic::combiner::*;
use sm_logic::scheme::Scheme;
use sm_logic::positioner::ManualPos;
use sm_logic::util::GateMode::{AND, OR, XOR};
use sm_logic::util::Facing;

fn main() {
	match adder(8).compile() {
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

			/*println!("\nShapes:");
			for (pos, _, shape) in scheme.shapes() {
				println!("\tpos {:?} - {:?}", pos.tuple_ref(), shape);
			}*/

			println!("Writing to json...");
			let json = scheme.to_json().to_string();
			std::fs::write(r#"C:\Users\redch\AppData\Roaming\Axolot Games\Scrap Mechanic\User\User_76561198288016737\Blueprints\e153cd62-9736-409c-be41-0921439f3848/blueprint.json"#, json).unwrap();
			println!("Done");
		}
	}
}

fn adder(word_size: u32) -> Combiner<ManualPos> {
	let section = section();

	// Создание сумматора из таких секций
	let mut adder = Combiner::pos_manual();

	// Добавление энного количества секций и подключение выходов бита переноса
	// ко входам битов переноса следующих секций
	for i in 0..word_size {
		adder.add(format!("{}", i), section.clone()).unwrap();
		adder.pos().place_last((0, i as i32, 0));

		// Последнее соединение (в несуществующий 17-тый модуль) просто отсеется
		adder.connect(format!("{}", i), format!("{}", i+1));
	}

	// Бинд входа "а", размером (16, 1, 1) - в данном случае 16 бит сумматор
	let mut bind = Bind::new("a", "binary", (word_size, 1u32, 1u32));
	for x in 0..(word_size as i32) {
		bind.connect(((x, 0, 0), (1, 1, 1)), format!("{}/a", x));
	}
	adder.bind_input(bind).unwrap();

	let mut bind = Bind::new("b", "binary", (word_size, 1u32, 1u32));
	bind.connect_func(|x, _, _| Some(format!("{}/b", x)));
	// Бинд второго входа
	adder.bind_input(bind).unwrap();

	// Бинд выхода (сумма двух)
	let mut bind = Bind::new("_", "binary", (word_size, 1u32, 1u32));
	bind.connect_func(|x, _, _| Some(format!("{}/res", x)));
	adder.bind_output(bind).unwrap();

	adder
}

fn section() -> Scheme {
	// Создание секции сумматора
	let mut s = Combiner::pos_manual();

	// Создание именованных гейтов
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

	// Биндинг - создание корреляции абстрактных входов "a", "b", "_"
	// и непосредственно шейпов
	// Бит 1
	let mut bind = Bind::new("a", "bit", (1, 1, 1));
	bind.connect_full("a");
	s.bind_input(bind).unwrap();

	// Бит 2
	let mut bind = Bind::new("b", "bit", (1, 1, 1));
	bind.connect_full("b");
	s.bind_input(bind).unwrap();

	// Бит переноса (вход)
	let mut bind = Bind::new("_", "bit", (1, 1, 1));
	bind.connect_full("_");
	s.bind_input(bind).unwrap();

	// Подключение гейтов "a", "b", "_" в гейт "res"
	s.connect_iter(["a", "b", "_"], ["res"]);

	s.connect_iter(["a"], ["and_1", "and_2"]);
	s.connect_iter(["b"], ["and_2", "and_3"]);
	s.connect_iter(["_"], ["and_3", "and_1"]);

	// Биндинг выходов секции
	// Бит переноса
	let mut bind = Bind::new("_", "bit", (1, 1, 1));
	bind.connect_full("and_1")
		.connect_full("and_2")
		.connect_full("and_3");
	s.bind_output(bind).unwrap();

	// Бит результата
	let mut bind = Bind::new("res", "bit", (1, 1, 1));
	bind.connect_full("res");
	s.bind_output(bind).unwrap();

	// Секция готова.
	let (scheme, _) = s.compile().unwrap();
	scheme
}