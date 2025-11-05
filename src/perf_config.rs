/// Global performance configuration module
/// Manages performance settings like RAM limits, thread counts, and SIMD usage

use crate::cpu_info::{PlatformInfo, SimdCapability};
use once_cell::sync::OnceCell;
use std::sync::Mutex;

/// Maximum RAM to use (in GB) - default cap
const DEFAULT_MAX_RAM_GB: f64 = 16.0;

/// Global performance configuration
static PERF_CONFIG: OnceCell<Mutex<PerformanceConfig>> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Effective RAM limit in GB
    pub effective_ram_gb: f64,
    /// Effective thread count
    pub effective_threads: usize,
    /// SIMD capability
    pub simd_capability: SimdCapability,
    /// Platform information
    pub platform_info: PlatformInfo,
    /// User overrides (from GUI or config file)
    pub user_ram_override: Option<f64>,
    pub user_threads_override: Option<usize>,
    pub user_simd_override: Option<bool>,
}

impl PerformanceConfig {
    /// Initialize with default detection
    pub fn init_default() -> Self {
        let platform_info = PlatformInfo::detect();
        
        // Set effective RAM: min(system RAM, 16GB)
        let effective_ram_gb = platform_info.total_memory_gb.min(DEFAULT_MAX_RAM_GB);
        
        // Set effective threads to logical CPU count
        let effective_threads = platform_info.logical_cpus;
        
        // SIMD from platform detection
        let simd_capability = platform_info.simd_capability;
        
        PerformanceConfig {
            effective_ram_gb,
            effective_threads,
            simd_capability,
            platform_info,
            user_ram_override: None,
            user_threads_override: None,
            user_simd_override: None,
        }
    }
    
    /// Apply user overrides
    pub fn apply_overrides(&mut self) {
        if let Some(ram) = self.user_ram_override {
            self.effective_ram_gb = ram.min(self.platform_info.total_memory_gb);
        }
        
        if let Some(threads) = self.user_threads_override {
            self.effective_threads = threads.min(self.platform_info.logical_cpus);
        }
        
        if let Some(false) = self.user_simd_override {
            // User disabled SIMD
            self.simd_capability = SimdCapability::None;
        }
    }
    
    /// Set user RAM override (in GB)
    pub fn set_ram_override(&mut self, ram_gb: f64) {
        self.user_ram_override = Some(ram_gb);
        self.apply_overrides();
    }
    
    /// Set user thread count override
    pub fn set_threads_override(&mut self, threads: usize) {
        self.user_threads_override = Some(threads);
        self.apply_overrides();
    }
    
    /// Set user SIMD override
    pub fn set_simd_override(&mut self, enabled: bool) {
        self.user_simd_override = Some(enabled);
        self.apply_overrides();
    }
    
    /// Initialize Rayon thread pool with optimal settings for the platform
    /// On Apple Silicon, Rayon's work-stealing algorithm will naturally utilize
    /// both Performance and Efficiency cores as scheduled by macOS
    pub fn init_rayon_threadpool(&self) -> Result<(), rayon::ThreadPoolBuildError> {
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.effective_threads)
            .build_global()
    }
    
    /// Log the current configuration
    pub fn log_config(&self) {
        log::info!("=== Performance Configuration ===");
        log::info!("Architecture: {}", self.platform_info.architecture);
        log::info!("Physical CPUs: {}", self.platform_info.physical_cpus);
        log::info!("Logical CPUs: {}", self.platform_info.logical_cpus);
        log::info!("Total System RAM: {:.2} GB", self.platform_info.total_memory_gb);
        log::info!("Effective RAM Limit: {:.2} GB", self.effective_ram_gb);
        log::info!("Effective Thread Count: {}", self.effective_threads);
        log::info!("SIMD Capability: {}", self.simd_capability);
        
        // Apple Silicon specific notes
        #[cfg(all(target_arch = "aarch64", target_vendor = "apple"))]
        {
            log::info!("Apple Silicon detected: Rayon work-stealing will leverage");
            log::info!("  Performance and Efficiency cores via macOS scheduling");
        }
        
        if self.user_ram_override.is_some() {
            log::info!("  (RAM override applied)");
        }
        if self.user_threads_override.is_some() {
            log::info!("  (Thread override applied)");
        }
        if self.user_simd_override.is_some() {
            log::info!("  (SIMD override applied)");
        }
        
        log::info!("================================");
    }
}

/// Get or initialize the global performance configuration
pub fn get_or_init() -> &'static Mutex<PerformanceConfig> {
    PERF_CONFIG.get_or_init(|| Mutex::new(PerformanceConfig::init_default()))
}

/// Get a copy of the current configuration
pub fn get_config() -> PerformanceConfig {
    get_or_init().lock().unwrap().clone()
}

/// Update the global configuration
pub fn update_config<F>(updater: F)
where
    F: FnOnce(&mut PerformanceConfig),
{
    let config_mutex = get_or_init();
    let mut config = config_mutex.lock().unwrap();
    updater(&mut config);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_init_default() {
        let config = PerformanceConfig::init_default();
        
        // Verify defaults are reasonable
        assert!(config.effective_ram_gb > 0.0);
        assert!(config.effective_ram_gb <= DEFAULT_MAX_RAM_GB);
        assert!(config.effective_threads > 0);
        assert!(config.effective_threads <= config.platform_info.logical_cpus);
    }
    
    #[test]
    fn test_ram_override() {
        let mut config = PerformanceConfig::init_default();
        
        // Set override to 8GB
        config.set_ram_override(8.0);
        assert_eq!(config.effective_ram_gb, 8.0);
        assert_eq!(config.user_ram_override, Some(8.0));
    }
    
    #[test]
    fn test_ram_override_exceeds_system() {
        let mut config = PerformanceConfig::init_default();
        let system_ram = config.platform_info.total_memory_gb;
        
        // Try to set override higher than system RAM
        config.set_ram_override(system_ram + 10.0);
        
        // Should be capped at system RAM
        assert_eq!(config.effective_ram_gb, system_ram);
    }
    
    #[test]
    fn test_threads_override() {
        let mut config = PerformanceConfig::init_default();
        
        // Set override to half of logical CPUs
        let half_cpus = config.platform_info.logical_cpus / 2;
        if half_cpus > 0 {
            config.set_threads_override(half_cpus);
            assert_eq!(config.effective_threads, half_cpus);
            assert_eq!(config.user_threads_override, Some(half_cpus));
        }
    }
    
    #[test]
    fn test_threads_override_exceeds_system() {
        let mut config = PerformanceConfig::init_default();
        let logical_cpus = config.platform_info.logical_cpus;
        
        // Try to set override higher than logical CPUs
        config.set_threads_override(logical_cpus + 10);
        
        // Should be capped at logical CPUs
        assert_eq!(config.effective_threads, logical_cpus);
    }
    
    #[test]
    fn test_simd_override() {
        let mut config = PerformanceConfig::init_default();
        let original_simd = config.simd_capability;
        
        // Disable SIMD
        config.set_simd_override(false);
        assert_eq!(config.simd_capability, SimdCapability::None);
        assert_eq!(config.user_simd_override, Some(false));
        
        // Re-enable should restore original if we re-init
        // (In practice, the override just controls whether to use detected SIMD)
        let mut config2 = PerformanceConfig::init_default();
        config2.set_simd_override(true);
        assert_eq!(config2.simd_capability, original_simd);
    }
}
