#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(async_fn_in_trait)]

mod graph;
pub mod provider;
mod resolve;
mod target;

pub use resolve::Resolve;
pub use resolve::ResolveExt;
pub use target::Target;
