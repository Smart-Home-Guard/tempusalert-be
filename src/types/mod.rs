use crate::handlers;

pub type Result<T> = std::result::Result<T, handlers::Error>;
