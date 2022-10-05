use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, Index, IndexMut, Neg};
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::DivAssign;
use std::ops::Mul;
use std::ops::MulAssign;
use std::ops::Rem;
use std::ops::RemAssign;
use std::ops::Sub;
use std::ops::SubAssign;

/// Simple Vector of three elements.
///
/// +, -, *, / operations are just corresponding operations on eachs
/// pair of coordinates separately
///
/// # Example
/// ```
/// # use crate::sm_logic::util::Vec3;
/// let vec_a: Vec3<i32> = Vec3::new(1, 2, 3);
/// let vec_b: Vec3<i32> = Vec3::new(4, 5, 6);
///
/// assert_eq!(*vec_a.x(), 1);
/// assert_eq!(*vec_a.y(), 2);
/// assert_eq!(*vec_a.z(), 3);
///
/// assert_eq!(vec_b[0], 4);
/// assert_eq!(vec_b[1], 5);
/// assert_eq!(vec_b[2], 6);
///
/// assert_eq!(vec_a + vec_b, Vec3::new(5_i32, 7_i32, 9_i32));
/// assert_eq!(vec_a - vec_b, Vec3::new(-3_i32, -3_i32, -3_i32));
/// assert_eq!(vec_a * vec_b, Vec3::new(4_i32, 10_i32, 18_i32));
/// assert_eq!(vec_a / vec_b, Vec3::new(0_i32, 0_i32, 0_i32));
/// ```
pub struct Vec3<N> {
	x: N,
	y: N,
	z: N,
}

impl<N> Index<usize> for Vec3<N> {
	type Output = N;

	fn index(&self, index: usize) -> &Self::Output {
		match index {
			0 => self.x(),
			1 => self.y(),
			2 => self.z(),
			i => panic!("Invalid index in Vec3: {}", i),
		}
	}
}

impl<N> IndexMut<usize> for Vec3<N> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		match index {
			0 => self.x_mut(),
			1 => self.y_mut(),
			2 => self.z_mut(),
			i => panic!("Invalid index in Vec3: {}", i),
		}
	}
}

impl<T, N: Neg<Output = T>> Neg for Vec3<N> {
	type Output = Vec3<T>;

	fn neg(self) -> Self::Output {
		Vec3 {
			x: -self.x,
			y: -self.y,
			z: -self.z
		}
	}
}

impl<N: Mul<N, Output = N> + Add<N, Output = N>> Vec3<N> {
	pub fn dot(self, other: Self) -> N {
		(self.x * other.x) + (self.y * other.y) + (self.z * other.z)
	}
}

impl<N> Vec3<N> {
	pub fn new<A, B, C>(x: A, y: B, z: C) -> Vec3<N>
		where A: Into<N>, B: Into<N>, C: Into<N>
	{
		let x = x.into();
		let y = y.into();
		let z = z.into();
		Vec3 { x, y, z }
	}

	pub fn new_ng(x: N, y: N, z: N) -> Self {
		Self::new(x, y, z)
	}

	pub fn from_tuple<A, B, C>(t: (A, B, C)) -> Vec3<N>
		where A: Into<N>, B: Into<N>, C: Into<N>
	{
		Self::new(t.0, t.1, t.2)
	}

	pub fn x(&self) -> &N {
		&self.x
	}

	pub fn y(&self) -> &N {
		&self.y
	}

	pub fn z(&self) -> &N {
		&self.z
	}

	pub fn x_mut(&mut self) -> &mut N {
		&mut self.x
	}

	pub fn y_mut(&mut self) -> &mut N {
		&mut self.y
	}

	pub fn z_mut(&mut self) -> &mut N {
		&mut self.z
	}

	pub fn tuple(self) -> (N, N, N) {
		(self.x, self.y, self.z)
	}

	pub fn tuple_ref(&self) -> (&N, &N, &N) {
		(self.x(), self.y(), self.z())
	}

	pub fn try_cast<T: TryFrom<N>>(self) -> Result<Vec3<T>, <T as TryFrom<N>>::Error>
	{
		Ok(
			Vec3 {
				x: self.x.try_into()?,
				y: self.y.try_into()?,
				z: self.z.try_into()?,
			}
		)
	}

	pub fn cast<T: TryFrom<N> + Debug>(self) -> Vec3<T> {
		match self.try_cast::<T>() {
			Ok(result) => result,
			Err(_) => panic!("Failed to cast Vec3 between types ;("),
		}
	}
}

impl<N: Copy> Copy for Vec3<N> {}

impl<N: Clone> Clone for Vec3<N> {
	fn clone(&self) -> Self {
		Vec3 {
			x: self.x.clone(),
			y: self.y.clone(),
			z: self.z.clone(),
		}
	}
}

impl<N> Into<Vec3<N>> for (N, N, N) {
	fn into(self) -> Vec3<N> {
		Vec3::from_tuple(self)
	}
}

impl<N> Into<(N, N, N)> for Vec3<N> {
	fn into(self) -> (N, N, N) {
		self.tuple()
	}
}


impl<N: Debug> Debug for Vec3<N> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str(&format!("Vec3{{{:?}, {:?}, {:?}}}", self.x(), self.y(), self.z()))
	}
}

impl<N: Display> Display for Vec3<N> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str(&format!("Vec3{{{}, {}, {}}}", self.x(), self.y(), self.z()))
	}
}

impl<N: PartialEq<N>> PartialEq<Vec3<N>> for Vec3<N> {
	fn eq(&self, other: &Self) -> bool {
		self.x().eq(other.x()) &&
			self.y().eq(other.y()) &&
			self.z().eq(other.z())
	}
}


