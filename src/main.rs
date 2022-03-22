//! discord-db is a Rust Discord BOT.
//!
//! To simply runs this bot fill the credentials.json file at the root of your directory with your informations
//!
//! # Credential.json
//! ```json
//! {
//!   "email": "your@email.io",
//!   "password": "password",
//!   "domain": "ssl0.ovh.net",
//!   "token": "YOURDISCORDTOKEN"
//! }
//! ```
//!
//! And run `cargo run`
//!
//! This bot is compose of 2 modules:
//!
//!  *  [Core][core docs] Wich is the active connection with discord and manage the events.
//!
//!  *  [Features][features docs] The features that the bot do.
//!
//!
//! [core docs]: core/index.html
//! [features docs]: features/index.html

#![warn(clippy::all)]
#![feature(drain_filter)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
#[macro_use]
pub mod macros;
#[macro_use]
extern crate rocket;

mod constants;
mod core;
mod database;
mod features;

use dotenv::dotenv;
use std::env;

fn main() {
  env::set_var("RUST_BACKTRACE", "full");
  env::set_var("RUST_LOG", "rbot_discord,rocket");
  dotenv().ok();
  pretty_env_logger::init();

  let _ = core::run().join().unwrap();
}
