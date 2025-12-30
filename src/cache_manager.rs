//! Cache management for pre-caching OSM data and elevation data
//!
//! This module provides functionality to cache downloaded OSM data and elevation data
//! to disk, allowing users to pre-download data for large areas without generating
//! the world immediately. This is especially useful for lower-end systems that may
//! crash during generation but can still download and parse the data.

use crate::coordinate_system::geographic::LLBBox;
use crate::elevation_data::ElevationData;
use chrono::{DateTime, Duration, Utc};
use image::{ImageBuffer, Rgb, RgbImage};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{Read, Write};
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Metadata for a cached region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Unique identifier for this cache entry
    pub id: String,
    /// Human-readable name for the cached region
    pub name: String,
    /// Bounding box of the cached region
    pub bbox: String,
    /// Scale factor used
    pub scale: f64,
    /// Whether terrain/elevation data is included
    pub has_terrain: bool,
    /// Timestamp when cache was created
    pub created_at: DateTime<Utc>,
    /// Estimated size in bytes
    pub size_bytes: u64,
    /// Number of OSM elements
    pub element_count: usize,
    /// Whether a preview image exists
    pub has_preview: bool,
    /// Expiration timestamp (if set)
    pub expires_at: Option<DateTime<Utc>>,
}

/// Complete cache entry with all data
#[derive(Debug)]
pub struct CacheEntry {
    pub metadata: CacheMetadata,
    pub osm_data: Value,
    pub elevation_data: Option<ElevationData>,
}

/// Cache manager for handling pre-cached data
pub struct CacheManager {
    cache_dir: PathBuf,
}

impl CacheManager {
    /// Create a new cache manager with the default cache directory
    pub fn new() -> Result<Self, String> {
        let cache_dir = Self::get_default_cache_dir()?;
        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;
        Ok(Self { cache_dir })
    }

    /// Create a cache manager with a custom cache directory
    pub fn with_directory(cache_dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;
        Ok(Self { cache_dir })
    }

    /// Get the default cache directory for the platform
    fn get_default_cache_dir() -> Result<PathBuf, String> {
        let base_dir = if cfg!(target_os = "windows") {
            dirs::data_local_dir()
                .ok_or("Failed to get local data directory")?
                .join("arnis")
        } else if cfg!(target_os = "macos") {
            dirs::data_dir()
                .ok_or("Failed to get data directory")?
                .join("arnis")
        } else {
            // Linux and others
            dirs::data_dir()
                .ok_or("Failed to get data directory")?
                .join("arnis")
        };

        Ok(base_dir.join("cache"))
    }

    /// Generate a unique cache ID based on bbox and timestamp
    fn generate_cache_id(bbox: &LLBBox) -> String {
        let timestamp = Utc::now().timestamp();
        let bbox_str = format!(
            "{:.4}_{:.4}_{:.4}_{:.4}",
            bbox.min().lat(),
            bbox.min().lng(),
            bbox.max().lat(),
            bbox.max().lng()
        );
        format!("cache_{}_{}", bbox_str.replace('.', ""), timestamp)
    }

