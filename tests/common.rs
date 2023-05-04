use sciimg::*;

// https://stackoverflow.com/questions/30856285/assert-eq-with-floating-point-numbers-and-delta
#[macro_export]
macro_rules! assert_delta {
    ($x:expr, $y:expr, $d:expr) => {
        if !($x - $y < $d || $y - $x < $d) {
            panic!();
        }
    };
}

#[allow(dead_code)]
pub const DEFAULT_DELTA: Dn = 0.0001;
