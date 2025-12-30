//! Chunked generation module for handling large bounding boxes
//!
//! This module provides functionality to split large generation areas into smaller chunks
//! that can be processed sequentially to avoid memory issues and crashes on lower-end systems.

use crate::args::Args;
use crate::coordinate_system::cartesian::XZBBox;
use crate::coordinate_system::geographic::LLBBox;
use crate::coordinate_system::transformation::CoordTransformer;
use crate::data_processing::{self, GenerationOptions};
use crate::ground::Ground;
use crate::map_transformation;
use crate::osm_parser::{self, ProcessedElement};
use crate::progress::emit_gui_progress_update;
use colored::Colorize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Maximum safe area for generation without chunking (in square meters)
/// Approximately 2km x 2km = 4,000,000 m²
const MAX_SAFE_AREA_M2: f64 = 4_000_000.0;

/// Maximum chunk size (approximately 1km x 1km)
const MAX_CHUNK_SIZE_M2: f64 = 1_000_000.0;

/// Overlap between chunks in meters (to ensure seamless boundaries)
const CHUNK_OVERLAP_M: f64 = 50.0;

/// Represents a single chunk of a larger generation area
#[derive(Debug, Clone)]
pub struct GenerationChunk {
    /// Chunk identifier (e.g., "chunk_0_0", "chunk_0_1")
    pub id: String,
    /// Geographic bounding box for this chunk
    pub bbox: LLBBox,
    /// Row index in the chunk grid
    pub row: usize,
    /// Column index in the chunk grid
    pub col: usize,
    /// Total number of rows in the grid
    pub total_rows: usize,
    /// Total number of columns in the grid
    pub total_cols: usize,
}

/// Configuration for chunked generation
#[derive(Debug, Clone)]
pub struct ChunkedGenerationConfig {
    /// Enable chunked generation
    pub enabled: bool,
    /// Chunk size in square meters
    pub chunk_size_m2: f64,
    /// Overlap between chunks in meters
    pub overlap_m: f64,
    /// Maximum number of chunks (safety limit)
    pub max_chunks: usize,
}

impl Default for ChunkedGenerationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            chunk_size_m2: MAX_CHUNK_SIZE_M2,
            overlap_m: CHUNK_OVERLAP_M,
            max_chunks: 100,
        }
    }
}

/// Calculate the area of a bounding box in square meters
/// Uses an approximate Haversine-based calculation that accounts for Earth's curvature.
///
/// # Arguments
/// * `bbox` - Geographic bounding box (latitude/longitude)
///
/// # Returns
/// Area in square meters (m²)
#[inline]
pub fn calculate_bbox_area_m2(bbox: &LLBBox) -> f64 {
    // Earth's mean radius in meters
    const EARTH_RADIUS_M: f64 = 6_371_000.0;

    // Calculate the differences in latitude and longitude
    let lat_diff = bbox.max().lat() - bbox.min().lat();
    let lng_diff = bbox.max().lng() - bbox.min().lng();

    // Use center latitude for more accurate width calculation
    let lat_center = (bbox.min().lat() + bbox.max().lat()) / 2.0;

    // Convert degrees to radians for trigonometric calculations
    let lat_diff_rad = lat_diff.to_radians();
    let lng_diff_rad = lng_diff.to_radians();
    let lat_center_rad = lat_center.to_radians();

    // Calculate distances in meters
    // Height: latitude difference * Earth's radius
    let height_m = lat_diff_rad * EARTH_RADIUS_M;
    // Width: longitude difference * Earth's radius * cosine(latitude)
    // Cosine accounts for meridian convergence towards poles
    let width_m = lng_diff_rad * EARTH_RADIUS_M * lat_center_rad.cos();

    // Return rectangular approximation (good enough for small areas <100 km)
    height_m * width_m
}

/// Check if a bounding box needs chunked generation
///
/// Chunked generation is recommended for areas larger than 4 km² to prevent
/// memory overflow on low-end systems.
///
/// # Arguments
/// * `bbox` - Geographic bounding box to check
/// * `config` - Chunked generation configuration
///
/// # Returns
/// `true` if the area should be split into chunks, `false` otherwise
#[inline]
pub fn needs_chunking(bbox: &LLBBox, config: &ChunkedGenerationConfig) -> bool {
    if !config.enabled {
        return false;
    }

    let area = calculate_bbox_area_m2(bbox);
    area > MAX_SAFE_AREA_M2
}

