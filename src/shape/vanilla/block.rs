use json::{JsonValue, object};
use crate::scheme::Scheme;
use crate::shape::{Shape, ShapeBase, ShapeBuildData};
use crate::util::Bounds;

/// Describes all the blocks of Scrap Mechanic, that is accessible in creative.
#[derive(Debug, Clone, Copy)]
pub enum BlockType {
	Concrete1,
	Wood1,
	Metal1,
	Barrier,
	Tile,
	Brick,
	Glass,
	GlassTile,
	PathLight,
	Spaceship,
	Cardboard,
	ScrapWood,
	Wood2,
	Wood3,
	ScrapMetal,
	Metal2,
	Metal3,
	ScrapStone,
	Concrete2,
	Concrete3,
	CrackedConcrete,
	ConcreteSlab,
	RustedMetal,
	ExtrudedMetal,
	BubblePlastic,
	Plastic,
	Insulation,
	Plaster,
	Carpet,
	PaintedWall,
	Net,
	SolidNet,
	PunchedSteel,
	StripedNet,
	SquareMesh,
	Restroom,
	DiamondPlate,
	Aluminium,
	WornMetal,
	SpaceshipFloor,
	Sand,
	ArmoredGlass,
}

impl BlockType {
	/// Returns UUID ("shapeId" in JSON) of the block.
	pub fn uuid(&self) -> &str {
		match self {
			BlockType::Concrete1 => 	"a6c6ce30-dd47-4587-b475-085d55c6a3b4",
			BlockType::Wood1 => 		"df953d9c-234f-4ac2-af5e-f0490b223e71",
			BlockType::Metal1 => 		"8aedf6c2-94e1-4506-89d4-a0227c552f1e",
			BlockType::Barrier => 		"09ca2713-28ee-4119-9622-e85490034758",
			BlockType::Tile => 			"8ca49bff-eeef-4b43-abd0-b527a567f1b7",
			BlockType::Brick => 		"0603b36e-0bdb-4828-b90c-ff19abcdfe34",
			BlockType::Glass => 		"5f41af56-df4c-4837-9b3c-10781335757f",
			BlockType::GlassTile => 	"749f69e0-56c9-488c-adf6-66c58531818f",
			BlockType::PathLight => 	"073f92af-f37e-4aff-96b3-d66284d5081c",
			BlockType::Spaceship => 	"027bd4ec-b16d-47d2-8756-e18dc2af3eb6",
			BlockType::Cardboard => 	"f0cba95b-2dc4-4492-8fd9-36546a4cb5aa",
			BlockType::ScrapWood => 	"1fc74a28-addb-451a-878d-c3c605d63811",
			BlockType::Wood2 => 		"1897ee42-0291-43e4-9645-8c5a5d310398",
			BlockType::Wood3 => 		"061b5d4b-0a6a-4212-b0ae-9e9681f1cbfb",
			BlockType::ScrapMetal => 	"1f7ac0bb-ad45-4246-9817-59bdf7f7ab39",
			BlockType::Metal2 => 		"1016cafc-9f6b-40c9-8713-9019d399783f",
			BlockType::Metal3 => 		"c0dfdea5-a39d-433a-b94a-299345a5df46",
			BlockType::ScrapStone => 	"30a2288b-e88e-4a92-a916-1edbfc2b2dac",
			BlockType::Concrete2 => 	"ff234e42-5da4-43cc-8893-940547c97882",
			BlockType::Concrete3 => 	"e281599c-2343-4c86-886e-b2c1444e8810",
			BlockType::CrackedConcrete => "f5ceb7e3-5576-41d2-82d2-29860cf6e20e",
			BlockType::ConcreteSlab => 	"cd0eff89-b693-40ee-bd4c-3500b23df44e",
			BlockType::RustedMetal => 	"220b201e-aa40-4995-96c8-e6007af160de",
			BlockType::ExtrudedMetal => "25a5ffe7-11b1-4d3e-8d7a-48129cbaf05e",
			BlockType::BubblePlastic => "f406bf6e-9fd5-4aa0-97c1-0b3c2118198e",
			BlockType::Plastic => 		"628b2d61-5ceb-43e9-8334-a4135566df7a",
			BlockType::Insulation => 	"9be6047c-3d44-44db-b4b9-9bcf8a9aab20",
			BlockType::Plaster => 		"b145d9ae-4966-4af6-9497-8fca33f9aee3",
			BlockType::Carpet => 		"febce8a6-6c05-4e5d-803b-dfa930286944",
			BlockType::PaintedWall => 	"e981c337-1c8a-449c-8602-1dd990cbba3a",
			BlockType::Net => 			"4aa2a6f0-65a4-42e3-bf96-7dec62570e0b",
			BlockType::SolidNet => 		"3d0b7a6e-5b40-474c-bbaf-efaa54890e6a",
			BlockType::PunchedSteel => 	"ea6864db-bb4f-4a89-b9ec-977849b6713a",
			BlockType::StripedNet => 	"a479066d-4b03-46b5-8437-e99fec3f43ee",
			BlockType::SquareMesh => 	"b4fa180c-2111-4339-b6fd-aed900b57093",
			BlockType::Restroom => 		"920b40c8-6dfc-42e7-84e1-d7e7e73128f6",
			BlockType::DiamondPlate => 	"f7d4bfed-1093-49b9-be32-394c872a1ef4",
			BlockType::Aluminium => 	"3e3242e4-1791-4f70-8d1d-0ae9ba3ee94c",
			BlockType::WornMetal => 	"d740a27d-cc0f-4866-9e07-6a5c516ad719",
			BlockType::SpaceshipFloor => "4ad97d49-c8a5-47f3-ace3-d56ba3affe50",
			BlockType::Sand => 			"c56700d9-bbe5-4b17-95ed-cef05bd8be1b",
			BlockType::ArmoredGlass => 	"b5ee5539-75a2-4fef-873b-ef7c9398b3f5",
		}
	}

