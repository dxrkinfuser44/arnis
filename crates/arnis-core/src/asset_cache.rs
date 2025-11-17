/// Asset cache module for pre-downloading and managing OSM and elevation data
///
/// This module provides functionality to:
/// - Download OSM data and elevation data separately from processing
/// - Cache downloaded assets with validation
/// - Process from cached assets without re-downloading
/// - Verify cache integrity

use crate::coordinate_system::geographic::LLBBox;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Metadata for cached assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Bounding box of the cached area
    pub bbox: LLBBox,
    /// Download timestamp
    pub timestamp: u64,
    /// OSM data file path (relative to cache dir)
    pub osm_data_file: String,
    /// Elevation data file path if available (relative to cache dir)
    pub elevation_data_file: Option<String>,
    /// Checksum of OSM data for validation
    pub osm_checksum: String,
    /// Size of cached OSM data in bytes
    pub osm_data_size: u64,
    /// Download method used
    pub download_method: String,
}

/// Asset cache manager
pub struct AssetCache {
    /// Base directory for cache storage
    cache_dir: PathBuf,
}

impl AssetCache {
    /// Create a new asset cache with specified directory
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> std::io::Result<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        fs::create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    /// Get the default cache directory
    /// Uses OS-specific cache locations:
    /// - Linux: ~/.cache/arnis
    /// - macOS: ~/Library/Caches/arnis
    /// - Windows: %LOCALAPPDATA%\arnis\cache
    pub fn default_cache_dir() -> PathBuf {
        if let Some(cache_base) = dirs::cache_dir() {
            cache_base.join("arnis")
        } else {
            // Fallback to current directory
            PathBuf::from(".arnis_cache")
        }
    }

    /// Create asset cache with default directory
    pub fn default() -> std::io::Result<Self> {
        Self::new(Self::default_cache_dir())
    }

    /// Save OSM data to cache
    pub fn save_osm_data(
        &self,
        bbox: LLBBox,
        data: &str,
        download_method: &str,
    ) -> std::io::Result<CacheMetadata> {
        // Generate cache key from bounding box
        let cache_key = Self::generate_cache_key(&bbox);
        let cache_subdir = self.cache_dir.join(&cache_key);
        fs::create_dir_all(&cache_subdir)?;

        // Save OSM data
        let osm_file = cache_subdir.join("osm_data.json");
        let mut file = BufWriter::new(File::create(&osm_file)?);
        file.write_all(data.as_bytes())?;
        file.flush()?;

        // Calculate checksum
        let checksum = Self::calculate_checksum(data);

        // Get file size
        let metadata = fs::metadata(&osm_file)?;
        let size = metadata.len();

        // Get timestamp
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create metadata
        let cache_metadata = CacheMetadata {
            bbox,
            timestamp,
            osm_data_file: "osm_data.json".to_string(),
            elevation_data_file: None,
            osm_checksum: checksum,
            osm_data_size: size,
            download_method: download_method.to_string(),
        };

        // Save metadata
        self.save_metadata(&cache_key, &cache_metadata)?;

        Ok(cache_metadata)
    }

