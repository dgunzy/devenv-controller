#![allow(dead_code)]

use devenv_controller::crd::DevEnvironment;
use k8s_openapi::api::core::v1::Namespace;
use kube::{Api, Client, api::PostParams};
use std::time::Duration;
use tokio::time::sleep;

// Test namespace constants - all tests use this single namespace
pub const TEST_NAMESPACE: &str = "devenv-integration-tests";

// Test resource names - clear and predictable
pub const DEVENV_AUTO: &str = "test-auto";
pub const DEVENV_EXISTING: &str = "test-existing";
pub const DEVENV_DEPLOY: &str = "test-deploy";
pub const DEVENV_DELETE: &str = "test-delete";

// Target namespace names (where deployments actually go)
pub const TARGET_AUTO: &str = "test-target-auto";
pub const TARGET_EXISTING: &str = "test-target-existing";
pub const TARGET_DEPLOY: &str = "test-target-deploy";
pub const TARGET_DELETE: &str = "test-target-delete";

/// Ensure the test namespace exists
pub async fn setup_test_namespace(client: Client) -> Result<(), Box<dyn std::error::Error>> {
    let namespaces: Api<Namespace> = Api::all(client);

    match namespaces.get(TEST_NAMESPACE).await {
        Ok(_) => Ok(()),
        Err(_) => {
            let ns = serde_json::json!({
                "apiVersion": "v1",
                "kind": "Namespace",
                "metadata": {
                    "name": TEST_NAMESPACE,
                    "labels": {
                        "test": "true"
                    }
                }
            });
            namespaces
                .create(&PostParams::default(), &serde_json::from_value(ns)?)
                .await?;
            Ok(())
        }
    }
}

/// Create a namespace without managed label (for testing existing namespaces)
pub async fn create_unmanaged_namespace(
    client: Client,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let namespaces: Api<Namespace> = Api::all(client);

    match namespaces.get(name).await {
        Ok(_) => Ok(()),
        Err(_) => {
            let ns = serde_json::json!({
                "apiVersion": "v1",
                "kind": "Namespace",
                "metadata": {
                    "name": name
                }
            });
            namespaces
                .create(&PostParams::default(), &serde_json::from_value(ns)?)
                .await?;
            Ok(())
        }
    }
}

/// Helper to cleanup DevEnvironment resources (idempotent)
pub async fn cleanup_devenv(client: Client, name: &str) {
    let api: Api<DevEnvironment> = Api::namespaced(client, TEST_NAMESPACE);
    match api.delete(name, &kube::api::DeleteParams::default()).await {
        Ok(_) => {
            // Wait for deletion to complete
            for _ in 0..20 {
                if api.get(name).await.is_err() {
                    break;
                }
                sleep(Duration::from_millis(500)).await;
            }
        }
        Err(_) => {} // Already deleted, that's fine
    }
}

/// Helper to cleanup namespaces (idempotent)
pub async fn cleanup_namespace(client: Client, name: &str) {
    let api: Api<Namespace> = Api::all(client);
    match api.delete(name, &kube::api::DeleteParams::default()).await {
        Ok(_) => {
            // Wait for deletion to complete (namespaces can take a while)
            for _ in 0..60 {
                match api.get(name).await {
                    Err(_) => break,
                    Ok(ns) if ns.metadata.deletion_timestamp.is_some() => {
                        // Still deleting, keep waiting
                    }
                    _ => break,
                }
                sleep(Duration::from_millis(500)).await;
            }
        }
        Err(_) => {} // Already deleted, that's fine
    }
}

/// Test guard that ensures cleanup on drop (defer-like pattern)
pub struct TestGuard {
    client: Client,
    devenv_name: String,
    target_namespace: Option<String>,
}

impl TestGuard {
    pub fn new(client: Client, devenv_name: String, target_namespace: Option<String>) -> Self {
        Self {
            client,
            devenv_name,
            target_namespace,
        }
    }
}

impl Drop for TestGuard {
    fn drop(&mut self) {
        // Spawn cleanup in background (Drop can't be async)
        let client = self.client.clone();
        let devenv_name = self.devenv_name.clone();
        let target_namespace = self.target_namespace.clone();

        tokio::spawn(async move {
            cleanup_devenv(client.clone(), &devenv_name).await;
            if let Some(ns) = target_namespace {
                cleanup_namespace(client, &ns).await;
            }
        });
    }
}
