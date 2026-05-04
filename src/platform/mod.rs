pub mod api {
    pub use super::domain::{PlatformRole, PlatformUser};
    pub use super::dto::{
        CreateUserRequest, CreateUserResponse, LoginRequest, LoginResponse, MeResponse,
    };
}

pub(crate) mod handler;
pub(crate) mod service;

mod domain;
mod dto;
mod error;
mod query;
