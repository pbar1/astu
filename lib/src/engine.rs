use bon::Builder;

use crate::action::client::DynamicClientFactory;
use crate::action::AuthPayload;
use crate::db::DbImpl;
use crate::resolve::provider::ChainResolver;
use crate::resolve::ResolveExt;
use crate::resolve::Target;
use crate::resolve::TargetGraph;
use crate::util::id::Id;
use crate::util::id::IdGenerator;
use crate::util::id::IdGeneratorImpl;

#[derive(Builder)]
pub struct Engine {
    id_generator: IdGeneratorImpl,
    forward_resolver: ChainResolver,
    reverse_resolver: ChainResolver,
    _client_factory: DynamicClientFactory,
    _db: DbImpl,
}

impl Engine {
    /// Given some initial seed targets, creates a job plan.
    pub async fn job_plan(&self, targets: impl IntoIterator<Item = Target>) -> JobPlan {
        let id = self.id_generator.id_now();

        let mut graph = TargetGraph::default();
        for target in targets {
            self.forward_resolver
                .resolve_into_graph(target, &mut graph)
                .await;
        }
        for target in graph.nodes() {
            let target = target.clone();
            self.reverse_resolver
                .resolve_into_graph_reverse(target, &mut graph)
                .await;
        }

        JobPlan { id, graph }
    }
}

#[derive(Debug, Clone)]
pub struct JobPlan {
    pub id: Id,
    pub graph: TargetGraph,
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
    Auth { payload: AuthPayload },
    RunCommand { command: String },
}
