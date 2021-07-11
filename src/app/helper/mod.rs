pub mod files;
pub mod list;
pub mod terminal;

#[macro_export]
macro_rules! unwrap_or_ok {
    ($input:expr) => {
        if let Some(inner) = $input {
            inner
        } else {
            return Ok(());
        };
    };
}
