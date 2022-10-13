use sciimg::*;

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

    // And so ...
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
    // And so ...
}
