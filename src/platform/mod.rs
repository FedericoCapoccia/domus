pub mod api {
    pub use super::domain::{PlatformRole, PlatformStatus, PlatformUser};
    pub use super::dto::{
        CreateUserRequest, CreateUserResponse, LoginRequest, LoginResponse, MeResponse,
    };
}

pub(crate) mod error;
pub(crate) mod handler;
pub(crate) mod service;

mod domain;
mod dto;
mod query;
