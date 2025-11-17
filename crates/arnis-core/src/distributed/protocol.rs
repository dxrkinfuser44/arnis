/// HTTP API protocol types for coordinator-worker communication

use crate::distributed::work_unit::{WorkResult, WorkStatus, WorkUnit};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Worker registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterWorkerRequest {
    /// Unique worker identifier
    pub worker_id: String,
    
    /// Worker capabilities
    pub capabilities: WorkerCapabilities,
}

/// Worker capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilities {
    /// Operating system (e.g., "linux", "windows", "macos")
    pub os: String,
    
    /// Number of CPU cores
    pub cpu_cores: usize,
    
    /// Available memory in GB
    pub memory_gb: usize,
}

/// Worker registration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterWorkerResponse {
    /// Registration status
    pub status: String,
    
    /// Coordinator identifier
    pub coordinator_id: String,
}

/// Work request from worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkRequest {
    /// Worker identifier
    pub worker_id: String,
}

/// Work response from coordinator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkResponse {
    /// Work unit to process (None if no work available)
    pub work_unit: Option<WorkUnit>,
    
    /// URL to download OSM data for this chunk
    pub osm_data_url: Option<String>,
}

/// Result submission from worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResultRequest {
    /// Worker identifier
    pub worker_id: String,
    
    /// Work result
    pub result: WorkResult,
}

/// Result submission response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResultResponse {
    /// Acceptance status
    pub status: String,
    
    /// Optional next work unit
    pub next_work: Option<WorkUnit>,
}

/// Overall status request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusRequest {}

/// Overall status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    /// Total number of chunks
    pub total_chunks: usize,
    
    /// Number of completed chunks
    pub completed: usize,
    
    /// Number of chunks in progress
    pub in_progress: usize,
    
    /// Number of pending chunks
    pub pending: usize,
    
    /// Number of failed chunks
    pub failed: usize,
    
    /// Worker status
    pub workers: WorkerStatusSummary,
    
    /// Per-chunk status
    pub chunk_status: HashMap<String, WorkStatus>,
}

/// Worker status summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerStatusSummary {
    /// Number of active workers
    pub active: usize,
    
    /// Number of idle workers
    pub idle: usize,
    
    /// Individual worker details
    pub workers: Vec<WorkerStatus>,
}

/// Individual worker status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerStatus {
    /// Worker identifier
    pub worker_id: String,
    
    /// Current chunk being processed (if any)
    pub current_chunk: Option<String>,
    
    /// Number of chunks completed by this worker
    pub chunks_completed: usize,
    
    /// Worker capabilities
    pub capabilities: WorkerCapabilities,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate_system::geographic::LLBBox;
    use crate::distributed::work_unit::WorkSettings;

    #[test]
    fn test_register_worker_serialization() {
        let request = RegisterWorkerRequest {
            worker_id: "worker-123".to_string(),
            capabilities: WorkerCapabilities {
                os: "linux".to_string(),
                cpu_cores: 8,
                memory_gb: 16,
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: RegisterWorkerRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.worker_id, deserialized.worker_id);
        assert_eq!(request.capabilities.cpu_cores, deserialized.capabilities.cpu_cores);
    }

    #[test]
    fn test_work_response_serialization() {
        let work_unit = WorkUnit {
            chunk_id: "chunk_0_0".to_string(),
            bbox: LLBBox::new(40.0, -74.0, 40.01, -73.99).unwrap(),
            settings: WorkSettings::default(),
        };

        let response = WorkResponse {
            work_unit: Some(work_unit),
            osm_data_url: Some("http://example.com/data.json".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: WorkResponse = serde_json::from_str(&json).unwrap();

        assert!(deserialized.work_unit.is_some());
        assert!(deserialized.osm_data_url.is_some());
    }

    #[test]
    fn test_status_response_serialization() {
        let response = StatusResponse {
            total_chunks: 100,
            completed: 45,
            in_progress: 5,
            pending: 50,
            failed: 0,
            workers: WorkerStatusSummary {
                active: 3,
                idle: 1,
                workers: vec![],
            },
            chunk_status: HashMap::new(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: StatusResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.total_chunks, deserialized.total_chunks);
        assert_eq!(response.completed, deserialized.completed);
    }
}
