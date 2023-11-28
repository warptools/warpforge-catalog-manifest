use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct StrError {
    message: String,
}

impl StrError {
    pub fn new(message: &str) -> Self {
        StrError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for StrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl Error for StrError {}

#[allow(unused_macros)]
macro_rules! errf {
    ($format_string:expr $(, $args:expr)*) => {
       Box::new(StrError::new(format!($format_string $(, $args)*).as_str()))
    };
}
#[allow(unused_imports)]
pub(crate) use errf;

#[macro_use]
pub mod macros {
    #[allow(unused_imports)]
    pub(crate) use super::errf;
}
