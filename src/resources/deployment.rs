use crate::errors::Result;
use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{Container, PodSpec, PodTemplateSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector;
use kube::{
    Client,
    api::{Api, PostParams},
};
use std::collections::BTreeMap;
use tracing::info;

/// Configuration for creating a deployment
pub struct DeploymentConfig {
    /// Name of the deployment
    pub name: String,
    /// Target namespace
    pub namespace: String,
    /// Container image
    pub image: String,
    /// Number of replicas
    pub replicas: i32,
    /// Environment variables for the container
    pub env_vars: Vec<(String, String)>,
    /// Labels to apply
    pub labels: Vec<(String, String)>,
}

/// Create or update a deployment (idempotent)
pub async fn ensure_deployment(client: Client, config: DeploymentConfig) -> Result<()> {
    let deployments: Api<Deployment> = Api::namespaced(client, &config.namespace);

    info!(
        "Creating deployment {} in namespace {}",
        config.name, config.namespace
    );

    // Build labels
    let mut labels = BTreeMap::new();
    labels.insert("app".to_string(), config.name.clone());
    labels.insert("managed-by".to_string(), "devenv-controller".to_string());
    for (k, v) in config.labels {
        labels.insert(k, v);
    }

    // Build the deployment
    let deployment = Deployment {
        metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
            name: Some(config.name.clone()),
            namespace: Some(config.namespace.clone()),
            labels: Some(labels.clone()),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(config.replicas),
            selector: LabelSelector {
                match_labels: Some(labels.clone()),
                ..Default::default()
            },
            template: PodTemplateSpec {
                metadata: Some(k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                    labels: Some(labels.clone()),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    containers: vec![Container {
                        name: "app".to_string(),
                        image: Some(config.image.clone()),
                        // TODO: Add env vars from config.env_vars
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    };

    // Create or update
    match deployments.get(&config.name).await {
        Ok(_) => {
            info!("Deployment exists, updating...");
            deployments
                .replace(&config.name, &PostParams::default(), &deployment)
                .await?;
        }
        Err(kube::Error::Api(err)) if err.code == 404 => {
            info!("Creating new deployment...");
            deployments
                .create(&PostParams::default(), &deployment)
                .await?;
        }
        Err(e) => return Err(e.into()),
    }

    info!("Deployment {}/{} ready", config.namespace, config.name);
    Ok(())
}

/// Delete a deployment
pub async fn delete_deployment(client: Client, name: &str, namespace: &str) -> Result<()> {
    let deployments: Api<Deployment> = Api::namespaced(client, namespace);

    match deployments
        .delete(name, &kube::api::DeleteParams::default())
        .await
    {
        Ok(_) => {
            info!("Deployment {}/{} deleted", namespace, name);
            Ok(())
        }
        Err(kube::Error::Api(err)) if err.code == 404 => {
            info!(
                "Deployment {}/{} does not exist, nothing to delete",
                namespace, name
            );
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
