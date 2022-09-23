//! A whole bunch of boring-ass traits implementations

use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;
use std::ops::AddAssign;
use std::ops::DivAssign;
use std::ops::MulAssign;
use std::ops::SubAssign;

use super::Mat4;
use super::Vec4;
use super::Vec3;

impl From<[f32; 16]> for Mat4 {
    fn from(slice: [f32; 16]) -> Self {
        Mat4(slice)
    }
}

impl PartialEq for Mat4 {
    fn eq(&self, other: &Self) -> bool {
        self.0.iter()
            .zip(other.0.iter())
            .all(|(a, b)| *a == *b)
    }
}
impl PartialEq for Vec4 {
    fn eq(&self, other: &Self) -> bool {
        self.x() == other.x() &&
        self.y() == other.y() &&
        self.z() == other.z() &&
        self.w() == other.w()
    }
}
impl PartialEq for Vec3 {
    fn eq(&self, other: &Self) -> bool {
        self.x() == other.x() &&
            self.y() == other.y() &&
            self.z() == other.z()
    }
}

// Mathematics

// ======= Mat4 ========
impl Add<Mat4> for Mat4 {
    type Output = Mat4;

    fn add(self, rhs: Mat4) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..16 { new_mat.0[i] = self.0[i] + rhs.0[i]; }
        new_mat
    }
}
impl Add<f32> for Mat4 {
    type Output = Mat4;

    fn add(self, rhs: f32) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..16 { new_mat.0[i] = self.0[i] + rhs; }
        new_mat
    }
}
impl Add<Mat4> for f32 {
    type Output = Mat4;

    fn add(self, rhs: Mat4) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..16 { new_mat.0[i] = rhs.0[i] + self; }
        new_mat
    }
}

impl AddAssign for Mat4 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}
impl AddAssign<f32> for Mat4 {
    fn add_assign(&mut self, other: f32) {
        *self = *self + other;
    }
}

impl Sub<Mat4> for Mat4 {
    type Output = Mat4;

    fn sub(self, rhs: Mat4) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..16 { new_mat.0[i] = self.0[i] - rhs.0[i]; }
        new_mat
    }
}
impl Sub<f32> for Mat4 {
    type Output = Mat4;

    fn sub(self, rhs: f32) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..16 { new_mat.0[i] = self.0[i] - rhs; }
        new_mat
    }
}
impl Sub<Mat4> for f32 {
    type Output = Mat4;

    fn sub(self, rhs: Mat4) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..16 { new_mat.0[i] = self - rhs.0[i]; }
        new_mat
    }
}

impl SubAssign for Mat4 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}
impl SubAssign<f32> for Mat4 {
    fn sub_assign(&mut self, other: f32) {
        *self = *self - other;
    }
}

impl Mul<Mat4> for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..4 {
            for j in 0..4 {
                let id = i * 4 + j;
                for k in 0..4 {
                    new_mat.0[id] += self.0[i * 4 + k] * rhs.0[k * 4 + j];
                }
            }
        }
        new_mat
    }
}
impl Mul<f32> for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: f32) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..16 { new_mat.0[i] = self.0[i] * rhs; }
        new_mat
    }
}
impl Mul<Mat4> for f32 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..16 { new_mat.0[i] = rhs.0[i] * self; }
        new_mat
    }
}

impl MulAssign<f32> for Mat4 {
    fn mul_assign(&mut self, other: f32) {
        *self = *self * other;
    }
}
impl MulAssign<Mat4> for Mat4 {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl Div<Mat4> for Mat4 {
    type Output = Mat4;

    fn div(self, rhs: Mat4) -> Mat4 {
        self * rhs.inverse()
    }
}
impl Div<f32> for Mat4 {
    type Output = Mat4;

    fn div(self, rhs: f32) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..16 { new_mat.0[i] = self.0[i] / rhs; }
        new_mat
    }
}
impl Div<Mat4> for f32 {
    type Output = Mat4;

    fn div(self, rhs: Mat4) -> Mat4 {
        self * rhs.inverse()
    }
}