/// Split a bounding box into manageable chunks for sequential processing
///
/// This function divides large areas into a grid of smaller chunks that can be
/// processed one at a time, preventing memory overflow on low-end systems.
///
/// # Arguments
/// * `bbox` - Geographic bounding box to split
/// * `config` - Chunked generation configuration (chunk size, overlap, limits)
///
/// # Returns
/// * `Ok(Vec<GenerationChunk>)` - Vector of chunks (single chunk if no splitting needed)
/// * `Err(String)` - Error if area would require too many chunks
///
/// # Algorithm
/// 1. Calculate total area in m²
/// 2. Determine grid dimensions (N×N chunks)
/// 3. Split bbox into overlapping sub-regions
/// 4. Add overlap to prevent gaps at chunk boundaries
pub fn create_chunks(bbox: &LLBBox, config: &ChunkedGenerationConfig) -> Result<Vec<GenerationChunk>, String> {
    let area = calculate_bbox_area_m2(bbox);

    // No chunking needed for small areas
    if area <= MAX_SAFE_AREA_M2 {
        return Ok(vec![GenerationChunk {
            id: "chunk_0_0".to_string(),
            bbox: *bbox,
            row: 0,
            col: 0,
            total_rows: 1,
            total_cols: 1,
        }]);
    }

    // Calculate grid dimensions: create an N×N grid where N² ≈ area/chunk_size
    let num_chunks = (area / config.chunk_size_m2).ceil() as usize;
    let chunks_per_side = (num_chunks as f64).sqrt().ceil() as usize;

    // Safety check: prevent creating too many chunks
    if chunks_per_side * chunks_per_side > config.max_chunks {
        return Err(format!(
            "Area too large: would require {} chunks (max: {}). Please select a smaller area.",
            chunks_per_side * chunks_per_side,
            config.max_chunks
        ));
    }

    // Pre-allocate chunk vector with exact capacity
    let mut chunks = Vec::with_capacity(chunks_per_side * chunks_per_side);

    let lat_range = bbox.max().lat() - bbox.min().lat();
    let lng_range = bbox.max().lng() - bbox.min().lng();

    let lat_chunk_size = lat_range / chunks_per_side as f64;
    let lng_chunk_size = lng_range / chunks_per_side as f64;

    // Calculate overlap in degrees (approximate)
    let lat_overlap_deg = config.overlap_m / 111_000.0; // ~111km per degree latitude
    let lat_center = (bbox.min().lat() + bbox.max().lat()) / 2.0;
    let lng_overlap_deg = config.overlap_m / (111_000.0 * lat_center.to_radians().cos());

    for row in 0..chunks_per_side {
        for col in 0..chunks_per_side {
            let min_lat = bbox.min().lat() + (row as f64 * lat_chunk_size) - lat_overlap_deg;
            let max_lat = bbox.min().lat() + ((row + 1) as f64 * lat_chunk_size) + lat_overlap_deg;
            let min_lng = bbox.min().lng() + (col as f64 * lng_chunk_size) - lng_overlap_deg;
            let max_lng = bbox.min().lng() + ((col + 1) as f64 * lng_chunk_size) + lng_overlap_deg;

            // Clamp to original bbox bounds
            let min_lat = min_lat.max(bbox.min().lat());
            let max_lat = max_lat.min(bbox.max().lat());
            let min_lng = min_lng.max(bbox.min().lng());
            let max_lng = max_lng.min(bbox.max().lng());

            let chunk_bbox = LLBBox::from_str(&format!("{},{},{},{}", min_lat, min_lng, max_lat, max_lng))
                .map_err(|e| format!("Failed to create chunk bbox: {}", e))?;

            chunks.push(GenerationChunk {
                id: format!("chunk_{}_{}", row, col),
                bbox: chunk_bbox,
                row,
                col,
                total_rows: chunks_per_side,
                total_cols: chunks_per_side,
            });
        }
    }

    println!(
        "{}",
        format!(
            "Large area detected: splitting into {} chunks ({}x{} grid)",
            chunks.len(),
            chunks_per_side,
            chunks_per_side
        )
        .yellow()
        .bold()
    );

    Ok(chunks)
}

