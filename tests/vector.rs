use sciimg::*;

// https://stackoverflow.com/questions/30856285/assert-eq-with-floating-point-numbers-and-delta
macro_rules! assert_delta {
    ($x:expr, $y:expr, $d:expr) => {
        if !($x - $y < $d || $y - $x < $d) {
            panic!();
        }
    };
}

const DEFAULT_DELTA: Dn = 0.0001;

#[test]
fn test_dnvec() {
    let mut v = DnVec::fill(100, 0.0);

    assert_eq!(v.min(), 0.0);
    assert_eq!(v.max(), 0.0);
    assert_eq!(v.get_min_max().min, 0.0);
    assert_eq!(v.get_min_max().max, 0.0);

    v[50] = 50.0;
    assert_eq!(v[50], 50.0);
    assert_eq!(v.min(), 0.0);
    assert_eq!(v.max(), 50.0);
    assert_eq!(v.get_min_max().min, 0.0);
    assert_eq!(v.get_min_max().max, 50.0);
    assert_eq!(v.stddev(), 4.974937);
    assert_eq!(v.variance(), 24.75);

    v[20] = 20.0;
    assert_eq!(v[20], 20.0);
    assert_eq!(v.min(), 0.0);
    assert_eq!(v.max(), 50.0);
    assert_eq!(v.get_min_max().min, 0.0);
    assert_eq!(v.get_min_max().max, 50.0);
    assert_eq!(v.stddev(), 5.339475);
    assert_eq!(v.variance(), 28.509993);

    let mut v0 = DnVec::fill(100, 0.0);
    let v1 = DnVec::fill(100, 100.0);

    v0 = v0.add(&v1);
    assert_eq!(v0[50], 100.0);

    v0.add_mut(&v1);
    assert_eq!(v0[50], 200.0);

    v0 = v0.add(&v1);
    v0 = v0.add(&v1);
    assert_eq!(v0[50], 400.0);

    v0.divide_into_mut(4.0);
    assert_eq!(v0[50], 100.0);

    v0 = v0.add(&v1);
    v0 = v0.divide_into(2.0);
    assert_eq!(v0[50], 100.0);

    v0 = v0.divide(&v1);
    assert_eq!(v0[50], 1.0);

    v0 = v0.add(&v1);
    v0.divide_mut(&v1);
    assert_eq!(v0[50], 1.01);

    v0 = v0.add(&v1);
    assert_delta!(v0[50], 101.01, DEFAULT_DELTA);

    v0 = v0.subtract(&v1);
    assert_delta!(v0[50], 1.01, DEFAULT_DELTA);

    v0 = v0.add(&v1);
    assert_delta!(v0[50], 101.01, DEFAULT_DELTA);

    v0.subtract_mut(&v1);
    assert_delta!(v0[50], 1.01, DEFAULT_DELTA);

    v0 = v0.multiply(&v1);
    assert_delta!(v0[50], 101.01, DEFAULT_DELTA);

    v0.multiply_mut(&v1);
    assert_delta!(v0[50], 10101.0, DEFAULT_DELTA);

    v0 = v0.scale(0.01);
    assert_delta!(v0[50], 101.01, DEFAULT_DELTA);

    v0.scale_mut(0.01);
    assert_delta!(v0[50], 1.01, DEFAULT_DELTA);
}

#[test]
fn test_maskeddnvec() {
    let mut v = MaskedDnVec::fill(100, 0.0);

    assert_eq!(v.min(), 0.0);
    assert_eq!(v.max(), 0.0);
    assert_eq!(v.get_min_max().min, 0.0);
    assert_eq!(v.get_min_max().max, 0.0);

    v[50] = 50.0;
    assert_eq!(v[50], 50.0);
    assert_eq!(v.min(), 0.0);
    assert_eq!(v.max(), 50.0);
    assert_eq!(v.get_min_max().min, 0.0);
    assert_eq!(v.get_min_max().max, 50.0);
    assert_eq!(v.stddev(), 4.974937);
    assert_eq!(v.variance(), 24.75);

    v[20] = 20.0;
    assert_eq!(v[20], 20.0);
    assert_eq!(v.min(), 0.0);
    assert_eq!(v.max(), 50.0);
    assert_eq!(v.get_min_max().min, 0.0);
    assert_eq!(v.get_min_max().max, 50.0);
    assert_eq!(v.stddev(), 5.339475);
    assert_eq!(v.variance(), 28.509993);

    v.mask[50] = false;
    assert_eq!(v[50], 0.0);
    v[50] = 30.0;
    assert_eq!(v[50], 0.0);
    assert_eq!(v.min(), 0.0);
    assert_eq!(v.max(), 20.0);
    assert_eq!(v.get_min_max().min, 0.0);
    assert_eq!(v.get_min_max().max, 20.0);
    assert_eq!(v.stddev(), 1.9899765);
    assert_eq!(v.variance(), 3.9600065);

    v.mask[50] = true;
    assert_eq!(v[50], 50.0);
    assert_eq!(v.min(), 0.0);
    assert_eq!(v.max(), 50.0);
    assert_eq!(v.get_min_max().min, 0.0);
    assert_eq!(v.get_min_max().max, 50.0);
    assert_eq!(v.stddev(), 5.339475);
    assert_eq!(v.variance(), 28.509993);

    v[50] = 40.0;
    assert_eq!(v[50], 40.0);
    assert_eq!(v.min(), 0.0);
    assert_eq!(v.max(), 40.0);
    assert_eq!(v.get_min_max().min, 0.0);
    assert_eq!(v.get_min_max().max, 40.0);
    assert_eq!(v.stddev(), 4.4317026);
    assert_eq!(v.variance(), 19.63999);

    let mut v0 = MaskedDnVec::fill(100, 0.0);
    let v1 = MaskedDnVec::fill(100, 100.0);

    v0 = v0.add(&v1);
    assert_eq!(v0[50], 100.0);

    v0.add_mut(&v1);
    assert_eq!(v0[50], 200.0);

    v0 = v0.add(&v1);
    v0 = v0.add(&v1);
    assert_eq!(v0[50], 400.0);

    v0.divide_into_mut(4.0);
    assert_eq!(v0[50], 100.0);

    v0 = v0.add(&v1);
    v0 = v0.divide_into(2.0);
    assert_eq!(v0[50], 100.0);

    v0 = v0.divide(&v1);
    assert_eq!(v0[50], 1.0);

    v0 = v0.add(&v1);
    v0.divide_mut(&v1);
    assert_eq!(v0[50], 1.01);

    v0 = v0.add(&v1);
    assert_delta!(v0[50], 101.01, DEFAULT_DELTA);

    v0 = v0.subtract(&v1);
    assert_delta!(v0[50], 1.01, DEFAULT_DELTA);

    v0 = v0.add(&v1);
    assert_delta!(v0[50], 101.01, DEFAULT_DELTA);

    v0.subtract_mut(&v1);
    assert_delta!(v0[50], 1.01, DEFAULT_DELTA);

    v0 = v0.multiply(&v1);
    assert_delta!(v0[50], 101.01, DEFAULT_DELTA);

    v0.multiply_mut(&v1);
    assert_delta!(v0[50], 10101.0, DEFAULT_DELTA);

    v0 = v0.scale(0.01);
    assert_delta!(v0[50], 101.01, DEFAULT_DELTA);

    v0.scale_mut(0.01);
    assert_delta!(v0[50], 1.01, DEFAULT_DELTA);
}
