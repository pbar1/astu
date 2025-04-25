use crate::action::client::DynamicClientFactory;
use crate::action::AuthPayload;
use crate::db::DbImpl;
use crate::resolve::provider::ChainResolver;
use crate::resolve::Target;
use crate::resolve::TargetGraph;

pub struct Engine {
    forward_resolver: ChainResolver,
    reverse_resolver: ChainResolver,
    client_factory: DynamicClientFactory,
    db: DbImpl,
}

pub struct JobQuery {
    pub targets: Vec<Target>,
}

pub struct JobPlan {
    pub graph: TargetGraph,
}

pub struct Task {
    pub target: Target,
    pub actions: Vec<Action>,
}

pub enum Action {
    Connect,
    Auth { payload: AuthPayload },
    RunCommand { command: String },
}