/// Generate world using chunked approach for large areas
pub fn generate_world_chunked(
    chunks: Vec<GenerationChunk>,
    raw_data: Value,
    scale: f64,
    ground: &Ground,
    args: &Args,
    options: GenerationOptions,
) -> Result<PathBuf, String> {
    let total_chunks = chunks.len();

    if total_chunks == 1 {
        // No chunking needed, use regular generation
        let bbox = chunks[0].bbox;
        let (mut parsed_elements, mut xzbbox) = osm_parser::parse_osm_data(
            raw_data,
            bbox,
            scale,
            args.debug,
        );

        parsed_elements.sort_by_key(|element| osm_parser::get_priority(element));

        let mut ground_copy = ground.clone();
        map_transformation::transform_map(&mut parsed_elements, &mut xzbbox, &mut ground_copy);

        return data_processing::generate_world_with_options(
            parsed_elements,
            xzbbox,
            bbox,
            ground_copy,
            args,
            options,
        );
    }

    println!(
        "{}",
        format!("Starting chunked generation: {} chunks", total_chunks)
            .green()
            .bold()
    );

    // Process each chunk sequentially
    for (chunk_idx, chunk) in chunks.iter().enumerate() {
        let chunk_progress = (chunk_idx as f64 / total_chunks as f64) * 100.0;

        println!(
            "\n{} [{}/{}] Processing {} ({}x{} grid, area ~{:.2} km²)...",
            format!("[Chunk {}]", chunk_idx + 1).cyan().bold(),
            chunk_idx + 1,
            total_chunks,
            chunk.id.bright_white(),
            chunk.row + 1,
            chunk.col + 1,
            calculate_bbox_area_m2(&chunk.bbox) / 1_000_000.0
        );

        emit_gui_progress_update(
            chunk_progress,
            &format!("Processing chunk {}/{}", chunk_idx + 1, total_chunks),
        );

        // Filter elements for this chunk
        let chunk_elements = filter_elements_for_chunk(&raw_data, &chunk.bbox)?;

        if chunk_elements.is_empty() {
            println!("  {} No elements in this chunk, skipping...", "⚠".yellow());
            continue;
        }

        println!(
            "  {} {} elements in chunk",
            "✓".green(),
            chunk_elements.len()
        );

        // Parse and process chunk
        let (mut parsed_elements, mut xzbbox) = osm_parser::parse_osm_data(
            chunk_elements,
            chunk.bbox,
            scale,
            args.debug,
        );

        parsed_elements.sort_by_key(|element| osm_parser::get_priority(element));

        let mut ground_copy = ground.clone();
        map_transformation::transform_map(&mut parsed_elements, &mut xzbbox, &mut ground_copy);

        // Generate this chunk's portion of the world
        // Note: All chunks write to the same world, just different regions
        let chunk_result = data_processing::generate_world_with_options(
            parsed_elements,
            xzbbox.clone(),
            chunk.bbox,
            ground_copy,
            args,
            options.clone(),
        );

        match chunk_result {
            Ok(_) => {
                println!(
                    "  {} Chunk {} completed successfully",
                    "✓".green().bold(),
                    chunk.id.bright_white()
                );
            }
            Err(e) => {
                eprintln!(
                    "  {} Chunk {} failed: {}",
                    "✗".red().bold(),
                    chunk.id.bright_white(),
                    e.red()
                );
                return Err(format!("Chunk {} failed: {}", chunk.id, e));
            }
        }
    }

    println!(
        "\n{} All {} chunks processed successfully!",
        "✓".green().bold(),
        total_chunks
    );

    emit_gui_progress_update(100.0, "Chunked generation complete");

    Ok(options.path)
}

