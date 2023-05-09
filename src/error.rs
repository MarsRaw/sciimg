use std::result;

#[deprecated]
pub type Result<T> = result::Result<T, &'static str>;

#[macro_export]
macro_rules! ok {
    () => {
        Ok("ok")
    };
}
