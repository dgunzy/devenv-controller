use crate::errors::Result;
use k8s_openapi::api::core::v1::Namespace;
use kube::{
    Client,
    api::{Api, DeleteParams, ObjectMeta, PostParams},
};
use tracing::{info, warn};

/// Label key that marks namespaces as managed by this controller
const MANAGED_LABEL_KEY: &str = "devenv.example.com/managed";
const MANAGED_LABEL_VALUE: &str = "true";

/// Namespaces that should never be used for dev environments
const BLOCKED_NAMESPACES: &[&str] = &[
    "flux-system",
    "kube-system",
    "cert-manager",
    "ingress-nginx",
    "default",
];

/// Check if a namespace name is blocked
fn is_blocked(name: &str) -> bool {
    BLOCKED_NAMESPACES.contains(&name)
}

/// Check if a namespace has the managed label
fn is_managed(namespace: &Namespace) -> bool {
    namespace
        .metadata
        .labels
        .as_ref()
        .and_then(|labels| labels.get(MANAGED_LABEL_KEY))
        .map(|value| value == MANAGED_LABEL_VALUE)
        .unwrap_or(false)
}

/// Create a namespace if it doesn't exist (idempotent)
/// Returns an error if the namespace is blocked
/// Marks created namespaces with a management label
pub async fn ensure_namespace(client: Client, name: &str) -> Result<()> {
    // Check if namespace is blocked
    if is_blocked(name) {
        return Err(crate::errors::Error::ReconcileError(format!(
            "Namespace '{}' is blocked for dev environments",
            name
        )));
    }

    let namespaces: Api<Namespace> = Api::all(client);

    match namespaces.get(name).await {
        Ok(existing_ns) => {
            if is_managed(&existing_ns) {
                info!(
                    "Namespace {} already exists and is managed by controller",
                    name
                );
            } else {
                info!(
                    "Namespace {} already exists (user-provided, will not be deleted)",
                    name
                );
            }
            Ok(())
        }
        Err(kube::Error::Api(err)) if err.code == 404 => {
            info!("Creating namespace {} with management label", name);

            let namespace = Namespace {
                metadata: ObjectMeta {
                    name: Some(name.to_string()),
                    labels: Some(
                        [
                            ("managed-by".to_string(), "devenv-controller".to_string()),
                            (
                                MANAGED_LABEL_KEY.to_string(),
                                MANAGED_LABEL_VALUE.to_string(),
                            ),
                        ]
                        .iter()
                        .cloned()
                        .collect(),
                    ),
                    ..Default::default()
                },
                ..Default::default()
            };

            namespaces
                .create(&PostParams::default(), &namespace)
                .await?;
            info!(
                "Namespace {} created successfully with management label",
                name
            );
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

/// Delete a namespace ONLY if it has the management label
/// This prevents deletion of user-provided namespaces
pub async fn delete_namespace(client: Client, name: &str) -> Result<()> {
    // Check if namespace is in blocked list
    if is_blocked(name) {
        return Err(crate::errors::Error::ReconcileError(format!(
            "Namespace '{}' is blocked and cannot be deleted",
            name
        )));
    }

    let namespaces: Api<Namespace> = Api::all(client);

    // First, get the namespace to check if it's managed
    match namespaces.get(name).await {
        Ok(ns) => {
            if !is_managed(&ns) {
                warn!(
                    "Namespace {} is not managed by controller, skipping deletion",
                    name
                );
                return Ok(());
            }

            info!("Namespace {} is managed, proceeding with deletion", name);
            match namespaces.delete(name, &DeleteParams::default()).await {
                Ok(_) => {
                    info!("Namespace {} deleted successfully", name);
                    Ok(())
                }
                Err(e) => Err(e.into()),
            }
        }
        Err(kube::Error::Api(err)) if err.code == 404 => {
            info!("Namespace {} does not exist, nothing to delete", name);
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
