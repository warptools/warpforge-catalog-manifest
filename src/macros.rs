#[macro_export]
macro_rules! errf {
    ($format_string:expr $(, $args:expr)*) => {
       Box::new(StrError::new(format!($format_string $(, $args)*).as_str()))
    };
}
