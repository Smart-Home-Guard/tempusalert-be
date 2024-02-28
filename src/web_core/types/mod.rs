use crate::web_core::handlers;

pub type AppResult<T = ()> = std::result::Result<T, handlers::AppError>;
