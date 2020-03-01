#![feature(try_trait)]

extern crate bufstream;
extern crate native_tls;
#[macro_use]
extern crate prettytable;
#[macro_use]
extern crate log;

/// Classic code for reference
mod nntp;
pub use self::nntp::*;

/// Over-engineered magic code 8-)
pub mod capabilities;
pub mod client;
pub mod error;
pub mod prelude;
pub mod response;
pub mod stream;
