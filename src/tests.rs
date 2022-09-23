use crate::util::Rot;

#[test]
fn test_rot() {
	for rx in -5..5 {
		for ry in -5..5 {
			for rz in -5..5 {
				let rot = Rot::new(rx, ry, rz);
				let basis = rot.to_basis();
				assert_eq!(rot, Rot::from_basis(basis));
			}
		}
	}
}