impl DivAssign<f32> for Mat4 {
    fn div_assign(&mut self, other: f32) {
        *self = *self / other;
    }
}
impl DivAssign<Mat4> for Mat4 {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

impl std::fmt::Debug for Mat4 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut max_width = [0usize; 4];
        for row in 0..4 {
            for col in 0..4 {
                let id = 4 * row + col;
                let str_len = self.0[id].to_string().len();
                if str_len > max_width[col] { max_width[col] = str_len; }
            }
        };

        write!(f, "Mat4:\n")?;
        for row in 0..4 {
            write!(f, "| ")?;
            for col in 0..4 {
                let id = 4 * row + col;
                let str_len = self.0[id].to_string().len();
                write!(f, "{}", &self.0[id])?;
                write!(f, "{}", " ".chars().cycle().take(max_width[col] - str_len + 1).collect::<String>())?;
            }
            write!(f, "|\n")?;
        };
        Ok(())
    }
}


// ======= Vec4 ========
impl Add<Vec4> for Vec4 {
    type Output = Vec4;
    fn add(self, rhs: Vec4) -> Vec4 {
        Vec4::new(self.x() + rhs.x(), self.y() + rhs.y(), self.z() + rhs.z(), self.w() + rhs.w())
    }
}
impl Add<f32> for Vec4 {
    type Output = Vec4;
    fn add(self, rhs: f32) -> Vec4 {
        Vec4::new(self.x() + rhs, self.y() + rhs, self.z() + rhs, self.w() + rhs)
    }
}
impl Add<Vec4> for f32 {
    type Output = Vec4;
    fn add(self, rhs: Vec4) -> Vec4 {
        Vec4::new(self + rhs.x(), self + rhs.y(), self + rhs.z(), self + rhs.w())
    }
}

impl AddAssign for Vec4 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}
impl AddAssign<f32> for Vec4 {
    fn add_assign(&mut self, other: f32) {
        *self = *self + other;
    }
}

impl Sub<Vec4> for Vec4 {
    type Output = Vec4;
    fn sub(self, rhs: Vec4) -> Vec4 {
        Vec4::new(self.x() - rhs.x(), self.y() - rhs.y(), self.z() - rhs.z(), self.w() - rhs.w())
    }
}
impl Sub<f32> for Vec4 {
    type Output = Vec4;
    fn sub(self, rhs: f32) -> Vec4 {
        Vec4::new(self.x() - rhs, self.y() - rhs, self.z() - rhs, self.w() - rhs)
    }
}
impl Sub<Vec4> for f32 {
    type Output = Vec4;
    fn sub(self, rhs: Vec4) -> Vec4 {
        Vec4::new(self - rhs.x(), self - rhs.y(), self - rhs.z(), self - rhs.w())
    }
}

impl SubAssign for Vec4 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}
impl SubAssign<f32> for Vec4 {
    fn sub_assign(&mut self, other: f32) {
        *self = *self - other;
    }
}

impl Mul<Vec4> for Vec4 {
    type Output = Vec4;
    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4::new(
            self.x * rhs.x,
            self.y * rhs.y,
            self.z * rhs.z,
            self.w * rhs.w
        )
    }
}
impl Mul<f32> for Vec4 {
    type Output = Vec4;
    fn mul(self, rhs: f32) -> Vec4 {
        Vec4::new(self.x() * rhs, self.y() * rhs, self.z() * rhs, self.w() * rhs)
    }
}
impl Mul<Vec4> for f32 {
    type Output = Vec4;
    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4::new(self * rhs.x(), self * rhs.y(), self * rhs.z(), self * rhs.w())
    }
}
impl Mul<Vec4> for Mat4 {
    type Output = Vec4;
    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4::new(
            self.get(0, 0) * rhs.x() + self.get(0, 1) * rhs.y() + self.get(0, 2) * rhs.z() + self.get(0, 3) * rhs.w(),
            self.get(1, 0) * rhs.x() + self.get(1, 1) * rhs.y() + self.get(1, 2) * rhs.z() + self.get(1, 3) * rhs.w(),
            self.get(2, 0) * rhs.x() + self.get(2, 1) * rhs.y() + self.get(2, 2) * rhs.z() + self.get(2, 3) * rhs.w(),
            self.get(3, 0) * rhs.x() + self.get(3, 1) * rhs.y() + self.get(3, 2) * rhs.z() + self.get(3, 3) * rhs.w())
    }
}

