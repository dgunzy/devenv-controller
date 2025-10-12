use devenv_controller::{crd::DevEnvironment, errors::Result, reconciler::Context};

use futures::StreamExt;
use kube::{
    api::Api,
    client::Client,
    runtime::{controller::Controller, watcher},
};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("devenv_controller=info,kube=info")
        .init();

    info!("Starting DevEnvironment Controller");

    // Create Kubernetes client
    let client = Client::try_default().await?;
    info!("Connected to Kubernetes cluster");

    // Create context that will be passed to reconciler
    let context = Arc::new(Context {
        client: client.clone(),
    });

    // Set up the controller
    // This watches all DevEnvironment resources across all namespaces
    let dev_envs = Api::<DevEnvironment>::all(client);

    Controller::new(dev_envs, watcher::Config::default())
        .run(
            devenv_controller::reconciler::reconcile,
            devenv_controller::reconciler::error_policy,
            context,
        )
        .for_each(|res| async move {
            match res {
                Ok(o) => info!("Reconciled: {:?}", o),
                Err(e) => tracing::warn!("Reconciliation error: {:?}", e),
            }
        })
        .await;

    Ok(())
}
