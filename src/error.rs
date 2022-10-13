use std::result;

pub type Result<T> = result::Result<T, &'static str>;

#[macro_export]
macro_rules! ok {
    () => {
        Ok("ok")
    };
}

#[macro_export]
macro_rules! not_implemented {
    () => {
        Err("not implemented")
    };
}
