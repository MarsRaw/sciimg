use crate::{matrix::Matrix, vector::Vector};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion {
    q0: f64,
    q1: f64,
    q2: f64,
    q3: f64,
}

impl Default for Quaternion {
    fn default() -> Self {
        Quaternion {
            q0: 1.0,
            q1: 0.0,
            q2: 0.0,
            q3: 0.0,
        }
    }
}

// Adapted from some old code I wrote years ago: https://github.com/kmgill/jdem846/blob/master/jdem846/source/base/src/us/wthr/jdem846/math/Quaternion.java
// which itself was adapter from something, I just forget from where.... Probably three.js
impl Quaternion {
    pub fn from_axis_and_angle(axis: &Vector, angle: f64) -> Self {
        let half_theta = angle / 2.0;
        let q0 = half_theta.cos();
        let sin_half_theta = half_theta.sin();

        let real_axis = axis.normalized();

        Quaternion {
            q0,
            q1: real_axis.x * sin_half_theta,
            q2: real_axis.y * sin_half_theta,
            q3: real_axis.z * sin_half_theta,
        }
    }

    pub fn from_pitch_roll_yaw(roll: f64, pitch: f64, yaw: f64) -> Self {
        let roll_q = Quaternion::from_axis_and_angle(&Vector::new(1.0, 0.0, 0.0), roll);
        let pitch_q = Quaternion::from_axis_and_angle(&Vector::new(0.0, 1.0, 0.0), pitch);
        let yaw_q = Quaternion::from_axis_and_angle(&Vector::new(0.0, 0.0, 1.0), yaw);

        yaw_q.times(&pitch_q).times(&roll_q)
    }

    pub fn from_matrix(mat: &Matrix) -> Quaternion {
        let tr = mat.get(0, 0) + mat.get(1, 1) + mat.get(2, 2);

        if tr > 0.0 {
            let mut s = (tr + 1.0).sqrt();
            let q0 = s * 0.5;
            s = 0.5 / s;
            Quaternion {
                q0,
                q1: (mat.get(2, 1) - mat.get(1, 2)) * s,
                q2: (mat.get(0, 2) - mat.get(2, 0)) * s,
                q3: (mat.get(1, 0) - mat.get(0, 1)) * s,
            }
        } else {
            let mut i = if mat.get(1, 1) > mat.get(0, 0) { 1 } else { 0 };
            if mat.get(2, 2) > mat.get(i, i) {
                i = 2;
            }
            let j = (i + 1) % 3;
            let k = (j + 1) % 3;
            let mut s = ((mat.get(i, i) - (mat.get(j, j) + mat.get(k, k))) + 1.0).sqrt();
            let mut q = Quaternion::default();
            q.set_q(i + 1, s * 0.5);
            s = 0.5 / s;
            q.q0 = (mat.get(k, j) - mat.get(j, k)) * s;
            q.set_q(j + 1, (mat.get(j, i) + mat.get(i, j)) * s);
            q.set_q(k + 1, (mat.get(k, i) + mat.get(i, k)) * s);
            q
        }
    }

    pub fn within_epsilon(&self, other: &Quaternion, epsilon: f64) -> bool {
        (self.q0 - other.q0).abs() < epsilon
            && (self.q1 - other.q1).abs() < epsilon
            && (self.q2 - other.q2).abs() < epsilon
            && (self.q3 - other.q3).abs() < epsilon
    }

    pub fn get(&self) -> (Vector, f64) {
        let retval = 2.0 * self.q0.acos();
        let mut axis = Vector::new(self.q1, self.q2, self.q3);
        let len = axis.len();
        if len == 0.0 {
            (Vector::new(0.0, 0.0, 1.0), retval)
        } else {
            axis = axis.scale(1.0 / len);
            (axis, retval)
        }
    }

    pub fn set_q(&mut self, i: usize, val: f64) {
        match i {
            0 => self.q0 = val,
            1 => self.q1 = val,
            2 => self.q2 = val,
            3 => self.q3 = val,
            _ => panic!("Invalid quaternion index"),
        };
    }

    pub fn invert(&self) -> Quaternion {
        Quaternion {
            q0: self.q0,
            q1: self.q1 * -1.0,
            q2: self.q2 * -1.0,
            q3: self.q3 * -1.0,
        }
    }

    pub fn length(&self) -> f64 {
        (self.q0 * self.q0 + self.q1 * self.q1 + self.q2 * self.q2 + self.q3 * self.q3).sqrt()
    }

    pub fn normalized(&self) -> Quaternion {
        let len = self.length();
        Quaternion {
            q0: self.q0 / len,
            q1: self.q1 / len,
            q2: self.q2 / len,
            q3: self.q3 / len,
        }
    }

    pub fn times(&self, other: &Quaternion) -> Quaternion {
        Quaternion::mul(self, other)
    }

    pub fn mul(a: &Quaternion, b: &Quaternion) -> Quaternion {
        Quaternion {
            q0: (a.q0 * b.q0 - a.q1 * b.q1 - a.q2 * b.q2 - a.q3 * b.q3),
            q1: (a.q0 * b.q1 + a.q1 * b.q0 + a.q2 * b.q3 - a.q3 * b.q2),
            q2: (a.q0 * b.q2 + a.q2 * b.q0 - a.q1 * b.q3 + a.q3 * b.q1),
            q3: (a.q0 * b.q3 + a.q3 * b.q0 + a.q1 * b.q2 - a.q2 * b.q1),
        }
    }

    pub fn to_matrix(&self) -> Matrix {
        let q00 = self.q0 * self.q0;
        let q11 = self.q1 * self.q1;
        let q22 = self.q2 * self.q2;
        let q33 = self.q3 * self.q3;

        let mut m = Matrix::default();

        m.set(0, 0, q00 + q11 - q22 - q33);
        m.set(1, 1, q00 - q11 + q22 - q33);
        m.set(2, 2, q00 - q11 - q22 + q33);

        let q03 = self.q0 * self.q3;
        let q12 = self.q1 * self.q2;
        m.set(1, 0, 2.0 * (q12 - q03));
        m.set(0, 1, 2.0 * (q03 + q12));

        let q02 = self.q0 * self.q2;
        let q13 = self.q1 * self.q3;
        m.set(2, 0, 2.0 * (q02 + q13));
        m.set(0, 2, 2.0 * (q13 - q02));

        let q01 = self.q0 * self.q1;
        let q23 = self.q2 * self.q3;
        m.set(2, 1, 2.0 * (q23 - q01));
        m.set(1, 2, 2.0 * (q01 + q23));

        m
    }

    pub fn rotate_vector(&self, src: &Vector) -> Vector {
        let qvec = Vector::new(self.q1, self.q2, self.q3);

        let mut q_cross_x = qvec.cross_product(src);
        let mut q_cross_x_cross_q = q_cross_x.cross_product(&qvec);

        q_cross_x = q_cross_x.scale(2.0 * self.q0);
        q_cross_x_cross_q = q_cross_x_cross_q.scale(-2.0);

        src.add(&q_cross_x).add(&q_cross_x_cross_q)
    }
}