impl<N: Add<N, Output = N>> Add<Vec3<N>> for Vec3<N> {
	type Output = Vec3<N>;

	fn add(self, rhs: Self) -> Self::Output {
		Vec3 {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
			z: self.z + rhs.z,
		}
	}
}

impl<N: AddAssign<N>> AddAssign<Vec3<N>> for Vec3<N> {
	fn add_assign(&mut self, rhs: Vec3<N>) {
		self.x += rhs.x;
		self.y += rhs.y;
		self.z += rhs.z;
	}
}

impl<N: Copy + Add<N, Output = N>> Add<N> for Vec3<N> {
	type Output = Vec3<N>;

	fn add(self, rhs: N) -> Self::Output {
		Vec3 {
			x: self.x + rhs,
			y: self.y + rhs,
			z: self.z + rhs,
		}
	}
}

impl<N: Copy + AddAssign<N>> AddAssign<N> for Vec3<N> {
	fn add_assign(&mut self, rhs: N) {
		self.x += rhs;
		self.y += rhs;
		self.z += rhs;
	}
}


impl<N: Sub<N, Output = N>> Sub<Vec3<N>> for Vec3<N> {
	type Output = Vec3<N>;

	fn sub(self, rhs: Self) -> Self::Output {
		Vec3 {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
			z: self.z - rhs.z,
		}
	}
}

impl<N: SubAssign<N>> SubAssign<Vec3<N>> for Vec3<N> {
	fn sub_assign(&mut self, rhs: Vec3<N>) {
		self.x -= rhs.x;
		self.y -= rhs.y;
		self.z -= rhs.z;
	}
}

impl<N: Copy + Sub<N, Output = N>> Sub<N> for Vec3<N> {
	type Output = Vec3<N>;

	fn sub(self, rhs: N) -> Self::Output {
		Vec3 {
			x: self.x - rhs,
			y: self.y - rhs,
			z: self.z - rhs,
		}
	}
}

impl<N: Copy + SubAssign<N>> SubAssign<N> for Vec3<N> {
	fn sub_assign(&mut self, rhs: N) {
		self.x -= rhs;
		self.y -= rhs;
		self.z -= rhs;
	}
}


impl<N: Mul<N, Output = N>> Mul<Vec3<N>> for Vec3<N> {
	type Output = Vec3<N>;

	fn mul(self, rhs: Self) -> Self::Output {
		Vec3 {
			x: self.x * rhs.x,
			y: self.y * rhs.y,
			z: self.z * rhs.z,
		}
	}
}

impl<N: MulAssign<N>> MulAssign<Vec3<N>> for Vec3<N> {
	fn mul_assign(&mut self, rhs: Vec3<N>) {
		self.x *= rhs.x;
		self.y *= rhs.y;
		self.z *= rhs.z;
	}
}

impl<N: Copy + Mul<N, Output = N>> Mul<N> for Vec3<N> {
	type Output = Vec3<N>;

	fn mul(self, rhs: N) -> Self::Output {
		Vec3 {
			x: self.x * rhs,
			y: self.y * rhs,
			z: self.z * rhs,
		}
	}
}

impl<N: Copy + MulAssign<N>> MulAssign<N> for Vec3<N> {
	fn mul_assign(&mut self, rhs: N) {
		self.x *= rhs;
		self.y *= rhs;
		self.z *= rhs;
	}
}


impl<N: Div<N, Output = N>> Div<Vec3<N>> for Vec3<N> {
	type Output = Vec3<N>;

	fn div(self, rhs: Self) -> Self::Output {
		Vec3 {
			x: self.x / rhs.x,
			y: self.y / rhs.y,
			z: self.z / rhs.z,
		}
	}
}

impl<N: DivAssign<N>> DivAssign<Vec3<N>> for Vec3<N> {
	fn div_assign(&mut self, rhs: Vec3<N>) {
		self.x /= rhs.x;
		self.y /= rhs.y;
		self.z /= rhs.z;
	}
}

impl<N: Copy + Div<N, Output = N>> Div<N> for Vec3<N> {
	type Output = Vec3<N>;

	fn div(self, rhs: N) -> Self::Output {
		Vec3 {
			x: self.x / rhs,
			y: self.y / rhs,
			z: self.z / rhs,
		}
	}
}

impl<N: Copy + DivAssign<N>> DivAssign<N> for Vec3<N> {
	fn div_assign(&mut self, rhs: N) {
		self.x /= rhs;
		self.y /= rhs;
		self.z /= rhs;
	}
}


impl<N: Rem<N, Output = N>> Rem<Vec3<N>> for Vec3<N> {
	type Output = Vec3<N>;

	fn rem(self, rhs: Self) -> Self::Output {
		Vec3 {
			x: self.x % rhs.x,
			y: self.y % rhs.y,
			z: self.z % rhs.z,
		}
	}
}

impl<N: RemAssign<N>> RemAssign<Vec3<N>> for Vec3<N> {
	fn rem_assign(&mut self, rhs: Vec3<N>) {
		self.x %= rhs.x;
		self.y %= rhs.y;
		self.z %= rhs.z;
	}
}

impl<N: Copy + Rem<N, Output = N>> Rem<N> for Vec3<N> {
	type Output = Vec3<N>;

	fn rem(self, rhs: N) -> Self::Output {
		Vec3 {
			x: self.x % rhs,
			y: self.y % rhs,
			z: self.z % rhs,
		}
	}
}

impl<N: Copy + RemAssign<N>> RemAssign<N> for Vec3<N> {
	fn rem_assign(&mut self, rhs: N) {
		self.x %= rhs;
		self.y %= rhs;
		self.z %= rhs;
	}
}