/// Filter OSM elements to only include those within the chunk's bbox
fn filter_elements_for_chunk(raw_data: &Value, chunk_bbox: &LLBBox) -> Result<Value, String> {
    let elements = raw_data["elements"]
        .as_array()
        .ok_or("No elements array in OSM data")?;

    // 1. Index all nodes
    let mut node_coords: HashMap<i64, (f64, f64)> = HashMap::new();
    for element in elements {
        if element["type"].as_str() == Some("node") {
            if let (Some(id), Some(lat), Some(lon)) = (
                element["id"].as_i64(),
                element["lat"].as_f64(),
                element["lon"].as_f64(),
            ) {
                node_coords.insert(id, (lat, lon));
            }
        }
    }

    let mut relevant_nodes: HashSet<i64> = HashSet::new();
    let mut relevant_ways: HashSet<i64> = HashSet::new();
    let mut relevant_relations: HashSet<i64> = HashSet::new();

    // Helper to check if a point is in bbox
    let is_in_bbox = |lat: f64, lon: f64| -> bool {
        lat >= chunk_bbox.min().lat()
            && lat <= chunk_bbox.max().lat()
            && lon >= chunk_bbox.min().lng()
            && lon <= chunk_bbox.max().lng()
    };

    // Helper to check if a segment intersects bbox
    let segment_intersects_bbox = |p1: (f64, f64), p2: (f64, f64)| -> bool {
        let (lat1, lon1) = p1;
        let (lat2, lon2) = p2;
        let min_lat = chunk_bbox.min().lat();
        let max_lat = chunk_bbox.max().lat();
        let min_lon = chunk_bbox.min().lng();
        let max_lon = chunk_bbox.max().lng();

        if lat1.max(lat2) < min_lat || lat1.min(lat2) > max_lat ||
           lon1.max(lon2) < min_lon || lon1.min(lon2) > max_lon {
            return false;
        }
        
        // Check intersection with lat boundaries
        if (lat1 < min_lat && lat2 > min_lat) || (lat1 > min_lat && lat2 < min_lat) {
            let t = (min_lat - lat1) / (lat2 - lat1);
            let lon = lon1 + t * (lon2 - lon1);
            if lon >= min_lon && lon <= max_lon { return true; }
        }
        if (lat1 < max_lat && lat2 > max_lat) || (lat1 > max_lat && lat2 < max_lat) {
            let t = (max_lat - lat1) / (lat2 - lat1);
            let lon = lon1 + t * (lon2 - lon1);
            if lon >= min_lon && lon <= max_lon { return true; }
        }
        
        // Check intersection with lon boundaries
        if (lon1 < min_lon && lon2 > min_lon) || (lon1 > min_lon && lon2 < min_lon) {
            let t = (min_lon - lon1) / (lon2 - lon1);
            let lat = lat1 + t * (lat2 - lat1);
            if lat >= min_lat && lat <= max_lat { return true; }
        }
        if (lon1 < max_lon && lon2 > max_lon) || (lon1 > max_lon && lon2 < max_lon) {
            let t = (max_lon - lon1) / (lon2 - lon1);
            let lat = lat1 + t * (lat2 - lat1);
            if lat >= min_lat && lat <= max_lat { return true; }
        }
        
        false
    };

    // 2. Identify relevant ways and their nodes
    for element in elements {
        if element["type"].as_str() == Some("way") {
            if let Some(nodes) = element["nodes"].as_array() {
                let mut way_in_bbox = false;
                let mut way_node_ids = Vec::new();
                let mut way_coords = Vec::new();

                for node_val in nodes {
                    if let Some(node_id) = node_val.as_i64() {
                        way_node_ids.push(node_id);
                        if let Some(&(lat, lon)) = node_coords.get(&node_id) {
                            way_coords.push((lat, lon));
                            if is_in_bbox(lat, lon) {
                                way_in_bbox = true;
                            }
                        }
                    }
                }

                if !way_in_bbox && way_coords.len() > 1 {
                    for i in 0..way_coords.len()-1 {
                        if segment_intersects_bbox(way_coords[i], way_coords[i+1]) {
                            way_in_bbox = true;
                            break;
                        }
                    }
                }

                if way_in_bbox {
                    if let Some(id) = element["id"].as_i64() {
                        relevant_ways.insert(id);
                        // Include ALL nodes of this way, even if outside bbox
                        for node_id in way_node_ids {
                            relevant_nodes.insert(node_id);
                        }
                    }
                }
            }
        }
    }

    // 3. Identify relevant standalone nodes (e.g. trees, benches)
    // We also want to include nodes that are strictly inside the bbox, even if not part of a way
    for (id, (lat, lon)) in &node_coords {
        if is_in_bbox(*lat, *lon) {
            relevant_nodes.insert(*id);
        }
    }

    // 4. Identify relevant relations
    // A relation is relevant if it contains a relevant node or way
    for element in elements {
        if element["type"].as_str() == Some("relation") {
            if let Some(members) = element["members"].as_array() {
                let mut relation_in_bbox = false;
                for member in members {
                    let type_str = member["type"].as_str();
                    let ref_id = member["ref"].as_i64();

                    if let (Some(t), Some(r)) = (type_str, ref_id) {
                        if t == "node" && relevant_nodes.contains(&r) {
                            relation_in_bbox = true;
                            break;
                        } else if t == "way" && relevant_ways.contains(&r) {
                            relation_in_bbox = true;
                            break;
                        }
                    }
                }

                if relation_in_bbox {
                    if let Some(id) = element["id"].as_i64() {
                        relevant_relations.insert(id);
                    }
                }
            }
        }
    }

    // 5. Construct filtered list
    let mut filtered_elements = Vec::new();
    for element in elements {
        let type_str = element["type"].as_str();
        let id_opt = element["id"].as_i64();

        if let (Some(t), Some(id)) = (type_str, id_opt) {
            let include = match t {
                "node" => relevant_nodes.contains(&id),
                "way" => relevant_ways.contains(&id),
                "relation" => relevant_relations.contains(&id),
                _ => false,
            };

            if include {
                filtered_elements.push(element.clone());
            }
        }
    }

    // Create new JSON with filtered elements
    let mut filtered_data = raw_data.clone();
    filtered_data["elements"] = serde_json::json!(filtered_elements);

    Ok(filtered_data)
}

