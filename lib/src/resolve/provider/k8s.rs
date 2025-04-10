use std::str::FromStr;

use anyhow::Context;
use anyhow::Result;
use async_stream::try_stream;
use futures::stream::BoxStream;
use futures::StreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::api::ListParams;
use kube::api::PartialObjectMeta;
use kube::Api;
use kube::Client;
use kube::Resource;
use kube::ResourceExt;

use crate::resolve::Resolve;
use crate::resolve::Target;

/// Resolves DNS queries - both forward and reverse - into targets.
#[derive(Debug, Clone)]
pub struct K8sResolver {}

impl Resolve for K8sResolver {
    fn resolve_fallible(&self, target: Target) -> BoxStream<Result<Target>> {
        todo!()
    }
}

async fn resolve_namespace(namespace: &str) -> Result<Vec<Target>> {
    let client = Client::try_default().await?;
    let pods: Api<Pod> = Api::default_namespaced(client);

    pods.list_metadata(&ListParams::default())
        .await?
        .iter()
        .map(target_from_pod_metadata)
        .collect()
}

fn target_from_pod_metadata(pod: &PartialObjectMeta<Pod>) -> Result<Target> {
    let mut s = "k8s:".to_owned();
    if let Some(namespace) = pod.namespace() {
        s.push_str(&namespace);
        s.push('/');
    }
    s.push_str(&pod.name_any());
    Target::from_str(&s).context("unable to build pod uri")
}
