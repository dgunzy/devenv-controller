use crate::crd::DevEnvironment;
use crate::errors::{Error, Result};
use kube::runtime::controller::Action;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, instrument};

/// Context passed to the reconciler
pub struct Context {
    /// Kubernetes client
    pub client: kube::Client,
}

/// Main reconciliation function
#[instrument(skip(ctx))]
pub async fn reconcile(dev_env: Arc<DevEnvironment>, ctx: Arc<Context>) -> Result<Action> {
    let name = dev_env.metadata.name.as_ref().unwrap();
    let namespace = dev_env.metadata.namespace.as_ref().unwrap();

    info!("Reconciling DevEnvironment {}/{}", namespace, name);
    info!("Image: {}", dev_env.spec.image);
    info!("Replicas: {}", dev_env.spec.replicas);

    // For now, we just log and requeue
    // In the next phase, we'll create actual resources here

    Ok(Action::requeue(Duration::from_secs(300)))
}

/// What to do when reconciliation fails
pub fn error_policy(_dev_env: Arc<DevEnvironment>, error: &Error, _ctx: Arc<Context>) -> Action {
    tracing::warn!("Reconciliation error: {:?}", error);
    Action::requeue(Duration::from_secs(60))
}