    /// Generate a simple preview image for the cached region
    fn generate_preview_image(
        osm_data: &Value,
        bbox: &LLBBox,
        cache_path: &Path,
    ) -> Result<(), String> {
        const PREVIEW_WIDTH: u32 = 400;
        const PREVIEW_HEIGHT: u32 = 300;

        let mut img: RgbImage = ImageBuffer::from_pixel(
            PREVIEW_WIDTH,
            PREVIEW_HEIGHT,
            Rgb([240, 240, 240]), // Light gray background
        );

        if let Some(elements) = osm_data["elements"].as_array() {
            // 1. Build node map for fast lookup
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

            // Calculate bbox dimensions with safety check
            let lat_range = (bbox.max().lat() - bbox.min().lat()).max(0.000001);
            let lng_range = (bbox.max().lng() - bbox.min().lng()).max(0.000001);

            // Helper to map coords to pixels
            let map_coords = |lat: f64, lon: f64| -> Option<(u32, u32)> {
                let x = ((lon - bbox.min().lng()) / lng_range * PREVIEW_WIDTH as f64) as i32;
                let y = PREVIEW_HEIGHT as i32 - ((lat - bbox.min().lat()) / lat_range * PREVIEW_HEIGHT as f64) as i32;
                
                if x >= -10 && x < (PREVIEW_WIDTH as i32 + 10) && y >= -10 && y < (PREVIEW_HEIGHT as i32 + 10) {
                    Some((x.clamp(0, PREVIEW_WIDTH as i32 - 1) as u32, y.clamp(0, PREVIEW_HEIGHT as i32 - 1) as u32))
                } else {
                    None
                }
            };

            // 2. Draw elements
            for element in elements {
                let tags = &element["tags"];
                
                // Determine color
                let color = if tags["building"].is_object() || tags["building"].is_string() {
                    Rgb([200, 100, 100]) // Red: Buildings
                } else if tags["highway"].is_object() || tags["highway"].is_string() {
                    Rgb([100, 100, 200]) // Blue: Highways
                } else if tags["natural"].is_object() || tags["natural"].is_string() || tags["landuse"].is_string() {
                    Rgb([100, 200, 100]) // Green: Nature/Landuse
                } else if tags["water"].is_string() || tags["waterway"].is_string() {
                    Rgb([100, 150, 255]) // Light Blue: Water
                } else {
                    Rgb([180, 180, 180]) // Gray: Other
                };

                // Draw Ways (Lines)
                if element["type"].as_str() == Some("way") {
                    if let Some(nodes) = element["nodes"].as_array() {
                        let mut prev_point: Option<(u32, u32)> = None;
                        
                        for node_ref in nodes {
                            if let Some(id) = node_ref.as_i64() {
                                if let Some(&(lat, lon)) = node_coords.get(&id) {
                                    if let Some(curr_point) = map_coords(lat, lon) {
                                        if let Some((x0, y0)) = prev_point {
                                            // Simple line drawing (Bresenham-like)
                                            let (x1, y1) = curr_point;
                                            // Draw point
                                            img.put_pixel(x1, y1, color);
                                            
                                            // Interpolate a few points for continuity if needed
                                            // (For preview, just drawing vertices is often enough, 
                                            // but let's do a simple midpoint to connect dots)
                                            let xm = (x0 + x1) / 2;
                                            let ym = (y0 + y1) / 2;
                                            img.put_pixel(xm, ym, color);
                                        }
                                        prev_point = Some(curr_point);
                                    }
                                }
                            }
                        }
                    }
                } 
                // Draw Nodes (Points) - only if interesting tags
                else if element["type"].as_str() == Some("node") && !tags.as_object().map_or(true, |m| m.is_empty()) {
                     if let (Some(lat), Some(lon)) = (element["lat"].as_f64(), element["lon"].as_f64()) {
                        if let Some((x, y)) = map_coords(lat, lon) {
                            img.put_pixel(x, y, color);
                            // Make it a bit thicker
                            if x + 1 < PREVIEW_WIDTH { img.put_pixel(x + 1, y, color); }
                            if y + 1 < PREVIEW_HEIGHT { img.put_pixel(x, y + 1, color); }
                        }
                     }
                }
            }
        }

        // Save preview image
        let preview_path = cache_path.join("preview.png");
        img.save(&preview_path)
            .map_err(|e| format!("Failed to save preview image: {}", e))?;

        Ok(())
    }

    /// Save a cache entry to disk with all associated data
    ///
    /// Creates a cache directory containing:
    /// - `metadata.json`: Cache metadata (name, bbox, size, etc.)
    /// - `osm_data.json`: Pretty-printed OSM JSON data
    /// - `elevation_data.bin.gz`: Compressed elevation data (if terrain enabled)
    /// - `preview.png`: Auto-generated preview image
    ///
    /// # Arguments
    /// * `bbox` - Geographic bounding box of the cached region
    /// * `scale` - World scale factor used
    /// * `osm_data` - Raw OSM JSON data from Overpass API
    /// * `elevation_data` - Optional elevation data for terrain
    /// * `area_name` - Optional human-readable name (e.g., "New York City")
    /// * `expiration_days` - Optional expiration period (default: 30 days)
    ///
    /// # Returns
    /// * `Ok(String)` - The generated cache ID
    /// * `Err(String)` - Error message if caching fails
    pub fn save_cache(
        &self,
        bbox: &LLBBox,
        scale: f64,
        osm_data: &Value,
        elevation_data: Option<&ElevationData>,
        area_name: Option<String>,
        expiration_days: Option<u32>,
    ) -> Result<String, String> {
        let cache_id = Self::generate_cache_id(bbox);
        let cache_path = self.cache_dir.join(&cache_id);

        // Create cache directory structure
        fs::create_dir_all(&cache_path)
            .map_err(|e| format!("Failed to create cache entry directory: {}", e))?;

        // Save OSM data as pretty-printed JSON for human readability
        let osm_file = cache_path.join("osm_data.json");
        let osm_json = serde_json::to_string_pretty(osm_data)
            .map_err(|e| format!("Failed to serialize OSM data: {}", e))?;
        fs::write(&osm_file, osm_json)
            .map_err(|e| format!("Failed to write OSM data: {}", e))?;

        // Save elevation data if present (with GZip compression to save ~60-70% space)
        let has_terrain = elevation_data.is_some();
        if let Some(elev_data) = elevation_data {
            let elev_file = cache_path.join("elevation_data.bin.gz");
            
            // Serialize to binary format using bincode
            let elev_bytes = bincode::serialize(elev_data)
                .map_err(|e| format!("Failed to serialize elevation data: {}", e))?;
            
            // Compress with GZip (default compression level)
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&elev_bytes)
                .map_err(|e| format!("Failed to compress elevation data: {}", e))?;
            let compressed_bytes = encoder.finish()
                .map_err(|e| format!("Failed to finish compression: {}", e))?;

            fs::write(&elev_file, compressed_bytes)
                .map_err(|e| format!("Failed to write elevation data: {}", e))?;
        }

