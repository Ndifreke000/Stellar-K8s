//! Shared types for Stellar node specifications
//!
//! These types are used across the CRD definitions and controller logic.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Supported Stellar node types
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub enum NodeType {
    /// Full validator node running Stellar Core
    /// Participates in consensus and validates transactions
    Validator,

    /// Horizon API server for REST access to the Stellar network
    /// Provides a RESTful API for querying the Stellar ledger
    Horizon,

    /// Soroban RPC node for smart contract interactions
    /// Handles Soroban smart contract simulation and submission
    SorobanRpc,
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Validator => write!(f, "Validator"),
            NodeType::Horizon => write!(f, "Horizon"),
            NodeType::SorobanRpc => write!(f, "SorobanRpc"),
        }
    }
}

/// Target Stellar network
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub enum StellarNetwork {
    /// Stellar public mainnet
    Mainnet,
    /// Stellar testnet for testing
    Testnet,
    /// Futurenet for bleeding-edge features
    Futurenet,
    /// Custom network with passphrase
    Custom(String),
}

impl StellarNetwork {
    /// Get the network passphrase for this network
    pub fn passphrase(&self) -> &str {
        match self {
            StellarNetwork::Mainnet => "Public Global Stellar Network ; September 2015",
            StellarNetwork::Testnet => "Test SDF Network ; September 2015",
            StellarNetwork::Futurenet => "Test SDF Future Network ; October 2022",
            StellarNetwork::Custom(passphrase) => passphrase,
        }
    }
}

/// Kubernetes-style resource requirements
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResourceRequirements {
    /// Minimum resources requested
    pub requests: ResourceSpec,
    /// Maximum resources allowed
    pub limits: ResourceSpec,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            requests: ResourceSpec {
                cpu: "500m".to_string(),
                memory: "1Gi".to_string(),
            },
            limits: ResourceSpec {
                cpu: "2".to_string(),
                memory: "4Gi".to_string(),
            },
        }
    }
}

/// Resource specification for CPU and memory
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ResourceSpec {
    /// CPU cores (e.g., "500m", "2")
    pub cpu: String,
    /// Memory (e.g., "1Gi", "4Gi")
    pub memory: String,
}

/// Storage configuration for persistent data
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StorageConfig {
    /// Storage class name (e.g., "standard", "ssd", "premium-rwo")
    pub storage_class: String,
    /// Size of the PersistentVolumeClaim (e.g., "100Gi")
    pub size: String,
    /// Retention policy when the node is deleted
    #[serde(default)]
    pub retention_policy: RetentionPolicy,
    /// Optional annotations to apply to the PersistentVolumeClaim
    /// Useful for storage-class specific parameters (e.g., volumeBindingMode)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<BTreeMap<String, String>>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            storage_class: "standard".to_string(),
            size: "100Gi".to_string(),
            retention_policy: RetentionPolicy::default(),
            annotations: None,
        }
    }
}

/// PVC retention policy on node deletion
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub enum RetentionPolicy {
    /// Delete the PVC when the node is deleted
    #[default]
    Delete,
    /// Retain the PVC for manual cleanup or data recovery
    Retain,
}

/// Validator-specific configuration
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorConfig {
    /// Secret name containing the validator seed (key: STELLAR_CORE_SEED)
    pub seed_secret_ref: String,
    /// Quorum set configuration as TOML string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quorum_set: Option<String>,
    /// Enable history archive for this validator
    #[serde(default)]
    pub enable_history_archive: bool,
    /// History archive URLs to fetch from
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history_archive_urls: Vec<String>,
    /// Node is in catchup mode (syncing historical data)
    #[serde(default)]
    pub catchup_complete: bool,
}

/// Horizon API server configuration
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HorizonConfig {
    /// Secret reference for database credentials
    pub database_secret_ref: String,
    /// Enable real-time ingestion from Stellar Core
    #[serde(default = "default_true")]
    pub enable_ingest: bool,
    /// Stellar Core URL to ingest from
    pub stellar_core_url: String,
    /// Number of parallel ingestion workers
    #[serde(default = "default_ingest_workers")]
    pub ingest_workers: u32,
    /// Enable experimental features
    #[serde(default)]
    pub enable_experimental_ingestion: bool,
}

fn default_true() -> bool {
    true
}

fn default_ingest_workers() -> u32 {
    1
}

/// Soroban RPC server configuration
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SorobanConfig {
    /// Stellar Core endpoint URL
    pub stellar_core_url: String,
    /// Captive Core configuration (TOML format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captive_core_config: Option<String>,
    /// Enable transaction simulation preflight
    #[serde(default = "default_true")]
    pub enable_preflight: bool,
    /// Maximum number of events to return per request
    #[serde(default = "default_max_events")]
    pub max_events_per_request: u32,
}

fn default_max_events() -> u32 {
    10000
}

/// Condition for status reporting (Kubernetes convention)
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    /// Type of condition (e.g., "Ready", "Progressing", "Degraded")
    #[serde(rename = "type")]
    pub type_: String,
    /// Status of the condition: "True", "False", or "Unknown"
    pub status: String,
    /// Last time the condition transitioned
    pub last_transition_time: String,
    /// Machine-readable reason for the condition
    pub reason: String,
    /// Human-readable message
    pub message: String,
}

impl Condition {
    /// Create a new Ready condition
    pub fn ready(status: bool, reason: &str, message: &str) -> Self {
        Self {
            type_: "Ready".to_string(),
            status: if status { "True" } else { "False" }.to_string(),
            last_transition_time: chrono::Utc::now().to_rfc3339(),
            reason: reason.to_string(),
            message: message.to_string(),
        }
    }

    /// Create a new Progressing condition
    pub fn progressing(reason: &str, message: &str) -> Self {
        Self {
            type_: "Progressing".to_string(),
            status: "True".to_string(),
            last_transition_time: chrono::Utc::now().to_rfc3339(),
            reason: reason.to_string(),
            message: message.to_string(),
        }
    }
}
