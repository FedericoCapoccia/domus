pub mod api {
    pub use super::domain::PlatformRole;
    pub use super::dto::{CreateUserRequest, CreateUserResponse, LoginRequest, LoginResponse};
}

pub(crate) mod handler;
pub(crate) mod service;

mod domain;
mod dto;
mod error;
