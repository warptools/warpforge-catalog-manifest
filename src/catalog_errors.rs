use std::error::Error;

pub struct MalformedCatalog {
    path: String,
    message: String,
    cause: dyn Error,
}

impl Error for MalformedCatalog {}
