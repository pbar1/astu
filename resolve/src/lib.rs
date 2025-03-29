#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(async_fn_in_trait)]

mod graph;
mod provider;
mod resolve;
mod target;

pub use graph::TargetGraph;
pub use provider::cidr::CidrResolver;
pub use provider::dns::DnsResolver;
pub use provider::file::FileResolver;
pub use resolve::Resolve;
pub use resolve::ResolveExt;
pub use target::Target;
