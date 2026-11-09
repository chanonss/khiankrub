//! The browser side: Google sign-in, Picker, Drive transfers and the connection service.

mod api;
mod auth;
mod controller;
mod picker;
mod session;

pub use controller::{Drive, Host};
