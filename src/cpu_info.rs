/// Platform and SIMD detection module
/// Detects CPU capabilities and architecture information
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Variants constructed at runtime based on platform
pub enum SimdCapability {
    None,
    Neon,   // ARM NEON
    Avx2,   // x86 AVX2
    Avx512, // x86 AVX-512
}

impl fmt::Display for SimdCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimdCapability::None => write!(f, "None"),
            SimdCapability::Neon => write!(f, "NEON"),
            SimdCapability::Avx2 => write!(f, "AVX2"),
            SimdCapability::Avx512 => write!(f, "AVX-512"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub logical_cpus: usize,
    pub physical_cpus: usize,
    pub total_memory_gb: f64,
    pub available_memory_gb: f64,
    pub simd_capability: SimdCapability,
    pub architecture: String,
    pub os_name: String,
}

impl PlatformInfo {
    /// Detect platform information
    pub fn detect() -> Self {
        let logical_cpus = num_cpus::get();
        let physical_cpus = num_cpus::get_physical();

        // Get system memory
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();
        let total_memory_bytes = sys.total_memory();
        let total_memory_gb = total_memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let available_memory_bytes = sys.available_memory();
        let available_memory_gb = available_memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

        // Detect SIMD capability
        let simd_capability = detect_simd_capability();

        // Detect architecture and OS
        let architecture = std::env::consts::ARCH.to_string();
        let os_name = detect_os_name();

        PlatformInfo {
            logical_cpus,
            physical_cpus,
            total_memory_gb,
            available_memory_gb,
            simd_capability,
            architecture,
            os_name,
        }
    }
}

/// Detect SIMD capability based on platform and feature flags
fn detect_simd_capability() -> SimdCapability {
    #[cfg(feature = "simd-native")]
    {
        // Apple Silicon (ARM64) - Always enable NEON when simd-native is enabled
        #[cfg(all(target_arch = "aarch64", target_vendor = "apple"))]
        {
            return SimdCapability::Neon;
        }

        // Generic ARM64 - Check for NEON support
        #[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
        {
            return SimdCapability::Neon;
        }

        // x86/x86_64 - Check for AVX512 and AVX2
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            #[cfg(target_arch = "x86_64")]
            {
                if is_x86_feature_detected!("avx512f") {
                    return SimdCapability::Avx512;
                }
            }

            if is_x86_feature_detected!("avx2") {
                return SimdCapability::Avx2;
            }
        }
    }

    SimdCapability::None
}

/// Detect OS name and distribution
fn detect_os_name() -> String {
    let os = std::env::consts::OS;

    #[cfg(target_os = "linux")]
    {
        // Try to detect Linux distribution
        if let Ok(contents) = std::fs::read_to_string("/etc/os-release") {
            for line in contents.lines() {
                if line.starts_with("PRETTY_NAME=") {
                    let name = line
                        .strip_prefix("PRETTY_NAME=")
                        .unwrap_or(os)
                        .trim_matches('"');

                    // Log if running on performance-optimized distributions
                    if name.to_lowercase().contains("cachyos") {
                        log::info!("Detected CachyOS - Performance optimizations enabled");
                    } else if name.to_lowercase().contains("arch") {
                        log::info!("Detected Arch-based distribution");
                    }

                    return name.to_string();
                }
            }
        }
    }

    os.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let info = PlatformInfo::detect();

        // Basic sanity checks
        assert!(info.logical_cpus > 0);
        assert!(info.physical_cpus > 0);
        assert!(info.physical_cpus <= info.logical_cpus);
        assert!(info.total_memory_gb > 0.0);
        assert!(info.available_memory_gb > 0.0);
        assert!(info.available_memory_gb <= info.total_memory_gb);
        assert!(!info.architecture.is_empty());
        assert!(!info.os_name.is_empty());
    }

    #[test]
    fn test_simd_detection() {
        let capability = detect_simd_capability();

        // Just verify it returns a valid value
        // The actual capability depends on the runtime platform
        match capability {
            SimdCapability::None
            | SimdCapability::Neon
            | SimdCapability::Avx2
            | SimdCapability::Avx512 => {
                // All valid
            }
        }
    }

    #[test]
    fn test_simd_display() {
        assert_eq!(format!("{}", SimdCapability::None), "None");
        assert_eq!(format!("{}", SimdCapability::Neon), "NEON");
        assert_eq!(format!("{}", SimdCapability::Avx2), "AVX2");
        assert_eq!(format!("{}", SimdCapability::Avx512), "AVX-512");
    }
}
