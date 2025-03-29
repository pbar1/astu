use crate::resolve::Resolve;
use crate::target::Target;

pub struct ResolveChain {
    resolvers: Vec<Box<dyn Resolve>>,
}
