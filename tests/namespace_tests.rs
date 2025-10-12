mod common;

use common::*;
use devenv_controller::crd::DevEnvironment;
use k8s_openapi::api::core::v1::Namespace;
use kube::{Api, Client, api::PostParams};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
#[ignore]
async fn test_creates_managed_namespace() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::try_default().await?;
    setup_test_namespace(client.clone()).await?;

    // Cleanup before test
    cleanup_devenv(client.clone(), DEVENV_AUTO).await;
    cleanup_namespace(client.clone(), TARGET_AUTO).await;

    let _guard = TestGuard::new(
        client.clone(),
        DEVENV_AUTO.to_string(),
        Some(TARGET_AUTO.to_string()),
    );

    // Create DevEnvironment (will auto-generate target namespace)
    let devenv_api: Api<DevEnvironment> = Api::namespaced(client.clone(), TEST_NAMESPACE);
    let devenv = serde_json::json!({
        "apiVersion": "devenv.example.com/v1alpha1",
        "kind": "DevEnvironment",
        "metadata": {
            "name": DEVENV_AUTO
        },
        "spec": {
            "image": "nginx:alpine",
            "replicas": 1,
            "targetNamespace": TARGET_AUTO
        }
    });

    devenv_api
        .create(&PostParams::default(), &serde_json::from_value(devenv)?)
        .await?;

    sleep(Duration::from_secs(5)).await;

    // Check namespace was created with managed label
    let ns_api: Api<Namespace> = Api::all(client.clone());
    let ns = ns_api.get(TARGET_AUTO).await?;

    let labels = ns
        .metadata
        .labels
        .as_ref()
        .expect("Namespace should have labels");
    assert_eq!(
        labels.get("devenv.example.com/managed"),
        Some(&"true".to_string()),
        "Namespace should have managed label"
    );

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_uses_existing_namespace() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::try_default().await?;
    setup_test_namespace(client.clone()).await?;

    // Cleanup before test
    cleanup_devenv(client.clone(), DEVENV_EXISTING).await;
    cleanup_namespace(client.clone(), TARGET_EXISTING).await;

    // Create existing namespace WITHOUT managed label
    create_unmanaged_namespace(client.clone(), TARGET_EXISTING).await?;

    let _guard = TestGuard::new(
        client.clone(),
        DEVENV_EXISTING.to_string(),
        Some(TARGET_EXISTING.to_string()),
    );

    let devenv_api: Api<DevEnvironment> = Api::namespaced(client.clone(), TEST_NAMESPACE);
    let devenv = serde_json::json!({
        "apiVersion": "devenv.example.com/v1alpha1",
        "kind": "DevEnvironment",
        "metadata": {
            "name": DEVENV_EXISTING
        },
        "spec": {
            "image": "nginx:alpine",
            "replicas": 1,
            "targetNamespace": TARGET_EXISTING
        }
    });

    devenv_api
        .create(&PostParams::default(), &serde_json::from_value(devenv)?)
        .await?;

    sleep(Duration::from_secs(5)).await;

    // Check namespace doesn't have managed label
    let ns_api: Api<Namespace> = Api::all(client.clone());
    let ns = ns_api.get(TARGET_EXISTING).await?;

    let has_managed_label = ns
        .metadata
        .labels
        .as_ref()
        .and_then(|labels| labels.get("devenv.example.com/managed"))
        .is_some();

    assert!(
        !has_managed_label,
        "Existing namespace should not have managed label"
    );

    Ok(())
}
