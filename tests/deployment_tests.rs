mod common;

use common::*;
use devenv_controller::crd::DevEnvironment;
use k8s_openapi::api::apps::v1::Deployment;
use kube::{Api, Client, api::PostParams};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
#[ignore]
async fn test_creates_deployment() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::try_default().await?;
    setup_test_namespace(client.clone()).await?;

    // Cleanup before test
    cleanup_devenv(client.clone(), DEVENV_DEPLOY).await;
    cleanup_namespace(client.clone(), TARGET_DEPLOY).await;

    let _guard = TestGuard::new(
        client.clone(),
        DEVENV_DEPLOY.to_string(),
        Some(TARGET_DEPLOY.to_string()),
    );

    // Create DevEnvironment
    let devenv_api: Api<DevEnvironment> = Api::namespaced(client.clone(), TEST_NAMESPACE);
    let devenv = serde_json::json!({
        "apiVersion": "devenv.example.com/v1alpha1",
        "kind": "DevEnvironment",
        "metadata": {
            "name": DEVENV_DEPLOY
        },
        "spec": {
            "image": "nginx:alpine",
            "replicas": 2,
            "targetNamespace": TARGET_DEPLOY
        }
    });

    devenv_api
        .create(&PostParams::default(), &serde_json::from_value(devenv)?)
        .await?;

    // Wait for reconciliation
    sleep(Duration::from_secs(10)).await;

    // Check deployment was created
    let deploy_api: Api<Deployment> = Api::namespaced(client.clone(), TARGET_DEPLOY);
    let deployment = deploy_api.get(DEVENV_DEPLOY).await?;

    assert_eq!(
        deployment.spec.as_ref().and_then(|s| s.replicas),
        Some(2),
        "Deployment should have 2 replicas"
    );

    let labels = deployment
        .metadata
        .labels
        .as_ref()
        .expect("Deployment should have labels");
    assert_eq!(
        labels.get("managed-by"),
        Some(&"devenv-controller".to_string())
    );

    println!("✓ Deployment created successfully");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_deployment_persists_after_devenv_deletion() -> Result<(), Box<dyn std::error::Error>>
{
    let client = Client::try_default().await?;
    setup_test_namespace(client.clone()).await?;

    // Cleanup before test
    cleanup_devenv(client.clone(), DEVENV_DELETE).await;
    cleanup_namespace(client.clone(), TARGET_DELETE).await;

    let _guard = TestGuard::new(
        client.clone(),
        DEVENV_DELETE.to_string(),
        Some(TARGET_DELETE.to_string()),
    );

    // Create DevEnvironment
    let devenv_api: Api<DevEnvironment> = Api::namespaced(client.clone(), TEST_NAMESPACE);
    let devenv = serde_json::json!({
        "apiVersion": "devenv.example.com/v1alpha1",
        "kind": "DevEnvironment",
        "metadata": {
            "name": DEVENV_DELETE
        },
        "spec": {
            "image": "nginx:alpine",
            "replicas": 1,
            "targetNamespace": TARGET_DELETE
        }
    });

    devenv_api
        .create(&PostParams::default(), &serde_json::from_value(devenv)?)
        .await?;

    // Wait for creation
    sleep(Duration::from_secs(10)).await;

    // Verify deployment exists
    let deploy_api: Api<Deployment> = Api::namespaced(client.clone(), TARGET_DELETE);
    let deployment_exists = deploy_api.get(DEVENV_DELETE).await.is_ok();
    assert!(
        deployment_exists,
        "Deployment should exist before DevEnvironment deletion"
    );

    println!("✓ Deployment exists");

    // Delete the DevEnvironment
    devenv_api
        .delete(DEVENV_DELETE, &kube::api::DeleteParams::default())
        .await?;

    sleep(Duration::from_secs(5)).await;

    // Verify DevEnvironment is deleted
    let devenv_exists = devenv_api.get(DEVENV_DELETE).await.is_ok();
    assert!(!devenv_exists, "DevEnvironment should be deleted");

    println!("✓ DevEnvironment deleted");

    // For now, deployment and namespace persist (no finalizer/cleanup logic yet)
    // This test documents current behavior
    let deployment_still_exists = deploy_api.get(DEVENV_DELETE).await.is_ok();
    let ns_api: Api<k8s_openapi::api::core::v1::Namespace> = Api::all(client.clone());
    let namespace_still_exists = ns_api.get(TARGET_DELETE).await.is_ok();

    println!("✓ Deployment persists: {}", deployment_still_exists);
    println!("✓ Namespace persists: {}", namespace_still_exists);

    // This is expected behavior until we implement finalizers in Phase 4
    assert!(
        deployment_still_exists && namespace_still_exists,
        "Currently, resources persist after DevEnvironment deletion (finalizers not yet implemented)"
    );

    println!("✓ Test passed - resources persist as expected (no cleanup yet)");

    Ok(())
}
