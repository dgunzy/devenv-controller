use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The spec for a DevEnvironment - what the user wants
#[derive(CustomResource, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[kube(
    group = "devenv.example.com",
    version = "v1alpha1",
    kind = "DevEnvironment",
    plural = "devenvironments",
    namespaced,
    status = "DevEnvironmentStatus"
)]
pub struct DevEnvironmentSpec {
    /// Container image to deploy
    pub image: String,

    /// Number of replicas
    #[serde(default = "default_replicas")]
    pub replicas: i32,

    /// Optional database type
    pub database: Option<String>,
}

fn default_replicas() -> i32 {
    1
}

/// The status of a DevEnvironment - what actually exists
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DevEnvironmentStatus {
    /// Current phase (Pending, Creating, Ready, Failed)
    pub phase: Option<String>,

    /// Human-readable message about current state
    pub message: Option<String>,

    /// URLs where the environment can be accessed
    pub endpoints: Option<Vec<String>>,
}
