use crate::crd::DevEnvironment;
use crate::errors::{Error, Result};
use crate::resources::{deployment, namespace};
use kube::runtime::controller::Action;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, instrument};

/// Context passed to the reconciler
pub struct Context {
    /// Kubernetes client
    pub client: kube::Client,
}

/// Main reconciliation function
#[instrument(skip(ctx, dev_env), fields(name = %dev_env.metadata.name.as_ref().unwrap(), namespace = %dev_env.metadata.namespace.as_ref().unwrap()))]
pub async fn reconcile(dev_env: Arc<DevEnvironment>, ctx: Arc<Context>) -> Result<Action> {
    let name = dev_env.metadata.name.as_ref().unwrap();

    info!("Reconciling DevEnvironment");
    debug!("Full spec: {:?}", dev_env.spec);

    // Determine target namespace - use spec value or generate from name
    let target_namespace = dev_env
        .spec
        .target_namespace
        .as_ref()
        .map(|s| s.clone())
        .unwrap_or_else(|| format!("devenv-{}", name));

    // Step 1: Create the namespace (or verify existing one)
    namespace::ensure_namespace(ctx.client.clone(), &target_namespace).await?;
    info!("Namespace {} is ready", target_namespace);

    // Step 2: Create the deployment
    let deployment_config = deployment::DeploymentConfig {
        name: name.clone(),
        namespace: target_namespace.clone(),
        image: dev_env.spec.image.clone(),
        replicas: dev_env.spec.replicas,
        env_vars: vec![],
        labels: vec![],
    };

    deployment::ensure_deployment(ctx.client.clone(), deployment_config).await?;
    info!("Deployment {} is ready", name);

    Ok(Action::requeue(Duration::from_secs(300)))
}

/// What to do when reconciliation fails
pub fn error_policy(_dev_env: Arc<DevEnvironment>, error: &Error, _ctx: Arc<Context>) -> Action {
    tracing::warn!("Reconciliation error: {:?}", error);
    Action::requeue(Duration::from_secs(60))
}
