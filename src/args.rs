use crate::coordinate_system::geographic::LLBBox;
use crate::logger::LogLevel;
use clap::Parser;
use std::path::PathBuf;
use std::time::Duration;

/// Command-line arguments parser
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// Bounding box of the area (min_lat,min_lng,max_lat,max_lng) (required)
    #[arg(long, allow_hyphen_values = true, value_parser = LLBBox::from_str)]
    pub bbox: LLBBox,

    /// JSON file containing OSM data (optional)
    #[arg(long, group = "location")]
    pub file: Option<String>,

    /// JSON file to save OSM data to (optional)
    #[arg(long, group = "location")]
    pub save_json_file: Option<String>,

    /// Path to the Minecraft world (required)
    #[arg(long, value_parser = validate_minecraft_world_path)]
    pub path: PathBuf,

    /// Downloader method (requests/curl/wget) (optional)
    #[arg(long, default_value = "requests")]
    pub downloader: String,

    /// World scale to use, in blocks per meter
    #[arg(long, default_value_t = 1.0)]
    pub scale: f64,

    /// Ground level to use in the Minecraft world
    #[arg(long, default_value_t = -62)]
    pub ground_level: i32,

    /// Enable terrain (optional)
    #[arg(long)]
    pub terrain: bool,

    /// Enable interior generation (optional)
    #[arg(long, default_value_t = true)]
    pub interior: bool,

    /// Enable roof generation (optional)
    #[arg(long, default_value_t = true)]
    pub roof: bool,

    /// Enable filling ground (optional)
    #[arg(long, default_value_t = false)]
    pub fillground: bool,

    /// Enable city boundary ground generation (optional)
    /// When enabled, detects building clusters and places stone ground in urban areas.
    /// Isolated buildings in rural areas will keep grass around them.
    #[arg(long, default_value_t = true)]
    pub city_boundaries: bool,

    /// Enable debug mode (optional)
    #[arg(long)]
    pub debug: bool,

    /// Override the log level (error|warn|info|debug|trace)
    #[arg(long, value_parser = parse_log_level)]
    pub log_level: Option<LogLevel>,

    /// Disable timestamps in log output
    #[arg(long)]
    pub no_log_timestamps: bool,

    /// Disable colored log output
    #[arg(long)]
    pub no_log_colors: bool,

    /// Maximum number of tiles to prefetch during terrain download
    #[arg(long, default_value_t = 8)]
    pub tile_prefetch: usize,

    /// Set floodfill timeout (seconds) (optional)
    #[arg(long, value_parser = parse_duration)]
    pub timeout: Option<Duration>,
}

fn validate_minecraft_world_path(path: &str) -> Result<PathBuf, String> {
    let mc_world_path = PathBuf::from(path);
    if !mc_world_path.exists() {
        return Err(format!("Path does not exist: {path}"));
    }
    if !mc_world_path.is_dir() {
        return Err(format!("Path is not a directory: {path}"));
    }
    let region = mc_world_path.join("region");
    if !region.is_dir() {
        return Err(format!("No Minecraft world found at {region:?}"));
    }
    Ok(mc_world_path)
}

fn parse_duration(arg: &str) -> Result<std::time::Duration, std::num::ParseIntError> {
    let seconds = arg.parse()?;
    Ok(std::time::Duration::from_secs(seconds))
}

fn parse_log_level(value: &str) -> Result<LogLevel, String> {
    match value.to_ascii_lowercase().as_str() {
        "error" => Ok(LogLevel::Error),
        "warn" | "warning" => Ok(LogLevel::Warning),
        "info" => Ok(LogLevel::Info),
        "debug" => Ok(LogLevel::Debug),
        "trace" => Ok(LogLevel::Trace),
        other => Err(format!(
            "Invalid log level '{other}'. Use one of: error, warn, info, debug, trace"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minecraft_tmpdir() -> tempfile::TempDir {
        let tmpdir = tempfile::tempdir().unwrap();
        // create a `region` directory in the tempdir
        let region_path = tmpdir.path().join("region");
        std::fs::create_dir(&region_path).unwrap();
        tmpdir
    }
    #[test]
    fn test_flags() {
        let tmpdir = minecraft_tmpdir();
        let tmp_path = tmpdir.path().to_str().unwrap();

        // Test that terrain/debug are SetTrue
        let cmd = [
            "arnis",
            "--path",
            tmp_path,
            "--bbox",
            "1,2,3,4",
            "--terrain",
            "--debug",
        ];
        let args = Args::parse_from(cmd.iter());
        assert!(args.debug);
        assert!(args.terrain);

        let cmd = ["arnis", "--path", tmp_path, "--bbox", "1,2,3,4"];
        let args = Args::parse_from(cmd.iter());
        assert!(!args.debug);
        assert!(!args.terrain);
    }

    #[test]
    fn test_required_options() {
        let tmpdir = minecraft_tmpdir();
        let tmp_path = tmpdir.path().to_str().unwrap();

        let cmd = ["arnis"];
        assert!(Args::try_parse_from(cmd.iter()).is_err());

        let cmd = ["arnis", "--path", tmp_path, "--bbox", "1,2,3,4"];
        assert!(Args::try_parse_from(cmd.iter()).is_ok());

        let cmd = ["arnis", "--path", tmp_path, "--file", ""];
        assert!(Args::try_parse_from(cmd.iter()).is_err());

        // The --gui flag isn't used here, ugh. TODO clean up main.rs and its argparse usage.
        // let cmd = ["arnis", "--gui"];
        // assert!(Args::try_parse_from(cmd.iter()).is_ok());
    }

    #[test]
    fn parses_log_level_flag() {
        let tmpdir = minecraft_tmpdir();
        let tmp_path = tmpdir.path().to_str().unwrap();

        let cmd = [
            "arnis",
            "--path",
            tmp_path,
            "--bbox",
            "1,2,3,4",
            "--log-level",
            "trace",
        ];
        let args = Args::parse_from(cmd.iter());
        assert_eq!(args.log_level, Some(LogLevel::Trace));
    }

    #[test]
    fn parses_tile_prefetch_flag() {
        let tmpdir = minecraft_tmpdir();
        let tmp_path = tmpdir.path().to_str().unwrap();

        let cmd = [
            "arnis",
            "--path",
            tmp_path,
            "--bbox",
            "1,2,3,4",
            "--tile-prefetch",
            "16",
        ];
        let args = Args::parse_from(cmd.iter());
        assert_eq!(args.tile_prefetch, 16);
    }
}
