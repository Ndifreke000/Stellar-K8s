//! kubectl-stellar: A kubectl plugin for managing Stellar nodes
//!
//! This plugin provides convenient commands to interact with StellarNode resources:
//! - `kubectl stellar list` - List all StellarNode resources
//! - `kubectl stellar logs <node-name>` - Get logs from pods associated with a StellarNode
//! - `kubectl stellar status [node-name]` - Get sync status of StellarNode(s)

use std::process;

use clap::{Parser, Subcommand};
use kube::{api::Api, Client, ResourceExt};
use k8s_openapi::api::core::v1::Pod;

use stellar_k8s::crd::StellarNode;
use stellar_k8s::controller::check_node_health;
use stellar_k8s::error::{Error, Result};

#[derive(Parser)]
#[command(name = "kubectl-stellar")]
#[command(about = "A kubectl plugin for managing Stellar nodes", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Kubernetes namespace (defaults to current context namespace)
    #[arg(short, long, global = true)]
    namespace: Option<String>,

    /// Output format (table, json, yaml)
    #[arg(short, long, global = true, default_value = "table")]
    output: String,
}

#[derive(Subcommand)]
enum Commands {
    /// List all StellarNode resources
    List {
        /// Show all namespaces
        #[arg(short = 'A', long)]
        all_namespaces: bool,
    },
    /// Get logs from pods associated with a StellarNode
    Logs {
        /// Name of the StellarNode
        node_name: String,
        /// Container name (if multiple containers in pod)
        #[arg(short, long)]
        container: Option<String>,
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
        /// Number of lines to show from the end of logs
        #[arg(short, long, default_value = "100")]
        tail: Option<i64>,
    },
    /// Get sync status of StellarNode(s)
    Status {
        /// Name of a specific StellarNode (optional, shows all if omitted)
        node_name: Option<String>,
        /// Show all namespaces
        #[arg(short = 'A', long)]
        all_namespaces: bool,
    },
    /// Alias for status command
    #[command(name = "sync-status")]
    SyncStatus {
        /// Name of a specific StellarNode (optional, shows all if omitted)
        node_name: Option<String>,
        /// Show all namespaces
        #[arg(short = 'A', long)]
        all_namespaces: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    let client = Client::try_default()
        .await
        .map_err(|e| Error::KubeError(e))?;

    match cli.command {
        Commands::List { all_namespaces } => {
            list_nodes(&client, all_namespaces, &cli.output).await
        }
        Commands::Logs {
            node_name,
            container,
            follow,
            tail,
        } => {
            let namespace = cli.namespace.as_deref().unwrap_or("default");
            logs(&client, namespace, &node_name, container.as_deref(), follow, tail).await
        }
        Commands::Status {
            node_name,
            all_namespaces,
        } => {
            status(&client, node_name.as_deref(), all_namespaces, cli.namespace.as_deref(), &cli.output).await
        }
        Commands::SyncStatus {
            node_name,
            all_namespaces,
        } => {
            status(&client, node_name.as_deref(), all_namespaces, cli.namespace.as_deref(), &cli.output).await
        }
    }
}

/// List all StellarNode resources
async fn list_nodes(client: &Client, all_namespaces: bool, output: &str) -> Result<()> {
    if all_namespaces {
        let api: Api<StellarNode> = Api::all(client.clone());
        let nodes = api.list(&Default::default()).await.map_err(Error::KubeError)?;

        match output {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&nodes.items)?);
            }
            "yaml" => {
                println!("{}", serde_yaml::to_string(&nodes.items).map_err(|e| Error::ConfigError(format!("YAML serialization error: {}", e)))?);
            }
            _ => {
                println!("{:<30} {:<15} {:<15} {:<10} {:<15} {:<10}", "NAME", "TYPE", "NETWORK", "REPLICAS", "PHASE", "NAMESPACE");
                println!("{}", "-".repeat(100));
                for node in nodes.items {
                    let namespace = node.namespace().unwrap_or_else(|| "default".to_string());
                    let name = node.name_any();
                    let node_type = format!("{:?}", node.spec.node_type);
                    let network = format!("{:?}", node.spec.network);
                    let replicas = node.spec.replicas;
                    let phase = node
                        .status
                        .as_ref()
                        .map(|s| s.phase.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    println!(
                        "{:<30} {:<15} {:<15} {:<10} {:<15} {:<10}",
                        name, node_type, network, replicas, phase, namespace
                    );
                }
            }
        }
    } else {
        // Use default namespace
        let namespace = "default";
        
        let api: Api<StellarNode> = Api::namespaced(client.clone(), namespace);
        let nodes = api.list(&Default::default()).await.map_err(Error::KubeError)?;

        match output {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&nodes.items)?);
            }
            "yaml" => {
                println!("{}", serde_yaml::to_string(&nodes.items).map_err(|e| Error::ConfigError(format!("YAML serialization error: {}", e)))?);
            }
            _ => {
                println!("{:<30} {:<15} {:<15} {:<10} {:<15}", "NAME", "TYPE", "NETWORK", "REPLICAS", "PHASE");
                println!("{}", "-".repeat(85));
                for node in nodes.items {
                    let name = node.name_any();
                    let node_type = format!("{:?}", node.spec.node_type);
                    let network = format!("{:?}", node.spec.network);
                    let replicas = node.spec.replicas;
                    let phase = node
                        .status
                        .as_ref()
                        .map(|s| s.phase.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    println!(
                        "{:<30} {:<15} {:<15} {:<10} {:<15}",
                        name, node_type, network, replicas, phase
                    );
                }
            }
        }
    }

    Ok(())
}