impl MulAssign<f32> for Vec4 {
    fn mul_assign(&mut self, other: f32) {
        *self = *self * other;
    }
}
impl MulAssign<Vec4> for Vec4 {
    fn mul_assign(&mut self, other: Vec4) {
        *self = *self * other;
    }
}

impl Div<Vec4> for Vec4 {
    type Output = Self;
    fn div(self, rhs: Vec4) -> Self {
        Self::new(
            self.x() / rhs.x(),
            self.y() / rhs.y(),
            self.z() / rhs.z(),
            self.w() / rhs.w(),
        )
    }
}
impl Div<f32> for Vec4 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self::new(
            self.x() / rhs,
            self.y() / rhs,
            self.z() / rhs,
            self.w() / rhs,
        )
    }
}
impl Div<Vec4> for f32 {
    type Output = Vec4;
    fn div(self, rhs: Vec4) -> Vec4 {
        Vec4::new(
            self / rhs.x(),
            self / rhs.y(),
            self / rhs.z(),
            self / rhs.w(),
        )
    }
}

impl DivAssign<Vec4> for Vec4 {
    fn div_assign(&mut self, other: Vec4) {
        *self = *self / other;
    }
}
impl DivAssign<f32> for Vec4 {
    fn div_assign(&mut self, other: f32) {
        *self = *self / other;
    }
}


// ======= Vec3 ========
impl Add<Vec3> for Vec3 {
    type Output = Self;
    fn add(self, rhs: Vec3) -> Self {
        Self::new(self.x() + rhs.x(), self.y() + rhs.y(), self.z() + rhs.z())
    }
}
impl Add<f32> for Vec3 {
    type Output = Self;
    fn add(self, rhs: f32) -> Self {
        Self::new(self.x() + rhs, self.y() + rhs, self.z() + rhs)
    }
}
impl Add<Vec3> for f32 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self + rhs.x(), self + rhs.y(), self + rhs.z())
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}
impl AddAssign<f32> for Vec3 {
    fn add_assign(&mut self, other: f32) {
        *self = *self + other;
    }
}

impl Sub<Vec3> for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x() - rhs.x(), self.y() - rhs.y(), self.z() - rhs.z())
    }
}
impl Sub<f32> for Vec3 {
    type Output = Self;
    fn sub(self, rhs: f32) -> Self {
        Self::new(self.x() - rhs, self.y() - rhs, self.z() - rhs)
    }
}
impl Sub<Vec3> for f32 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self - rhs.x(), self - rhs.y(), self - rhs.z())
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}
impl SubAssign<f32> for Vec3 {
    fn sub_assign(&mut self, other: f32) {
        *self = *self - other;
    }
}

impl Mul<Vec3> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3::new(
            self.x * rhs.x,
            self.y * rhs.y,
            self.z * rhs.z
        )
    }
}
impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x() * rhs, self.y() * rhs, self.z() * rhs)
    }
}
impl Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self * rhs.x(), self * rhs.y(), self * rhs.z())
    }
}

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, other: f32) {
        *self = *self * other;
    }
}

impl Div<Vec3> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: Vec3) -> Vec3 {
        Self::new(
            self.x() / rhs.x(),
            self.y() / rhs.y(),
            self.z() / rhs.z(),
        )
    }
}
impl Div<f32> for Vec3 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self::new(
            self.x() / rhs,
            self.y() / rhs,
            self.z() / rhs,
        )
    }
}
impl Div<Vec3> for f32 {
    type Output = Vec3;
    fn div(self, rhs: Vec3) -> Vec3 {
        Vec3::new(
            self / rhs.x(),
            self / rhs.y(),
            self / rhs.z(),
        )
    }
}

impl DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, other: f32) {
        *self = *self / other;
    }
}
impl DivAssign<Vec3> for Vec3 {
    fn div_assign(&mut self, other: Vec3) {
        *self = *self / other;
    }
}

impl From<Vec4> for Vec3 {
    fn from(item: Vec4) -> Self {
        Self::new(item.x, item.y, item.z)
    }
}
impl From<Vec3> for Vec4 {
    fn from(item: Vec3) -> Self {
        Self::new(item.x, item.y, item.z, 0.0)
    }
}