	/// Returns the default color of the block.
	pub fn default_color(&self) -> &str {
		match self {
			BlockType::Concrete1 => 			"8d8f89",
			BlockType::Wood1 => 				"9b683a",
			BlockType::Metal1 => 				"675f51",
			BlockType::Barrier => 				"ce9e0c",
			BlockType::Tile => 					"bfdfed",
			BlockType::Brick => 				"af967b",
			BlockType::Glass => 				"e4f8ff",
			BlockType::GlassTile => 			"c2f9ff",
			BlockType::PathLight => 			"727272",
			BlockType::Spaceship => 			"820a0a",
			BlockType::Cardboard => 			"a48052",
			BlockType::ScrapWood => 			"cd9d71",
			BlockType::Wood2 => 				"dc9153",
			BlockType::Wood3 => 				"f2ad74",
			BlockType::ScrapMetal => 			"df6226",
			BlockType::Metal2 => 				"869499",
			BlockType::Metal3 => 				"88a5ac",
			BlockType::ScrapStone => 			"848484",
			BlockType::Concrete2 => 			"8d8f89",
			BlockType::Concrete3 => 			"c9d7dc",
			BlockType::CrackedConcrete => 		"8d8f89",
			BlockType::ConcreteSlab => 			"af967b",
			BlockType::RustedMetal => 			"738192",
			BlockType::ExtrudedMetal => 		"858795",
			BlockType::BubblePlastic => 		"9acfd2",
			BlockType::Plastic => 				"0b9ade",
			BlockType::Insulation => 			"fff063",
			BlockType::Plaster => 				"979797",
			BlockType::Carpet => 				"368085",
			BlockType::PaintedWall => 			"eeeeee",
			BlockType::Net => 					"435359",
			BlockType::SolidNet => 				"888888",
			BlockType::PunchedSteel => 			"888888",
			BlockType::StripedNet => 			"888888",
			BlockType::SquareMesh => 			"c36512",
			BlockType::Restroom => 				"607b79",
			BlockType::DiamondPlate => 			"43494d",
			BlockType::Aluminium => 			"727272",
			BlockType::WornMetal => 			"66837c",
			BlockType::SpaceshipFloor => 		"dadada",
			BlockType::Sand => 					"c69146",
			BlockType::ArmoredGlass => 			"3abfb1",
		}
	}
}

/// Body of given block type with some physical size.
///
/// # Examples
/// ```
/// use crate::sm_logic::shape::vanilla::BlockType;
/// use crate::sm_logic::shape::vanilla::BlockBody;
///
/// let iron_pillar = BlockBody::new(BlockType::Metal1, (10, 10, 400));
/// let concrete_plate = BlockBody::new(BlockType::Concrete1, (400, 400, 3));
/// let wood_block = BlockBody::new(BlockType::Wood1, (64, 64, 64));
/// ```
#[derive(Debug, Clone)]
pub struct BlockBody {
	block_type: BlockType,
	size: Bounds,
}

impl BlockBody {
	pub fn new<B: Into<Bounds>>(block_type: BlockType, size: B) -> Shape {
		Shape::new(
			Box::new(
				BlockBody {
					block_type,
					size: size.into()
				}
			)
		)
	}
}

impl ShapeBase for BlockBody {
	fn build(&self, data: ShapeBuildData) -> JsonValue {
		let (xaxis, zaxis, offset) = data.rot.to_sm_data();
		let (x, y, z) = (data.pos + offset).tuple();
		let bounds = self.size.tuple();

		object!{
			"color": match data.color {
				None => self.block_type.default_color(),
				Some(color) => color,
			},
			"shapeId": self.block_type.uuid(),
			"xaxis": xaxis,
			"zaxis": zaxis,
			"pos": {
				"x": x,
				"y": y,
				"z": z,
			},
			"bounds": {
				"x": bounds.0,
				"y": bounds.1,
				"z": bounds.2,
			},
		}
	}

	fn size(&self) -> Bounds {
		self.size.clone()
	}

	fn has_input(&self) -> bool {
		false
	}

	fn has_output(&self) -> bool {
		false
	}
}

impl Into<Shape> for BlockBody {
	fn into(self) -> Shape {
		Shape::new(Box::new(self))
	}
}

impl Into<Scheme> for BlockBody {
	fn into(self) -> Scheme {
		let shape: Shape = self.into();
		shape.into()
	}
}