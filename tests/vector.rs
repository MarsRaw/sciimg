mod common;

use sciimg::vector::*;

#[test]
fn test_vector_indexing_nonmut() {
    let v0 = Vector {
        x: 0.0,
        y: 1.0,
        z: 2.0,
    };
    assert!(v0.x == 0.0);
    assert!(v0.y == 1.0);
    assert!(v0.z == 2.0);

    assert!(v0[0] == 0.0);
    assert!(v0[1] == 1.0);
    assert!(v0[2] == 2.0);
}

#[test]
#[should_panic]
fn test_vector_indexing_nonmut_bounds() {
    let v0 = Vector {
        x: 0.0,
        y: 1.0,
        z: 2.0,
    };
    assert!(v0[3] == 3.0);
}

#[test]
fn test_vector_indexing_mut() {
    let mut v0 = Vector {
        x: 0.0,
        y: 1.0,
        z: 2.0,
    };

    assert!(v0.x == 0.0);
    assert!(v0.y == 1.0);
    assert!(v0.z == 2.0);

    assert!(v0[0] == 0.0);
    assert!(v0[1] == 1.0);
    assert!(v0[2] == 2.0);

    v0[0] = 5.0;
    v0[1] = 6.0;
    v0[2] = 7.0;

    assert!(v0.x == 5.0);
    assert!(v0.y == 6.0);
    assert!(v0.z == 7.0);

    assert!(v0[0] == 5.0);
    assert!(v0[1] == 6.0);
    assert!(v0[2] == 7.0);
}

#[test]
#[should_panic]
fn test_vector_indexing_mut_bounds() {
    let mut v0 = Vector {
        x: 0.0,
        y: 1.0,
        z: 2.0,
    };
    v0[3] = 3.0;
}
