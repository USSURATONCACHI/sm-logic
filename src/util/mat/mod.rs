// This module is used from my other project

//! # Overview
//! This module provides rust-native implementation for three mathematical
//! objects. These objects are:
//! - [`Mat4`] - Matrix 4x4
//! - [`Vec4`] - Vector of four numbers
//! - [`Vec3`] - Vector of three numbers
//! Note: every number has the type [`f32`]
//!

mod traits;


/// Mat4 is the a matrix 4 by 4 elements.<br>
/// Data is stored in array in row-major order: first set of four
/// elements is first row, second set of four - second row and so on.
#[derive(Copy, Clone)]
pub struct Mat4 (pub [f32; 16]);

impl Mat4 {
    /// Constructs an unit matrix multiplied by `det`. Determinant
    /// of returned matrix is equal to `det`.
    pub fn new(det: f32) -> Mat4 {
        let mut res = [0.0_f32; 16];
        for i in 0..4 {
            res[i * 5] = det;
        }
        Mat4(res)
    }

    /// Constructs a matrix filled with given number.<br>
    pub fn filled(with: f32) -> Mat4 {
        Mat4([with; 16])
    }

    /// Transposes a matrix
    pub fn transpos(&self) -> Mat4 {


        let mut res: [f32; 16] = [0.0; 16];
        for i in 0..4 {
            for j in 0..4 {
                res[i * 4 + j] = self.0[j * 4 + i];
            }
        }
        Mat4(res)
    }

    /// Returns i-th, j-th element of matrix<br>
    /// IMPORTANT: count starts from 0
    pub fn get(&self, i: usize, j: usize) -> f32 {
        self.0[i * 4 + j]
    }

    /// Sets i-th, j-th element of matrix with given value<br>
    /// IMPORTANT: count starts from 0
    pub fn set(&mut self, i: usize, j: usize, val: f32) {
        self.0[i * 4 + j] = val;
    }

    /// Return matrix without given row and column
    /// This matrix has size 3 by 3, so it is represented as \[f32; 9]
    pub fn sub_matrix(&self, row: usize, col: usize) -> [f32; 9] {
        let mut res = [0_f32; 9];

        let mut tmp_i: usize = 0;
        let mut tmp_j: usize;

        for i in 0..4 {
            tmp_j = 0;
            if i != row {
                for j in 0..4 {
                    if j != col {
                        res[tmp_i * 3 + tmp_j] = self.0[i * 4 + j];
                        tmp_j += 1;
                    }
                }
                tmp_i += 1;
            }
        }

        res
    }

    /// Return determinant of a matrix
    pub fn det(&self) -> f32 {
        self.0[0] * sub_det(self.sub_matrix(0, 0)) -
        self.0[1] * sub_det(self.sub_matrix(0, 1)) +
        self.0[2] * sub_det(self.sub_matrix(0, 2)) -
        self.0[3] * sub_det(self.sub_matrix(0, 3))
    }

    //Алгебраические дополнения
    pub fn alg_add(&self) -> Mat4 {
        let mut res = Mat4::new(0.0);
        for i in 0..4 {
            for j in 0..4 {
                res.0[i * 4 + j] = (-1.0_f32).powf((i + j) as f32) * sub_det(self.sub_matrix(i, j));
            }
        }
        res
    }

    //Обратная матрица
    pub fn inverse(&self) -> Mat4 {
        self.alg_add().transpos() / self.det()
    }
}
/** Набор конструкторов для полезных матриц, вроде матриц поворота, сдвига и т.д.*/
impl Mat4 {
    //Матрица сдвига, поворота, масштаба по трем осям
    pub fn object_mat(dx: f32, dy: f32, dz: f32, ax: f32, ay: f32, az: f32, sx: f32, sy: f32, sz: f32) -> Mat4 {
        /*Mat4([
            ay.cos() * az.cos()    * sx,
            -ay.cos() * az.sin()      * sy,
            ay.sin()               * sz,
            dx,

            (ax.sin() * ay.sin() * az.cos() + ax.cos() * az.sin())   * sx,
            (-ax.sin() * ay.sin() * az.sin() + ax.cos() * az.cos())   * sy,
            -ax.sin() * ay.cos()                                    * sz,
            dy,

            (-ax.cos() * ay.sin() * az.cos() + ax.sin() * az.sin())   * sx,
            (ax.cos() * ay.sin() * az.sin() + ax.sin() * az.cos())   * sy,
            ax.cos() * ay.cos()                                    * sz,
            dz,

            0.0,
            0.0,
            0.0,
            1.0,
        ])*/
        Mat4::translate_mat(dx, dy, dz) * Mat4::rotation_mat(ax, ay, az) * Mat4::scale_mat(sx, sy, sz)

    }

    pub fn scale_mat(sx: f32, sy: f32, sz: f32) -> Mat4 {
        Mat4([
            sx, 0.0, 0.0, 0.0,
            0.0, sy, 0.0, 0.0,
            0.0, 0.0, sz, 0.0,
            0.0, 0.0, 0.0, 1.0
        ])
    }