    /// Load OSM data from cache
    pub fn load_osm_data(&self, bbox: &LLBBox) -> std::io::Result<String> {
        let cache_key = Self::generate_cache_key(bbox);
        let cache_subdir = self.cache_dir.join(&cache_key);
        let osm_file = cache_subdir.join("osm_data.json");

        if !osm_file.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Cache not found for this bounding box",
            ));
        }

        // Load data
        let data = fs::read_to_string(&osm_file)?;

        // Verify cache integrity
        let metadata = self.load_metadata(&cache_key)?;
        let checksum = Self::calculate_checksum(&data);
        if checksum != metadata.osm_checksum {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Cache integrity check failed - checksum mismatch",
            ));
        }

        Ok(data)
    }

    /// Check if cache exists for a bounding box
    pub fn has_cache(&self, bbox: &LLBBox) -> bool {
        let cache_key = Self::generate_cache_key(bbox);
        let cache_subdir = self.cache_dir.join(&cache_key);
        let osm_file = cache_subdir.join("osm_data.json");
        let metadata_file = cache_subdir.join("metadata.json");

        osm_file.exists() && metadata_file.exists()
    }

    /// Get cache metadata
    pub fn get_metadata(&self, bbox: &LLBBox) -> std::io::Result<CacheMetadata> {
        let cache_key = Self::generate_cache_key(bbox);
        self.load_metadata(&cache_key)
    }

    /// List all cached areas
    pub fn list_cached_areas(&self) -> std::io::Result<Vec<CacheMetadata>> {
        let mut cached_areas = Vec::new();

        if !self.cache_dir.exists() {
            return Ok(cached_areas);
        }

        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let metadata_file = path.join("metadata.json");
                if metadata_file.exists() {
                    if let Ok(metadata) = self.load_metadata_from_file(&metadata_file) {
                        cached_areas.push(metadata);
                    }
                }
            }
        }

        Ok(cached_areas)
    }

    /// Clear cache for a specific bounding box
    pub fn clear_cache(&self, bbox: &LLBBox) -> std::io::Result<()> {
        let cache_key = Self::generate_cache_key(bbox);
        let cache_subdir = self.cache_dir.join(&cache_key);

        if cache_subdir.exists() {
            fs::remove_dir_all(&cache_subdir)?;
        }

        Ok(())
    }

    /// Clear all cached data
    pub fn clear_all(&self) -> std::io::Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
            fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    /// Get total cache size in bytes
    pub fn get_cache_size(&self) -> std::io::Result<u64> {
        let mut total_size = 0u64;

        if !self.cache_dir.exists() {
            return Ok(0);
        }

        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                total_size += Self::get_dir_size(&path)?;
            }
        }

        Ok(total_size)
    }

    // Private helper methods

    /// Generate a cache key from bounding box
    fn generate_cache_key(bbox: &LLBBox) -> String {
        // Create a unique key from bbox coordinates
        // Round to 6 decimal places to avoid floating point issues
        format!(
            "{:.6}_{:.6}_{:.6}_{:.6}",
            bbox.min().lat(),
            bbox.min().lng(),
            bbox.max().lat(),
            bbox.max().lng()
        )
        .replace(['.', '-'], "_")
    }

    /// Calculate simple checksum for data validation
    fn calculate_checksum(data: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Save metadata to file
    fn save_metadata(&self, cache_key: &str, metadata: &CacheMetadata) -> std::io::Result<()> {
        let cache_subdir = self.cache_dir.join(cache_key);
        let metadata_file = cache_subdir.join("metadata.json");

        let file = BufWriter::new(File::create(&metadata_file)?);
        serde_json::to_writer_pretty(file, metadata)?;

        Ok(())
    }

    /// Load metadata from cache key
    fn load_metadata(&self, cache_key: &str) -> std::io::Result<CacheMetadata> {
        let cache_subdir = self.cache_dir.join(cache_key);
        let metadata_file = cache_subdir.join("metadata.json");
        self.load_metadata_from_file(&metadata_file)
    }

    /// Load metadata from file path
    fn load_metadata_from_file(&self, metadata_file: &Path) -> std::io::Result<CacheMetadata> {
        let file = BufReader::new(File::open(metadata_file)?);
        let metadata: CacheMetadata = serde_json::from_reader(file)?;
        Ok(metadata)
    }

    /// Get directory size recursively
    fn get_dir_size(path: &Path) -> std::io::Result<u64> {
        let mut size = 0u64;

        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let entry_path = entry.path();

                if entry_path.is_dir() {
                    size += Self::get_dir_size(&entry_path)?;
                } else {
                    let metadata = fs::metadata(&entry_path)?;
                    size += metadata.len();
                }
            }
        } else {
            let metadata = fs::metadata(path)?;
            size += metadata.len();
        }

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_creation() {
        let temp_dir = TempDir::new().unwrap();
        let _cache = AssetCache::new(temp_dir.path()).unwrap();
        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_save_and_load_osm_data() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AssetCache::new(temp_dir.path()).unwrap();

        let bbox = LLBBox::new(40.7128, -74.0060, 40.7589, -73.9350).unwrap();

        let test_data = r#"{"elements": []}"#;

        // Save data
        let metadata = cache.save_osm_data(bbox, test_data, "test").unwrap();
        assert_eq!(metadata.bbox.min().lat(), 40.7128);
        assert_eq!(metadata.download_method, "test");

        // Check cache exists
        assert!(cache.has_cache(&bbox));

        // Load data
        let loaded_data = cache.load_osm_data(&bbox).unwrap();
        assert_eq!(loaded_data, test_data);
    }

    #[test]
    fn test_cache_integrity() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AssetCache::new(temp_dir.path()).unwrap();

        let bbox = LLBBox::new(40.0, -74.0, 41.0, -73.0).unwrap();

        let test_data = r#"{"test": "data"}"#;

        // Save data
        cache.save_osm_data(bbox, test_data, "test").unwrap();

        // Corrupt the data
        let cache_key = AssetCache::generate_cache_key(&bbox);
        let osm_file = temp_dir.path().join(&cache_key).join("osm_data.json");
        fs::write(&osm_file, "corrupted data").unwrap();

        // Try to load - should fail integrity check
        let result = cache.load_osm_data(&bbox);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_cached_areas() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AssetCache::new(temp_dir.path()).unwrap();

        let bbox1 = LLBBox::new(40.0, -74.0, 41.0, -73.0).unwrap();
        let bbox2 = LLBBox::new(50.0, -84.0, 51.0, -83.0).unwrap();

        cache.save_osm_data(bbox1, "{}", "test").unwrap();
        cache.save_osm_data(bbox2, "{}", "test").unwrap();

        let cached_areas = cache.list_cached_areas().unwrap();
        assert_eq!(cached_areas.len(), 2);
    }

    #[test]
    fn test_clear_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AssetCache::new(temp_dir.path()).unwrap();

        let bbox = LLBBox::new(40.0, -74.0, 41.0, -73.0).unwrap();

        cache.save_osm_data(bbox, "{}", "test").unwrap();
        assert!(cache.has_cache(&bbox));

        cache.clear_cache(&bbox).unwrap();
        assert!(!cache.has_cache(&bbox));
    }
}
