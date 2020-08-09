#![allow(warnings)]

#[macro_use]
extern crate log;

mod client;
mod epd;
mod util;

pub use client::*;
pub use epd::*;
pub use util::*;

pub type EpdResult<T> = Result<T, EpdError>;