/// Get logs from pods associated with a StellarNode
async fn logs(
    client: &Client,
    namespace: &str,
    node_name: &str,
    container: Option<&str>,
    follow: bool,
    tail: Option<i64>,
) -> Result<()> {
    // First, verify the StellarNode exists
    let node_api: Api<StellarNode> = Api::namespaced(client.clone(), namespace);
    let _node = node_api
        .get(node_name)
        .await
        .map_err(Error::KubeError)?;

    // Find pods using the same label selector as the controller
    let pod_api: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let label_selector = format!(
        "app.kubernetes.io/instance={},app.kubernetes.io/name=stellar-node",
        node_name
    );

    let pods = pod_api
        .list(&kube::api::ListParams::default().labels(&label_selector))
        .await
        .map_err(Error::KubeError)?;

    if pods.items.is_empty() {
        return Err(Error::ConfigError(format!(
            "No pods found for StellarNode {}/{}",
            namespace, node_name
        )));
    }

    // Get logs from pods (if multiple pods, show logs from all)
    // For StatefulSets (Validators), there's typically one pod
    // For Deployments (Horizon/Soroban), there may be multiple pods
    if pods.items.len() > 1 && !follow {
        println!("Found {} pods, showing logs from all:", pods.items.len());
    }
    
    for (idx, pod) in pods.items.iter().enumerate() {
        let pod_name = pod.name_any();
        
        if pods.items.len() > 1 && !follow {
            println!("\n=== Pod: {} ===", pod_name);
        }
        
        // Use kubectl logs command via exec since kube-rs doesn't have a direct logs API
        // This is the standard way kubectl plugins handle logs
        let mut cmd = std::process::Command::new("kubectl");
        cmd.arg("logs");
        cmd.arg("-n").arg(namespace);
        cmd.arg(&pod_name);
        
        if let Some(container_name) = container {
            cmd.arg("-c").arg(container_name);
        }
        
        if follow {
            cmd.arg("-f");
        }
        
        if let Some(tail_lines) = tail {
            cmd.arg("--tail").arg(tail_lines.to_string());
        }

        // For follow mode, we need to spawn and wait, otherwise just execute
        if follow && idx == 0 {
            // Only follow the first pod in follow mode
            let status = cmd.status().map_err(|e| {
                Error::ConfigError(format!("Failed to execute kubectl logs: {}", e))
            })?;
            
            if !status.success() {
                return Err(Error::ConfigError(format!(
                    "kubectl logs failed with exit code: {:?}",
                    status.code()
                )));
            }
            break; // Exit after following first pod
        } else {
            let output = cmd.output().map_err(|e| {
                Error::ConfigError(format!("Failed to execute kubectl logs: {}", e))
            })?;
            
            if !output.status.success() {
                return Err(Error::ConfigError(format!(
                    "kubectl logs failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                )));
            }
            
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }

    Ok(())
}

/// Get sync status of StellarNode(s)
async fn status(
    client: &Client,
    node_name: Option<&str>,
    all_namespaces: bool,
    namespace: Option<&str>,
    output: &str,
) -> Result<()> {
    let nodes = if let Some(name) = node_name {
        // Get specific node
        let ns = namespace.unwrap_or("default");
        let api: Api<StellarNode> = Api::namespaced(client.clone(), ns);
        let node = api.get(name).await.map_err(Error::KubeError)?;
        vec![node]
    } else if all_namespaces {
        // Get all nodes across all namespaces
        let api: Api<StellarNode> = Api::all(client.clone());
        let list = api.list(&Default::default()).await.map_err(Error::KubeError)?;
        list.items
    } else {
        // Get nodes in specified or default namespace
        let ns = namespace.unwrap_or("default");
        let api: Api<StellarNode> = Api::namespaced(client.clone(), ns);
        let list = api.list(&Default::default()).await.map_err(Error::KubeError)?;
        list.items
    };

    if nodes.is_empty() {
        println!("No StellarNode resources found.");
        return Ok(());
    }

    match output {
        "json" => {
            let mut results = Vec::new();
            for node in nodes {
                let health_result = check_node_health(client, &node, None).await?;
                results.push(serde_json::json!({
                    "name": node.name_any(),
                    "namespace": node.namespace().unwrap_or_else(|| "default".to_string()),
                    "type": format!("{:?}", node.spec.node_type),
                    "network": format!("{:?}", node.spec.network),
                    "phase": node.status.as_ref().map(|s| s.phase.clone()).unwrap_or_else(|| "Unknown".to_string()),
                    "healthy": health_result.healthy,
                    "synced": health_result.synced,
                    "ledger_sequence": health_result.ledger_sequence,
                    "message": health_result.message,
                }));
            }
            println!("{}", serde_json::to_string_pretty(&results).map_err(|e| Error::ConfigError(format!("JSON serialization error: {}", e)))?);
        }
        "yaml" => {
            let mut results = Vec::new();
            for node in nodes {
                let health_result = check_node_health(client, &node, None).await?;
                results.push(serde_json::json!({
                    "name": node.name_any(),
                    "namespace": node.namespace().unwrap_or_else(|| "default".to_string()),
                    "type": format!("{:?}", node.spec.node_type),
                    "network": format!("{:?}", node.spec.network),
                    "phase": node.status.as_ref().map(|s| s.phase.clone()).unwrap_or_else(|| "Unknown".to_string()),
                    "healthy": health_result.healthy,
                    "synced": health_result.synced,
                    "ledger_sequence": health_result.ledger_sequence,
                    "message": health_result.message,
                }));
            }
            println!("{}", serde_yaml::to_string(&results).map_err(|e| Error::ConfigError(format!("YAML serialization error: {}", e)))?);
        }
        _ => {
            // Table format
            if all_namespaces || node_name.is_none() {
                println!("{:<30} {:<15} {:<15} {:<10} {:<10} {:<10} {:<15} {:<20}", 
                    "NAME", "NAMESPACE", "TYPE", "HEALTHY", "SYNCED", "LEDGER", "PHASE", "MESSAGE");
                println!("{}", "-".repeat(135));
            } else {
                println!("{:<30} {:<15} {:<10} {:<10} {:<15} {:<20}", 
                    "NAME", "TYPE", "HEALTHY", "SYNCED", "PHASE", "MESSAGE");
                println!("{}", "-".repeat(110));
            }

            for node in nodes {
                let health_result = check_node_health(client, &node, None).await?;
                let name = node.name_any();
                let node_type = format!("{:?}", node.spec.node_type);
                let phase = node.status.as_ref().map(|s| s.phase.clone()).unwrap_or_else(|| "Unknown".to_string());
                let healthy = if health_result.healthy { "Yes" } else { "No" };
                let synced = if health_result.synced { "Yes" } else { "No" };
                let ledger = health_result.ledger_sequence
                    .map(|l| l.to_string())
                    .unwrap_or_else(|| "N/A".to_string());
                let message = if health_result.message.len() > 20 {
                    format!("{}...", &health_result.message[..17])
                } else {
                    health_result.message.clone()
                };

                if all_namespaces || (node_name.is_none() && namespace.is_none()) {
                    let namespace = node.namespace().unwrap_or_else(|| "default".to_string());
                    println!("{:<30} {:<15} {:<15} {:<10} {:<10} {:<10} {:<15} {:<20}", 
                        name, namespace, node_type, healthy, synced, ledger, phase, message);
                } else {
                    println!("{:<30} {:<15} {:<10} {:<10} {:<15} {:<20}", 
                        name, node_type, healthy, synced, phase, message);
                }
            }
        }
    }

    Ok(())
}