        // Count elements for metadata
        let element_count = osm_data["elements"]
            .as_array()
            .map(|arr| arr.len())
            .unwrap_or(0);

        // Generate preview image (400x300 PNG showing OSM elements)
        let has_preview = Self::generate_preview_image(osm_data, bbox, &cache_path).is_ok();

        // Calculate total cache size on disk
        let size_bytes = Self::calculate_directory_size(&cache_path)?;

        // Calculate expiration date if specified
        let expires_at = expiration_days.map(|days| {
            Utc::now() + Duration::days(days as i64)
        });

        // Create metadata
        let metadata = CacheMetadata {
            id: cache_id.clone(),
            name: area_name.unwrap_or_else(|| format!("Region {}", cache_id)),
            bbox: format!(
                "{},{},{},{}",
                bbox.min().lat(),
                bbox.min().lng(),
                bbox.max().lat(),
                bbox.max().lng()
            ),
            scale,
            has_terrain,
            created_at: Utc::now(),
            size_bytes,
            element_count,
            has_preview,
            expires_at,
        };

        // Save metadata
        let metadata_file = cache_path.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
        fs::write(&metadata_file, metadata_json)
            .map_err(|e| format!("Failed to write metadata: {}", e))?;

        Ok(cache_id)
    }

    /// Load a cache entry from disk
    pub fn load_cache(&self, cache_id: &str) -> Result<CacheEntry, String> {
        let cache_path = self.cache_dir.join(cache_id);

        if !cache_path.exists() {
            return Err(format!("Cache entry '{}' not found", cache_id));
        }

        // Load metadata
        let metadata_file = cache_path.join("metadata.json");
        let metadata_json = fs::read_to_string(&metadata_file)
            .map_err(|e| format!("Failed to read metadata: {}", e))?;
        let metadata: CacheMetadata = serde_json::from_str(&metadata_json)
            .map_err(|e| format!("Failed to parse metadata: {}", e))?;

        // Load OSM data
        let osm_file = cache_path.join("osm_data.json");
        let osm_json = fs::read_to_string(&osm_file)
            .map_err(|e| format!("Failed to read OSM data: {}", e))?;
        let osm_data: Value = serde_json::from_str(&osm_json)
            .map_err(|e| format!("Failed to parse OSM data: {}", e))?;

        // Load elevation data if present
        let elevation_data = if metadata.has_terrain {
            let compressed_file = cache_path.join("elevation_data.bin.gz");
            let legacy_file = cache_path.join("elevation_data.bin");
            
            if compressed_file.exists() {
                let compressed_bytes = fs::read(&compressed_file)
                    .map_err(|e| format!("Failed to read elevation data: {}", e))?;
                let mut decoder = GzDecoder::new(&compressed_bytes[..]);
                let mut decoded_bytes = Vec::new();
                decoder.read_to_end(&mut decoded_bytes)
                    .map_err(|e| format!("Failed to decompress elevation data: {}", e))?;
                
                let elev_data: ElevationData = bincode::deserialize(&decoded_bytes)
                    .map_err(|e| format!("Failed to deserialize elevation data: {}", e))?;
                Some(elev_data)
            } else if legacy_file.exists() {
                // Fallback for older caches
                let elev_bytes = fs::read(&legacy_file)
                    .map_err(|e| format!("Failed to read elevation data: {}", e))?;
                let elev_data: ElevationData = bincode::deserialize(&elev_bytes)
                    .map_err(|e| format!("Failed to deserialize elevation data: {}", e))?;
                Some(elev_data)
            } else {
                None
            }
        } else {
            None
        };

        Ok(CacheEntry {
            metadata,
            osm_data,
            elevation_data,
        })
    }

    /// List all available cache entries
    pub fn list_caches(&self) -> Result<Vec<CacheMetadata>, String> {
        let mut caches = Vec::new();

        if !self.cache_dir.exists() {
            return Ok(caches);
        }

        let entries = fs::read_dir(&self.cache_dir)
            .map_err(|e| format!("Failed to read cache directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let metadata_file = path.join("metadata.json");
                if metadata_file.exists() {
                    match fs::read_to_string(&metadata_file) {
                        Ok(metadata_json) => {
                            if let Ok(metadata) = serde_json::from_str::<CacheMetadata>(&metadata_json) {
                                caches.push(metadata);
                            }
                        }
                        Err(_) => continue,
                    }
                }
            }
        }

        // Sort by creation date (newest first)
        caches.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(caches)
    }

    /// Delete a cache entry
    pub fn delete_cache(&self, cache_id: &str) -> Result<(), String> {
        let cache_path = self.cache_dir.join(cache_id);

        if !cache_path.exists() {
            return Err(format!("Cache entry '{}' not found", cache_id));
        }

        fs::remove_dir_all(&cache_path)
            .map_err(|e| format!("Failed to delete cache entry: {}", e))?;

        Ok(())
    }

    /// Clean up expired caches
    pub fn cleanup_expired_caches(&self) -> Result<usize, String> {
        let caches = self.list_caches()?;
        let now = Utc::now();
        let mut cleaned_count = 0;

        for cache in caches {
            if let Some(expires_at) = cache.expires_at {
                if now > expires_at {
                    self.delete_cache(&cache.id)?;
                    cleaned_count += 1;
                }
            }
        }

        Ok(cleaned_count)
    }

    /// Get preview image path for a cache
    pub fn get_preview_path(&self, cache_id: &str) -> Option<PathBuf> {
        let preview_path = self.cache_dir.join(cache_id).join("preview.png");
        if preview_path.exists() {
            Some(preview_path)
        } else {
            None
        }
    }

    /// Get preview image as base64 string
    pub fn get_preview_base64(&self, cache_id: &str) -> Result<Option<String>, String> {
        if let Some(preview_path) = self.get_preview_path(cache_id) {
            let image_bytes = fs::read(&preview_path)
                .map_err(|e| format!("Failed to read preview image: {}", e))?;
            let base64_image = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &image_bytes,
            );
            Ok(Some(format!("data:image/png;base64,{}", base64_image)))
        } else {
            Ok(None)
        }
    }

    /// Get the total size of all caches
    pub fn get_total_cache_size(&self) -> Result<u64, String> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }

        Self::calculate_directory_size(&self.cache_dir)
    }

    /// Clear all caches
    pub fn clear_all_caches(&self) -> Result<(), String> {
        if !self.cache_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&self.cache_dir)
            .map_err(|e| format!("Failed to read cache directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                fs::remove_dir_all(&path)
                    .map_err(|e| format!("Failed to delete cache entry: {}", e))?;
            }
        }

        Ok(())
    }

    /// Calculate the total size of a directory recursively
    fn calculate_directory_size(path: &Path) -> Result<u64, String> {
        let mut total_size = 0u64;

        if path.is_dir() {
            let entries = fs::read_dir(path)
                .map_err(|e| format!("Failed to read directory: {}", e))?;

            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    total_size += Self::calculate_directory_size(&entry_path)?;
                } else if entry_path.is_file() {
                    total_size += entry.metadata()
                        .map(|m| m.len())
                        .unwrap_or(0);
                }
            }
        }

        Ok(total_size)
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default cache manager")
    }
}

/// Format bytes into a human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cache_manager = CacheManager::with_directory(temp_dir.path().to_path_buf());
        assert!(cache_manager.is_ok());
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_cache_id_generation() {
        let bbox = LLBBox::from_str("40.0,-74.0,40.1,-73.9").unwrap();
        let id1 = CacheManager::generate_cache_id(&bbox);
        let id2 = CacheManager::generate_cache_id(&bbox);
        // IDs should be different due to different timestamps
        assert_ne!(id1, id2);
        assert!(id1.starts_with("cache_"));
    }
}
