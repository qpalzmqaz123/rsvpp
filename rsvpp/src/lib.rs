mod client;
mod error;
mod hard_coded_message;
pub mod message;
mod session;
mod stats;
mod transport;

pub use client::Client;
pub use error::Error;
pub use lazy_static;
pub use pack;
pub use session::{RecvEntry, Session};
pub use stats::Stats;
pub use transport::Transport;

pub type Result<T> = std::result::Result<T, Error>;

pub static CLIENT_NAME: &str = "rsvpp";