/// Estimate memory usage for a generation area
pub fn estimate_memory_usage_mb(bbox: &LLBBox, element_count: usize) -> f64 {
    let area_km2 = calculate_bbox_area_m2(bbox) / 1_000_000.0;

    // Rough estimates based on typical usage patterns
    let base_memory_mb = 100.0; // Base overhead
    let per_element_mb = 0.005; // ~5KB per element
    let per_km2_mb = 50.0; // Terrain and world data

    base_memory_mb + (element_count as f64 * per_element_mb) + (area_km2 * per_km2_mb)
}

/// Provide generation recommendations based on area size
pub fn get_generation_recommendations(bbox: &LLBBox, element_count: usize) -> Vec<String> {
    let area_m2 = calculate_bbox_area_m2(bbox);
    let area_km2 = area_m2 / 1_000_000.0;
    let estimated_memory_mb = estimate_memory_usage_mb(bbox, element_count);

    let mut recommendations = Vec::new();

    if area_m2 > MAX_SAFE_AREA_M2 {
        recommendations.push(format!(
            "Large area detected ({:.2} km²). Chunked generation will be used automatically.",
            area_km2
        ));
    }

    if estimated_memory_mb > 4000.0 {
        recommendations.push(
            "High memory usage expected (>4GB). Consider using --cache-only first.".to_string(),
        );
        recommendations.push(
            "Close other applications to free up memory during generation.".to_string(),
        );
    }

    if element_count > 100_000 {
        recommendations.push(format!(
            "Large number of elements ({}). Generation may take significant time.",
            element_count
        ));
    }

    if area_m2 > MAX_SAFE_AREA_M2 * 10.0 {
        recommendations.push(
            "Very large area. Consider splitting into multiple smaller generations.".to_string(),
        );
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_bbox_area() {
        // Small area (~1km x 1km)
        let bbox = LLBBox::from_str("40.7,-74.0,40.71,-73.99").unwrap();
        let area = calculate_bbox_area_m2(&bbox);
        assert!(area > 500_000.0 && area < 1_500_000.0);
    }

    #[test]
    fn test_needs_chunking() {
        let config = ChunkedGenerationConfig::default();

        // Small area - should not need chunking
        let small_bbox = LLBBox::from_str("40.7,- 74.0,40.71,-73.99").unwrap();
        assert!(!needs_chunking(&small_bbox, &config));

        // Large area - should need chunking
        let large_bbox = LLBBox::from_str("40.7,-74.0,40.8,-73.8").unwrap();
        assert!(needs_chunking(&large_bbox, &config));
    }

    #[test]
    fn test_create_chunks() {
        let config = ChunkedGenerationConfig::default();

        // Small area - should return 1 chunk
        let small_bbox = LLBBox::from_str("40.7,-74.0,40.71,-73.99").unwrap();
        let chunks = create_chunks(&small_bbox, &config).unwrap();
        assert_eq!(chunks.len(), 1);

        // Large area - should return multiple chunks
        let large_bbox = LLBBox::from_str("40.7,-74.0,40.85,-73.85").unwrap();
        let chunks = create_chunks(&large_bbox, &config).unwrap();
        assert!(chunks.len() > 1);
    }

    #[test]
    fn test_memory_estimation() {
        let bbox = LLBBox::from_str("40.7,-74.0,40.75,-73.95").unwrap();
        let memory = estimate_memory_usage_mb(&bbox, 10_000);
        assert!(memory > 100.0); // Should be at least 100MB
    }
}
