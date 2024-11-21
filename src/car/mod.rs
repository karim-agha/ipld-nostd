//! Implementation of the [car](https://ipld.io/specs/transport/car/) format.

mod error;
mod header;
mod reader;
mod util;
mod writer;

pub use error::Error;
pub use header::CarHeader;
pub use reader::CarReader;
pub use writer::CarWriter;
