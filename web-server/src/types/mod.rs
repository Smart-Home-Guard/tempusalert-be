use crate::handlers;

pub type AppResult<T = ()> = std::result::Result<T, handlers::AppError>;
