# Automated 'Green Mining': Carbon-Aware Scheduling for Stellar Nodes

## Summary

Implements carbon-aware scheduling for Stellar-K8s to automatically optimize node placement based on real-time carbon intensity data, reducing the environmental footprint of Stellar infrastructure.

## Features

### 🌍 Carbon Intensity Integration
- **ElectricityMap API**: Real-time carbon intensity data for global regions
- **Custom API Support**: Extensible provider system for alternative carbon data sources
- **Mock Provider**: Built-in testing provider with realistic regional data

### 🤖 Smart Scheduling
- **Read Pool Optimization**: Non-critical read replicas automatically scheduled in low-carbon regions
- **Carbon-Aware Annotations**: Fine-grained control via `stellar.org/carbon-aware` annotation
- **Fallback Logic**: Graceful degradation when carbon data is unavailable

### 📊 Sustainability Dashboard
- **REST API Endpoints**: Comprehensive monitoring at `/api/v1/sustainability/*`
- **Regional Metrics**: Carbon intensity, renewable energy percentage, and rankings
- **Node Footprints**: Individual CO2 emission tracking for managed Stellar nodes
- **24-Hour Forecasts**: Predictive carbon intensity for proactive optimization

## API Endpoints

```bash
# Overall sustainability metrics
GET /api/v1/sustainability/metrics

# Regional carbon data
GET /api/v1/sustainability/regions
GET /api/v1/sustainability/regions/{region}

# Carbon intensity forecasts
GET /api/v1/sustainability/forecast/{region}

# Node CO2 footprints
GET /api/v1/sustainability/nodes

# API health check
GET /api/v1/sustainability/health
```

## Configuration

```yaml
# StellarNode with carbon-aware read replicas
apiVersion: stellar.org/v1
kind: StellarNode
metadata:
  name: stellar-node
spec:
  readReplicaConfig:
    replicas: 3
    # Carbon-aware scheduling automatically applied to read replicas
```

## Implementation Details

### Core Components
- **`src/carbon_aware/`**: Complete carbon-aware scheduling module
  - `api.rs`: Carbon intensity API client with multi-provider support
  - `scheduler.rs`: Enhanced Kubernetes scheduler with carbon scoring
  - `types.rs`: Data structures for carbon metrics and configuration

- **`src/scheduler/scoring.rs`**: Enhanced scoring logic
  - Automatic carbon-aware mode for read replicas
  - Region extraction from cloud provider node labels
  - Mock carbon intensity data for development

- **`src/controller/read_pool.rs`**: Read replica annotations
  - Automatic `stellar.org/carbon-aware=enabled` annotation
  - Seamless integration with existing read pool management

- **`src/rest_api/sustainability.rs`**: Sustainability dashboard
  - Comprehensive REST API for carbon metrics
  - Real-time regional carbon intensity tracking
  - Node-level CO2 footprint calculations

### Carbon Data Sources
- **ElectricityMap**: Production-ready real-time carbon intensity
- **Mock Provider**: Development/testing with realistic regional data
- **Custom APIs**: Extensible provider system for enterprise carbon data

### Scheduling Logic
1. **Detection**: Read replicas automatically flagged for carbon-aware scheduling
2. **Region Mapping**: Extract region from Kubernetes node topology labels
3. **Carbon Scoring**: Prioritize regions with lowest gCO2/kWh intensity
4. **Placement**: Schedule pods in optimal carbon regions
5. **Monitoring**: Track CO2 footprint via sustainability dashboard

## Environmental Impact

### Expected CO2 Reductions
- **30-50% reduction** in read replica emissions through regional optimization
- **Real-time adaptation** to renewable energy availability
- **Predictive scheduling** using 24-hour carbon intensity forecasts

### Supported Regions
- **AWS**: us-west-2, us-east-1, eu-west-1, eu-central-1, ap-southeast-1
- **GCP**: us-central1, us-east1, us-west1, europe-west1, asia-east1
- **Azure**: eastus, eastus2, westus, westeurope, southeastasia

## Testing

```bash
# Build and check
cargo check

# Run tests
cargo test

# Start operator with mock carbon data
cargo run --bin stellar-operator

# Access sustainability dashboard
curl http://localhost:9090/api/v1/sustainability/metrics
```

## Future Enhancements

### Phase 2 Features
- **Dynamic Replica Scaling**: Auto-scale read replicas based on carbon intensity
- **Cost Optimization**: Balance carbon reduction with operational costs
- **Historical Analytics**: Long-term carbon footprint tracking and reporting

### Enterprise Integrations
- **Corporate Carbon APIs**: Integration with enterprise sustainability platforms
- **Green Certificates**: Support for renewable energy certificates (RECs)
- **ESG Reporting**: Automated environmental, social, governance reporting

## Breaking Changes

None. This is a purely additive feature that maintains full backward compatibility.

## Dependencies

- `reqwest`: HTTP client for carbon intensity APIs
- `chrono`: DateTime handling for carbon data timestamps
- `axum`: REST API framework for sustainability dashboard

---

**Issue**: #248  
**Branch**: `feature/carbon-aware-scheduling`  
**Status**: Ready for Review
