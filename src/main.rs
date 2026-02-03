#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod args;
#[cfg(feature = "bedrock")]
mod bedrock_block_map;
mod block_definitions;
mod bresenham;
mod clipping;
mod colors;
mod coordinate_system;
mod data_processing;
mod deterministic_rng;
mod element_processing;
mod elevation_data;
mod floodfill;
mod floodfill_cache;
mod ground;
mod logger;
mod map_renderer;
mod map_transformation;
mod osm_parser;
#[cfg(feature = "gui")]
mod progress;
mod retrieve_data;
#[cfg(feature = "gui")]
mod telemetry;
#[cfg(test)]
mod test_utilities;
mod urban_ground;
mod version_check;
mod world_editor;

use args::Args;
use clap::Parser;
use colored::*;
use logger::LogLevel;
use std::{env, fs, io::Write};

#[cfg(feature = "gui")]
mod gui;

// If the user does not want the GUI, it's easiest to just mock the progress module to do nothing
#[cfg(not(feature = "gui"))]
mod progress {
    pub fn emit_gui_error(_message: &str) {}
    pub fn emit_gui_progress_update(_progress: f64, _message: &str) {}
    pub fn emit_map_preview_ready() {}
    pub fn emit_open_mcworld_file(_path: &str) {}
    pub fn is_running_with_gui() -> bool {
        false
    }
}
#[cfg(target_os = "windows")]
use windows::Win32::System::Console::{AttachConsole, FreeConsole, ATTACH_PARENT_PROCESS};

#[allow(clippy::unnecessary_lazy_evaluations)]
fn run_cli() {
    // Parse input arguments
    let args: Args = Args::parse();

    // Initialize logging system. Use CLI overrides when present, otherwise default behavior.
    let log_level = args.log_level.unwrap_or_else(|| {
        if args.debug {
            LogLevel::Debug
        } else {
            LogLevel::Info
        }
    });
    let show_timestamps = !args.no_log_timestamps;
    let use_colors = !args.no_log_colors;
    logger::init(log_level, show_timestamps, use_colors);

    // Configure thread pool with 90% CPU cap to keep system responsive
    floodfill_cache::configure_rayon_thread_pool(0.9);
    info!("Configured thread pool with 90% CPU limit");

    // Clean up old cached elevation tiles on startup
    elevation_data::cleanup_old_cached_tiles();
    info!("Cleaned up cached elevation tiles");

    let version: &str = env!("CARGO_PKG_VERSION");
    let repository: &str = env!("CARGO_PKG_REPOSITORY");
    if use_colors {
        println!(
            r#"
        ▄████████    ▄████████ ███▄▄▄▄    ▄█     ▄████████
        ███    ███   ███    ███ ███▀▀▀██▄ ███    ███    ███
        ███    ███   ███    ███ ███   ███ ███▌   ███    █▀
        ███    ███  ▄███▄▄▄▄██▀ ███   ███ ███▌   ███
      ▀███████████ ▀▀███▀▀▀▀▀   ███   ███ ███▌ ▀███████████
        ███    ███ ▀███████████ ███   ███ ███           ███
        ███    ███   ███    ███ ███   ███ ███     ▄█    ███
        ███    █▀    ███    ███  ▀█   █▀  █▀    ▄████████▀
                     ███    ███

                          version {}
                {}
        "#,
            version,
            repository.bright_white().bold()
        );
    }

    info!("Starting Arnis v{}", version);

    // Check for updates
    if let Err(e) = version_check::check_for_updates() {
        error!("Failed to check for updates: {}", e);
    }

    info!(
        "Parsed CLI arguments - path: {:?}, bbox: {:?}",
        args.path, args.bbox
    );

    // Fetch data with better error handling
    let raw_data = match &args.file {
        Some(file) => {
            info!("Loading data from file: {}", file);
            retrieve_data::fetch_data_from_file(file)
        }
        None => {
            info!("Fetching data from Overpass API");
            retrieve_data::fetch_data_from_overpass(
                args.bbox,
                args.debug,
                args.downloader.as_str(),
                args.save_json_file.as_deref(),
            )
        }
    };

    let raw_data = match raw_data {
        Ok(data) => {
            info!("Successfully fetched OSM data");
            data
        }
        Err(e) => {
            error!("Failed to fetch data: {}", e);
            std::process::exit(1);
        }
    };

    let mut ground = ground::generate_ground_data(&args);

    // Parse raw data
    info!("Parsing OSM data...");
    let (mut parsed_elements, mut xzbbox) =
        osm_parser::parse_osm_data(raw_data, args.bbox, args.scale, args.debug);
    info!("Parsed {} elements", parsed_elements.len());

    parsed_elements
        .sort_by_key(|element: &osm_parser::ProcessedElement| osm_parser::get_priority(element));

    // Write the parsed OSM data to a file for inspection
    if args.debug {
        debug!("Writing parsed OSM data to debug file");
        let mut buf = std::io::BufWriter::new(
            fs::File::create("parsed_osm_data.txt").expect("Failed to create output file"),
        );
        for element in &parsed_elements {
            writeln!(
                buf,
                "Element ID: {}, Type: {}, Tags: {:?}",
                element.id(),
                element.kind(),
                element.tags(),
            )
            .expect("Failed to write to output file");
        }
    }

    // Transform map (parsed_elements). Operations are defined in a json file
    info!("Applying map transformations...");
    map_transformation::transform_map(&mut parsed_elements, &mut xzbbox, &mut ground);

    // Generate world
    info!("Starting world generation...");
    let _ = data_processing::generate_world(parsed_elements, xzbbox, args.bbox, ground, &args);
    info!("World generation completed");
}

fn main() {
    // If on Windows, free and reattach to the parent console when using as a CLI tool
    // Either of these can fail, but if they do it is not an issue, so the return value is ignored
    #[cfg(target_os = "windows")]
    unsafe {
        let _ = FreeConsole();
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }

    // Only run CLI mode if the user supplied args.
    #[cfg(feature = "gui")]
    {
        let gui_mode = std::env::args().len() == 1; // Just "arnis" with no args
        if gui_mode {
            gui::run_gui();
        }
    }

    run_cli();
}
