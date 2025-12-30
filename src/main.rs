#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod args;
#[cfg(feature = "bedrock")]
mod bedrock_block_map;
mod block_definitions;
mod bresenham;
mod cache_manager;
mod chunked_generation;
mod clipping;
mod colors;
mod coordinate_system;
mod data_processing;
mod element_processing;
mod elevation_data;
mod floodfill;
mod ground;
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
mod version_check;
mod world_editor;

use args::Args;
use cache_manager::{CacheManager, format_bytes};
use chunked_generation;
use clap::Parser;
use colored::*;
use data_processing::WorldFormat;
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

fn run_cli() {
    let version: &str = env!("CARGO_PKG_VERSION");
    let repository: &str = env!("CARGO_PKG_REPOSITORY");
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

    // Check for updates
    if let Err(e) = version_check::check_for_updates() {
        eprintln!(
            "{}: {}",
            "Error checking for version updates".red().bold(),
            e
        );
    }

    // Parse input arguments
    let args: Args = Args::parse();

    // Initialize cache manager
    let cache_manager = if let Some(ref cache_dir) = args.cache_dir {
        CacheManager::with_directory(cache_dir.clone())
    } else {
        CacheManager::new()
    }
    .expect("Failed to initialize cache manager");

    // Handle cache-specific commands
    if args.list_caches {
        handle_list_caches(&cache_manager);
        return;
    }

    if let Some(ref cache_id) = args.delete_cache {
        handle_delete_cache(&cache_manager, cache_id);
        return;
    }

    if args.clear_caches {
        handle_clear_caches(&cache_manager);
        return;
    }

    // Get bbox - either from args or from cache
    let bbox = if let Some(ref cache_id) = args.from_cache {
        handle_generate_from_cache(&cache_manager, cache_id, &args);
        return;
    } else {
        args.bbox.expect("Bounding box is required")
    };

    // Fetch data
    let raw_data = match &args.file {
        Some(file) => retrieve_data::fetch_data_from_file(file),
        None => retrieve_data::fetch_data_from_overpass(
            bbox,
            args.debug,
            args.downloader.as_str(),
            args.save_json_file.as_deref(),
        ),
    }
    .expect("Failed to fetch data");

    // If cache-only mode, save the data and exit
    if args.cache_only {
        handle_cache_only(&cache_manager, &bbox, args.scale, &raw_data, args.terrain);
        return;
    }

    // Check if chunked generation is needed
    let config = chunked_generation::ChunkedGenerationConfig::default();
    let needs_chunking = chunked_generation::needs_chunking(&bbox, &config);

    if needs_chunking {
        println!("{}", "Large area detected - using chunked generation for better performance".yellow().bold());

        // Get element count for recommendations
        let element_count = raw_data["elements"]
            .as_array()
            .map(|arr| arr.len())
            .unwrap_or(0);

        // Show recommendations
        let recommendations = chunked_generation::get_generation_recommendations(&bbox, element_count);
        if !recommendations.is_empty() {
            println!("\n{}", "Recommendations:".bright_white().bold());
            for rec in recommendations {
                println!("  • {}", rec.yellow());
            }
            println!();
        }

        // Create chunks
        let chunks = chunked_generation::create_chunks(&bbox, &config)
            .expect("Failed to create generation chunks");

        let ground = ground::generate_ground_data(&args);

        // Create generation options
        let path = args.path.as_ref()
            .expect("Path is required for world generation")
            .clone();
        let options = data_processing::GenerationOptions {
            path,
            format: data_processing::WorldFormat::JavaAnvil,
            level_name: None,
            spawn_point: None,
        };

        // Generate world using chunked approach
        let _ = chunked_generation::generate_world_chunked(
            chunks,
            raw_data,
            args.scale,
            &ground,
            &args,
            options,
        );
    } else {
        // Standard generation for smaller areas
        let mut ground = ground::generate_ground_data(&args);

        // Parse raw data
        let (mut parsed_elements, mut xzbbox) =
            osm_parser::parse_osm_data(raw_data, bbox, args.scale, args.debug);
        parsed_elements
            .sort_by_key(|element: &osm_parser::ProcessedElement| osm_parser::get_priority(element));

        // Write the parsed OSM data to a file for inspection
        if args.debug {
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
        map_transformation::transform_map(&mut parsed_elements, &mut xzbbox, &mut ground);

        // Generate world
        let _ = data_processing::generate_world(parsed_elements, xzbbox, bbox, ground, &args);
    }
}

/// Handle listing all cached regions
fn handle_list_caches(cache_manager: &CacheManager) {
    println!("{}", "Available cached regions:".bold());

    match cache_manager.list_caches() {
        Ok(caches) => {
            if caches.is_empty() {
                println!("  {}", "No cached regions found.".yellow());
                println!("\n  Use {} to pre-cache a region.", "--cache-only".green());
            } else {
                for cache in caches {
                    println!("\n  {} {}", "ID:".bold(), cache.id.cyan());
                    println!("  {} {}", "Name:".bold(), cache.name);
                    println!("  {} {}", "Bbox:".bold(), cache.bbox);
                    println!("  {} {:.2}", "Scale:".bold(), cache.scale);
                    println!("  {} {}", "Terrain:".bold(), if cache.has_terrain { "Yes".green() } else { "No".red() });
                    println!("  {} {}", "Elements:".bold(), cache.element_count);
                    println!("  {} {}", "Size:".bold(), format_bytes(cache.size_bytes));
                    println!("  {} {}", "Created:".bold(), cache.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
                }

                if let Ok(total_size) = cache_manager.get_total_cache_size() {
                    println!("\n  {} {}", "Total cache size:".bold(), format_bytes(total_size));
                }
            }
        }
        Err(e) => {
            eprintln!("{}: {}", "Error listing caches".red().bold(), e);
            std::process::exit(1);
        }
    }
}

/// Handle deleting a specific cached region
fn handle_delete_cache(cache_manager: &CacheManager, cache_id: &str) {
    println!("{} '{}'...", "Deleting cache".bold(), cache_id.cyan());

    match cache_manager.delete_cache(cache_id) {
        Ok(()) => {
            println!("{}", "Cache deleted successfully.".green().bold());
        }
        Err(e) => {
            eprintln!("{}: {}", "Error deleting cache".red().bold(), e);
            std::process::exit(1);
        }
    }
}

/// Handle clearing all cached regions
fn handle_clear_caches(cache_manager: &CacheManager) {
    println!("{}", "Clearing all caches...".bold());

    match cache_manager.clear_all_caches() {
        Ok(()) => {
            println!("{}", "All caches cleared successfully.".green().bold());
        }
        Err(e) => {
            eprintln!("{}: {}", "Error clearing caches".red().bold(), e);
            std::process::exit(1);
        }
    }
}

/// Handle cache-only mode (pre-cache data without generating world)
fn handle_cache_only(
    cache_manager: &CacheManager,
    bbox: &coordinate_system::geographic::LLBBox,
    scale: f64,
    raw_data: &serde_json::Value,
    terrain_enabled: bool,
) {
    println!("{}", "[Cache Mode] Saving data to cache...".bold());

    // Get area name for better cache identification
    let center_lat = (bbox.min().lat() + bbox.max().lat()) / 2.0;
    let center_lon = (bbox.min().lng() + bbox.max().lng()) / 2.0;
    let area_name = retrieve_data::fetch_area_name(center_lat, center_lon)
        .ok()
        .flatten();

    // Note: Elevation data would need to be fetched separately if terrain is enabled
    // For now, we just mark whether terrain was requested
    let elevation_data = if terrain_enabled {
        // TODO: Fetch and save elevation data here
        None
    } else {
        None
    };

    match cache_manager.save_cache(bbox, scale, raw_data, elevation_data.as_ref(), area_name, Some(30)) {
        Ok(cache_id) => {
            println!("{}", "Data cached successfully!".green().bold());
            println!("  {} {}", "Cache ID:".bold(), cache_id.cyan());
            println!("\n  Use {} to generate world from this cache.", format!("--from-cache {}", cache_id).green());
            println!("  Cache location: {}", cache_manager.cache_dir().display());
        }
        Err(e) => {
            eprintln!("{}: {}", "Error saving cache".red().bold(), e);
            std::process::exit(1);
        }
    }
}

/// Handle generating world from cached data
fn handle_generate_from_cache(
    cache_manager: &CacheManager,
    cache_id: &str,
    args: &Args,
) {
    println!("{} '{}'...", "Loading cache".bold(), cache_id.cyan());

    let cache_entry = match cache_manager.load_cache(cache_id) {
        Ok(entry) => entry,
        Err(e) => {
            eprintln!("{}: {}", "Error loading cache".red().bold(), e);
            std::process::exit(1);
        }
    };

    println!("{}", "Cache loaded successfully!".green().bold());
    println!("  {} {}", "Name:".bold(), cache_entry.metadata.name);
    println!("  {} {}", "Elements:".bold(), cache_entry.metadata.element_count);

    // Ensure path is provided
    if args.path.is_none() {
        eprintln!("{}", "Error: --path is required when generating from cache".red().bold());
        std::process::exit(1);
    }

    // Parse bbox from cache metadata
    let bbox = coordinate_system::geographic::LLBBox::from_str(&cache_entry.metadata.bbox)
        .expect("Invalid bbox in cache metadata");

    // Use scale from cache
    let scale = cache_entry.metadata.scale;

    // Generate ground data
    let terrain_enabled = cache_entry.metadata.has_terrain;
    let mut ground = if terrain_enabled && cache_entry.elevation_data.is_some() {
        // TODO: Use cached elevation data
        ground::Ground::new(args.ground_level, false)
    } else {
        ground::Ground::new(args.ground_level, false)
    };

    // Check if chunked generation is needed
    let config = chunked_generation::ChunkedGenerationConfig::default();
    if chunked_generation::needs_chunking(&bbox, &config) {
        println!("{}", "Large area detected in cache - using chunked generation".yellow().bold());

        // Create chunks
        let chunks = chunked_generation::create_chunks(&bbox, &config)
            .expect("Failed to create generation chunks");

        // Create generation options
        let path = args.path.as_ref()
            .expect("Path is required for world generation")
            .clone();
        let options = data_processing::GenerationOptions {
            path,
            format: data_processing::WorldFormat::JavaAnvil,
            level_name: None,
            spawn_point: None,
        };

        // Generate world using chunked approach
        let _ = chunked_generation::generate_world_chunked(
            chunks,
            cache_entry.osm_data,
            scale,
            &ground,
            args,
            options,
        );
    } else {
        // Standard generation
        let (mut parsed_elements, mut xzbbox) =
            osm_parser::parse_osm_data(cache_entry.osm_data, bbox, scale, args.debug);
        parsed_elements
            .sort_by_key(|element: &osm_parser::ProcessedElement| osm_parser::get_priority(element));

        // Transform map
        map_transformation::transform_map(&mut parsed_elements, &mut xzbbox, &mut ground);

        // Generate world
        let _ = data_processing::generate_world(parsed_elements, xzbbox, bbox, ground, args);
    }
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
