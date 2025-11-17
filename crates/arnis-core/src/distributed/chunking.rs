/// Geographic chunking logic for distributed processing

use crate::coordinate_system::geographic::LLBBox;
use crate::distributed::work_unit::{WorkSettings, WorkUnit};

/// Configuration for chunking a bounding box
#[derive(Debug, Clone)]
pub struct ChunkConfig {
    /// Size of each chunk in degrees (latitude/longitude)
    pub chunk_size_degrees: f64,
    
    /// Overlap between chunks in degrees (to handle edge cases)
    pub overlap_degrees: f64,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            // Default: ~1km chunks at mid-latitudes
            chunk_size_degrees: 0.01,
            // Default: ~100m overlap
            overlap_degrees: 0.001,
        }
    }
}

/// Split a bounding box into chunks for distributed processing
pub fn split_into_chunks(
    bbox: &LLBBox,
    config: &ChunkConfig,
    settings: &WorkSettings,
) -> Vec<WorkUnit> {
    let mut chunks = Vec::new();
    
    let min_lat = bbox.min().lat();
    let min_lng = bbox.min().lng();
    let max_lat = bbox.max().lat();
    let max_lng = bbox.max().lng();
    
    let lat_range = max_lat - min_lat;
    let lng_range = max_lng - min_lng;
    
    // Calculate number of chunks in each direction
    let num_lat_chunks = (lat_range / config.chunk_size_degrees).ceil() as usize;
    let num_lng_chunks = (lng_range / config.chunk_size_degrees).ceil() as usize;
    
    // Generate chunks
    for lat_idx in 0..num_lat_chunks {
        for lng_idx in 0..num_lng_chunks {
            let chunk_min_lat = min_lat + (lat_idx as f64 * config.chunk_size_degrees);
            let chunk_min_lng = min_lng + (lng_idx as f64 * config.chunk_size_degrees);
            
            let chunk_max_lat = (chunk_min_lat + config.chunk_size_degrees + config.overlap_degrees)
                .min(max_lat);
            let chunk_max_lng = (chunk_min_lng + config.chunk_size_degrees + config.overlap_degrees)
                .min(max_lng);
            
            // Create chunk bbox with overlap
            if let Ok(chunk_bbox) = LLBBox::new(
                chunk_min_lat,
                chunk_min_lng,
                chunk_max_lat,
                chunk_max_lng,
            ) {
                let chunk_id = format!("chunk_{}_{}", lat_idx, lng_idx);
                
                chunks.push(WorkUnit {
                    chunk_id,
                    bbox: chunk_bbox,
                    settings: settings.clone(),
                });
            }
        }
    }
    
    chunks
}

/// Calculate estimated processing time for a chunk (in seconds)
pub fn estimate_chunk_time(chunk: &WorkUnit) -> f64 {
    // Simple estimation based on area and settings
    let lat_range = chunk.bbox.max().lat() - chunk.bbox.min().lat();
    let lng_range = chunk.bbox.max().lng() - chunk.bbox.min().lng();
    let area = lat_range * lng_range;
    
    // Base time: ~60 seconds per 0.01° x 0.01° chunk
    let mut time = area / (0.01 * 0.01) * 60.0;
    
    // Adjust for settings
    if chunk.settings.terrain {
        time *= 1.5; // Terrain adds 50%
    }
    if chunk.settings.interior {
        time *= 1.2; // Interiors add 20%
    }
    
    time
}

/// Get chunk statistics
pub fn get_chunk_stats(chunks: &[WorkUnit]) -> ChunkStats {
    let total_chunks = chunks.len();
    let estimated_total_time: f64 = chunks.iter().map(estimate_chunk_time).sum();
    
    ChunkStats {
        total_chunks,
        estimated_total_time,
        estimated_time_per_chunk: if total_chunks > 0 {
            estimated_total_time / total_chunks as f64
        } else {
            0.0
        },
    }
}

/// Statistics about chunks
#[derive(Debug, Clone)]
pub struct ChunkStats {
    pub total_chunks: usize,
    pub estimated_total_time: f64,
    pub estimated_time_per_chunk: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_into_chunks() {
        let bbox = LLBBox::new(40.0, -74.0, 40.1, -73.9).unwrap();
        let config = ChunkConfig {
            chunk_size_degrees: 0.05,
            overlap_degrees: 0.001,
        };
        let settings = WorkSettings::default();

        let chunks = split_into_chunks(&bbox, &config, &settings);

        // 0.1 / 0.05 = 2 chunks in each direction = 4 total
        assert_eq!(chunks.len(), 4);

        // Check chunk IDs
        assert_eq!(chunks[0].chunk_id, "chunk_0_0");
        assert_eq!(chunks[1].chunk_id, "chunk_0_1");
        assert_eq!(chunks[2].chunk_id, "chunk_1_0");
        assert_eq!(chunks[3].chunk_id, "chunk_1_1");
    }

    #[test]
    fn test_chunk_with_default_config() {
        let bbox = LLBBox::new(40.0, -74.0, 40.01, -73.99).unwrap();
        let config = ChunkConfig::default();
        let settings = WorkSettings::default();

        let chunks = split_into_chunks(&bbox, &config, &settings);

        // Should create at least 1 chunk
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_estimate_chunk_time() {
        let chunk = WorkUnit {
            chunk_id: "test".to_string(),
            bbox: LLBBox::new(40.0, -74.0, 40.01, -73.99).unwrap(),
            settings: WorkSettings::default(),
        };

        let time = estimate_chunk_time(&chunk);
        assert!(time > 0.0);
    }

    #[test]
    fn test_chunk_stats() {
        let bbox = LLBBox::new(40.0, -74.0, 40.02, -73.98).unwrap();
        let config = ChunkConfig::default();
        let settings = WorkSettings::default();

        let chunks = split_into_chunks(&bbox, &config, &settings);
        let stats = get_chunk_stats(&chunks);

        assert_eq!(stats.total_chunks, chunks.len());
        assert!(stats.estimated_total_time > 0.0);
        assert!(stats.estimated_time_per_chunk > 0.0);
    }
}
