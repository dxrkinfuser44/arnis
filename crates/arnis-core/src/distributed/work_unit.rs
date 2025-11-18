/// Work unit structures for distributed processing

use crate::coordinate_system::geographic::LLBBox;
use serde::{Deserialize, Serialize};

/// A unit of work to be processed by a worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkUnit {
    /// Unique identifier for this chunk
    pub chunk_id: String,
    
    /// Geographic bounding box for this chunk
    pub bbox: LLBBox,
    
    /// Processing settings
    pub settings: WorkSettings,
}

/// Settings for processing a work unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSettings {
    /// World scale (blocks per meter)
    pub scale: f64,
    
    /// Enable terrain generation
    pub terrain: bool,
    
    /// Enable building interiors
    pub interior: bool,
    
    /// Enable building roofs
    pub roof: bool,
    
    /// Ground level in Minecraft world
    pub ground_level: i32,
}

impl Default for WorkSettings {
    fn default() -> Self {
        Self {
            scale: 1.0,
            terrain: false,
            interior: true,
            roof: true,
            ground_level: -62,
        }
    }
}

/// Result of processing a work unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkResult {
    /// Chunk identifier
    pub chunk_id: String,
    
    /// Status of processing
    pub status: WorkStatus,
    
    /// URL or path to result file (if successful)
    pub result_location: Option<String>,
    
    /// Error message (if failed)
    pub error: Option<String>,
    
    /// Processing time in seconds
    pub processing_time: f64,
}

/// Status of work unit processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkStatus {
    /// Work not yet started
    Pending,
    
    /// Work assigned to a worker
    Assigned,
    
    /// Work in progress
    InProgress,
    
    /// Work completed successfully
    Completed,
    
    /// Work failed with error
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_unit_serialization() {
        let work_unit = WorkUnit {
            chunk_id: "chunk_0_0".to_string(),
            bbox: LLBBox::new(40.0, -74.0, 40.01, -73.99).unwrap(),
            settings: WorkSettings::default(),
        };

        let json = serde_json::to_string(&work_unit).unwrap();
        let deserialized: WorkUnit = serde_json::from_str(&json).unwrap();

        assert_eq!(work_unit.chunk_id, deserialized.chunk_id);
        assert_eq!(work_unit.bbox.min().lat(), deserialized.bbox.min().lat());
    }

    #[test]
    fn test_work_result_serialization() {
        let result = WorkResult {
            chunk_id: "chunk_0_0".to_string(),
            status: WorkStatus::Completed,
            result_location: Some("/path/to/result.mca".to_string()),
            error: None,
            processing_time: 42.5,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: WorkResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.chunk_id, deserialized.chunk_id);
        assert_eq!(result.status, deserialized.status);
        assert_eq!(result.processing_time, deserialized.processing_time);
    }
}
