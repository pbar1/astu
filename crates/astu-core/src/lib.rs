mod id;
mod util;

use std::collections::BTreeSet;

use astu_resolve::ChainResolver;
use astu_resolve::ResolveExt;
use astu_types::Target;
use bon::Builder;

pub use crate::id::Id;
pub use crate::id::IdGenerator;
pub use crate::id::IdGeneratorImpl;
pub use crate::util::AstuTryFutureExt;
pub use crate::util::AstuTryStreamExt;

#[derive(Builder)]
pub struct Engine {
    id_generator: IdGeneratorImpl,
    forward_resolver: ChainResolver,
    reverse_resolver: ChainResolver,
}

impl Engine {
    /// Given some initial seed targets, creates a job plan.
    pub async fn job_plan(&self, targets: impl IntoIterator<Item = Target>) -> JobPlan {
        let id = self.id_generator.id_now();

        let mut resolved = BTreeSet::new();
        for target in targets {
            self.forward_resolver
                .resolve_into_set(target, &mut resolved)
                .await;
        }

        let snapshot: Vec<_> = resolved.iter().cloned().collect();
        for target in snapshot {
            self.reverse_resolver
                .resolve_into_set(target, &mut resolved)
                .await;
        }

        JobPlan {
            id,
            targets: resolved,
        }
    }
}

#[derive(Debug, Clone)]
pub struct JobPlan {
    pub id: Id,
    pub targets: BTreeSet<Target>,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub target: Target,
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone)]
pub enum Action {
    Connect,
    Ping,
    RunCommand { command: String },
}
