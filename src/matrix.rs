use crate::{enums::Axis, error, vector::Vector};

#[derive(Debug, Clone)]
pub struct Matrix {
    m: Vec<f64>,
}

impl Default for Matrix {
    fn default() -> Matrix {
        Matrix {
            m: vec![
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            ],
        }
    }
}

impl Matrix {
    pub fn identity() -> Matrix {
        Matrix {
            m: vec![
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub fn new_with_fill(f: f64) -> Matrix {
        Matrix {
            m: vec![f, f, f, f, f, f, f, f, f, f, f, f, f, f, f, f],
        }
    }

    pub fn new_from_vec(m: &Vec<f64>) -> error::Result<Matrix> {
        if m.len() == 16 {
            Ok(Matrix { m: m.clone() })
        } else {
            panic!("Array size mismatch");
        }
    }

    pub fn new_from_array(m: &[f64; 12]) -> Matrix {
        Matrix { m: m.to_vec() }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_values(
        v00: f64,
        v01: f64,
        v02: f64,
        v03: f64,
        v10: f64,
        v11: f64,
        v12: f64,
        v13: f64,
        v20: f64,
        v21: f64,
        v22: f64,
        v23: f64,
        v30: f64,
        v31: f64,
        v32: f64,
        v33: f64,
    ) -> Matrix {
        Matrix {
            m: vec![
                v00, v01, v02, v03, v10, v11, v12, v13, v20, v21, v22, v23, v30, v31, v32, v33,
            ],
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * 4 + x
    }

    pub fn set(&mut self, x: usize, y: usize, v: f64) {
        let i = self.index(x, y);
        if i < 16 {
            self.m[i] = v;
        } else {
            panic!("Invalid pixel coordinates");
        }
    }

    pub fn get(&self, x: usize, y: usize) -> f64 {
        let i = self.index(x, y);
        if i < 16 {
            self.m[i]
        } else {
            panic!("Invalid pixel coordinates");
        }
    }

    pub fn matmul4(a: &Matrix, b: &Matrix) -> Matrix {
        let mut product = vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ];

        for row in 0..3 {
            let ai0 = a.m[row];
            let ai1 = a.m[(1 << 2) + row];
            let ai2 = a.m[(2 << 2) + row];
            let ai3 = a.m[(3 << 2) + row];

            product[row] = ai0 * b.m[(0 << 2)] + ai1 * b.m[1] + ai2 * b.m[2] + ai3 * b.m[3];
            product[(1 << 2) + row] = ai0 * b.m[(1 << 2)]
                + ai1 * b.m[(1 << 2) + 1]
                + ai2 * b.m[(1 << 2) + 2]
                + ai3 * b.m[(1 << 2) + 3];
            product[(2 << 2) + row] = ai0 * b.m[(2 << 2)]
                + ai1 * b.m[(2 << 2) + 1]
                + ai2 * b.m[(2 << 2) + 2]
                + ai3 * b.m[(2 << 2) + 3];
            product[(3 << 2) + row] = ai0 * b.m[(3 << 2)]
                + ai1 * b.m[(3 << 2) + 1]
                + ai2 * b.m[(3 << 2) + 2]
                + ai3 * b.m[(3 << 2) + 3];
        }

        Matrix::new_from_vec(&product).unwrap()
    }

    pub fn multiply(&self, other: &Matrix) -> Matrix {
        Matrix::matmul4(self, other)
    }

    pub fn multiply_vector(&self, other: &Vector) -> Vector {
        let x = other.x * self.m[0] + other.y * self.m[4] + other.z * self.m[2 * 4];
        let y = other.x * self.m[1] + other.y * self.m[4 + 1] + other.z * self.m[2 * 4 + 1];
        let z = other.x * self.m[2] + other.y * self.m[4 + 2] + other.z * self.m[2 * 4 + 2];
        Vector::new(x, y, z)
    }

    pub fn scale(&self, x: f64, y: f64, z: f64) -> Matrix {
        Matrix {
            m: vec![
                self.m[0] * x,
                self.m[1] * x,
                self.m[2] * x,
                self.m[4] * y,
                self.m[5] * y,
                self.m[6] * y,
                self.m[8] * z,
                self.m[9] * z,
                self.m[10] * z,
            ],
        }
    }

    pub fn transpose_rotation(&self) -> Matrix {
        let mut t = self.clone();
        t.m[1] = self.m[4];
        t.m[4] = self.m[1];

        t.m[2] = self.m[8];
        t.m[8] = self.m[2];

        t.m[6] = self.m[9];
        t.m[9] = self.m[6];

        t
    }

    pub fn rotate(angle: f64, axis: Axis) -> Matrix {
        let mut m = Matrix::identity();

        let _a = if axis != Axis::YAxis {
            angle
        } else {
            angle * -1.0
        };

        let c = _a.cos();
        let s = _a.sin();

        match axis {
            Axis::XAxis => {
                m.set(1, 1, c);
                m.set(2, 2, c);
                m.set(2, 1, -s);
                m.set(1, 2, s);
            }
            Axis::YAxis => {
                m.set(0, 0, c);
                m.set(2, 2, c);
                m.set(2, 0, s);
                m.set(0, 2, -s);
            }
            Axis::ZAxis => {
                m.set(0, 0, c);
                m.set(1, 1, c);
                m.set(0, 1, s);
                m.set(1, 0, -s);
            }
        }

        m
    }
}