    //Матрица сдвига
    pub fn translate_mat(dx: f32, dy: f32, dz: f32) -> Mat4 {
        Mat4([
            1.0, 0.0, 0.0, dx,
            0.0, 1.0, 0.0, dy,
            0.0, 0.0, 1.0, dz,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    //Матрица поворота по X
    pub fn rot_x_mat(ax: f32) -> Mat4 {
        Mat4([
            1.0, 0.0,       0.0,        0.0,
            0.0, ax.cos(),  -ax.sin(),  0.0,
            0.0, ax.sin(),  ax.cos(),   0.0,
            0.0, 0.0,       0.0,        1.0,
        ])
    }
    //Матрица поворота по Y
    pub fn rot_y_mat(ay: f32) -> Mat4 {
        Mat4([
            ay.cos(),   0.0,  ay.sin(),  0.0,
            0.0,        1.0,  0.0,       0.0,
            -ay.sin(),  0.0,  ay.cos(),  0.0,
            0.0,        0.0,  0.0,       1.0,
        ])
    }
    //Матрица поворота по Z
    pub fn rot_z_mat(az: f32) -> Mat4 {
        Mat4([
            az.cos(),  -az.sin(),  0.0,  0.0,
            az.sin(),   az.cos(),  0.0,  0.0,
            0.0,       0.0,        1.0,  0.0,
            0.0,       0.0,        0.0,  1.0,
        ])
    }

    //Матрица поворота по трем осям
    pub fn rotation_mat(ax: f32, ay: f32, az: f32) -> Mat4 {
        /*Mat4::rot_z_mat(az) * Mat4::rot_y_mat(ay) * Mat4::rot_x_mat(ax)*/
        Mat4([
            az.cos() * ay.cos(),
            az.cos() * ay.sin() * az.sin() - az.sin() * ax.cos(),
            az.sin() * ax.sin() + az.cos() * ay.sin() * ax.cos(),
            0.0,

            az.sin() * ay.cos(),
            az.cos() * ax.cos() + az.sin() * ay.sin() * ax.sin(),
            az.sin() * ay.sin() * ax.cos() - az.cos() * ax.sin(),
            0.0,

            -ay.sin(),
            ay.cos() * ax.sin(),
            ay.cos() * ax.cos(),
            0.0,

            0.0,
            0.0,
            0.0,
            1.0,
        ])
    }

    //Матрица камеры (Right, Up, Forward, Position)
    pub fn view_mat(rx: f32, ry: f32, rz: f32,
                    ux: f32, uy: f32, uz: f32,
                    fx: f32, fy: f32, fz: f32,
                    x: f32, y: f32, z: f32) -> Mat4 {
        Mat4([rx, ux, fx, -x,
            ry, uy, fy, -y,
            rz, uz, fz, -z,
            0.0, 0.0, 0.0, 1.0]).transpos()
    }

    pub fn default_view_mat() -> Mat4 {
        Mat4([1.0, 0.0, 0.0, 0.0,
              0.0, 0.0, 1.0, 0.0,
              0.0, 1.0, 0.0, 0.0,
              0.0, 0.0, 0.0, 1.0])
    }

    //Камера, сдвинутая в точку, затем повернутая в плоскости yz на vert_ang (при том, что y - вперед), затем в xy на horz_ang
    pub fn cam_mat(vert_ang: f32, horz_ang: f32, x: f32, y: f32, z: f32) -> Mat4 {
        /*Mat4([
            horz_ang.cos(),                     -horz_ang.sin(),                    0.0,                -x * horz_ang.cos() + y * horz_ang.sin(),
            vert_ang.sin() * horz_ang.sin(),    vert_ang.sin() * horz_ang.cos(),    vert_ang.cos(),     -x * vert_ang.sin() * horz_ang.sin() - y * vert_ang.sin() * horz_ang.cos() - z * vert_ang.cos(),
            vert_ang.cos() * horz_ang.sin(),    vert_ang.cos() * horz_ang.cos(),    -vert_ang.sin(),    -x * vert_ang.cos() * horz_ang.sin() - y * vert_ang.cos() * horz_ang.cos() + z * vert_ang.sin(),
            0.0,                                0.0,                                0.0,                1.0,
        ])*/
        Mat4::view_mat(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0) *
        Mat4::rot_x_mat(-vert_ang) *
        Mat4::rot_z_mat(horz_ang) *
        Mat4::translate_mat(-x, -y, -z)
    }

    //Матрица перспективы
    pub fn perspective_mat(fov_y: f32, aspect: f32, z_near: f32, z_far: f32) -> Mat4 {
        let q = 1.0 / (fov_y / 2.0).tan();
        let a = q / aspect;
        let b = (z_near + z_far) / (z_near - z_far);
        let c = (2.0 * z_near * z_far) / (z_near - z_far);

        Mat4([
            a, 0.0, 0.0, 0.0,
            0.0, q, 0.0, 0.0,
            0.0, 0.0, b, c,
            0.0, 0.0, -1.0, 0.0,
        ])
    }

    //Матрица ортографической проекции
    pub fn orthographic_mat(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
        Mat4([
            2.0 / (right - left), 0.0, 0.0, -(right + left) / (right - left),
            0.0, 2.0 / (top - bottom), 0.0, -(top + bottom) / (top - bottom),
            0.0, 0.0, 2.0 / (far - near),  -(far + near) / (far - near),
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    /** Матрица вращения вокруг произвольной оси axis */
    pub fn axis_rotation_mat( axis: &Vec4, angle: f32) -> Mat4 {
        let (ang_y, ang_z) = axis.get_yz_angles();

        Mat4::rotation_mat(0.0, ang_y, ang_z) *
        Mat4::rot_x_mat(angle) *
        Mat4::rot_y_mat(-ang_y) *
        Mat4::rot_z_mat(-ang_z)
    }
}

impl AsRef<Mat4> for Mat4 {
    fn as_ref(&self) -> &Mat4 {
        self
    }
}

//Определитель фрагмента 3х3
pub fn sub_det(m: [f32; 9]) -> f32 {
    m[0] * m[4] * m[8] +
    m[2] * m[3] * m[7] +
    m[1] * m[5] * m[6] -

    m[2] * m[4] * m[6] -
    m[0] * m[5] * m[7] -
    m[1] * m[3] * m[8]
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Vec4  { Vec4 { x,       y,       z,       w }       }
    pub fn from_slice(s: [f32; 4]) -> Vec4              { Vec4 { x: s[0], y: s[1], z: s[2], w: s[3] } }

    pub fn len(&self) -> f32 {
        (self.x.powf(2.0) +
         self.y.powf(2.0) +
         self.z.powf(2.0) +
         self.w.powf(2.0) ).sqrt()
    }

    /** Единичный вектор, совпадающий направлением с данным*/
    pub fn unit(&self) -> Self {
        *self / self.len()
    }

    pub fn dot(&self, rhs: &Vec4) -> f32 {
        self.x() * rhs.x() +
        self.y() * rhs.y() +
        self.z() * rhs.z() +
        self.w() * rhs.w()
    }

    /** Возвращает два угла: повернув вектор {1; 0; 0} вокруг оси Y на первый угол,
        затем вокруг оси Z на второй угол - получится единичный вектор направления,
        идентичного  оригинальному
        Mat4::rotation_mat(0.0, vec.get_yz_angles().0, vec.get_yz_angles().1) == vec / vec.len() */
    pub fn get_yz_angles(&self) -> (f32, f32) {
        Vec3::new(self.x, self.y, self.z).get_yz_angles()
    }

    pub fn x(&self) -> f32 { self.x }
    pub fn y(&self) -> f32 { self.y }
    pub fn z(&self) -> f32 { self.z }
    pub fn w(&self) -> f32 { self.w }
}


#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3  {
        Vec3 { x,       y,       z }
    }
    pub fn from_slice(s: [f32; 3]) -> Vec3 {
        Vec3 { x: s[0], y: s[1], z: s[2] }
    }

    pub fn len(&self) -> f32 {
        (self.x.powf(2.0) +
         self.y.powf(2.0) +
         self.z.powf(2.0)  ).sqrt()
    }

    pub fn dot(&self, rhs: &Vec3) -> f32 {
        self.x() * rhs.x() +
        self.y() * rhs.y() +
        self.z() * rhs.z()
    }

    /** Возвращает два угла: повернув вектор {1; 0; 0} вокруг оси Y на первый угол,
           затем вокруг оси Z на второй угол - получится единичный вектор направления,
           идентичного  оригинальному
           Mat4::rotation_mat(0.0, vec.get_yz_angles().0, vec.get_yz_angles().1) == vec / vec.len() */
    pub fn get_yz_angles(&self) -> (f32, f32) {
        let vec = *self / self.len(); //Единичный вектор
        let xy_len = (vec.x().powf(2.0) + vec.y().powf(2.0)).sqrt(); //Длина проекции на плоскость xy
        //println!("-====---= Self: {:?}, len: {}, Vec: {:?}, XY len: {}", self, self.len(), vec, xy_len);

        let ang_y = ( xy_len ).acos();
        if xy_len == 0.0 {
            return ( if vec.z() >= 0.0 { -ang_y } else { ang_y }, 0.0)
        }

        let ang_z = ( vec.x() / xy_len ).acos();
        (
            if vec.z() >= 0.0 { -ang_y } else { ang_y },
            if vec.y() >= 0.0 { ang_z } else { -ang_z }
        )
    }

    pub fn x(&self) -> f32 { self.x }
    pub fn y(&self) -> f32 { self.y }
    pub fn z(&self) -> f32 { self.z }
}