use crate::coordinate_system::geographic::LLBBox;
use crate::debug;
use crate::error;
use crate::info;
use crate::osm_parser::OsmData;
use crate::progress::{emit_gui_error, emit_gui_progress_update, is_running_with_gui};
#[cfg(feature = "gui")]
use crate::telemetry::{send_log, LogLevel as TelemetryLogLevel};
use crate::warn;
use colored::Colorize;
use rand::seq::SliceRandom;
use reqwest::blocking::Client;
use reqwest::blocking::ClientBuilder;
use serde::Deserialize;
use serde_json::Value;
use std::fs::File;
use std::io::{self, BufReader, Cursor, Write};
use std::process::Command;
use std::time::Duration;

/// Function to download data using reqwest with proper error context
fn download_with_reqwest(url: &str, query: &str) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Building HTTP client for request to {}", url);
    let client: Client = ClientBuilder::new()
        .timeout(Duration::from_secs(360))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    debug!("Sending request to {}", url);
    let response: Result<reqwest::blocking::Response, reqwest::Error> =
        client.get(url).query(&[("data", query)]).send();

    match response {
        Ok(resp) => {
            emit_gui_progress_update(3.0, "Downloading data...");
            let status = resp.status();
            if status.is_success() {
                let text = resp
                    .text()
                    .map_err(|e| format!("Failed to read response body: {}", e))?;
                if text.is_empty() {
                    error!("Received empty response from server");
                    return Err("Received empty response from server".into());
                }
                debug!("Successfully downloaded {} bytes", text.len());
                Ok(text)
            } else {
                let err_msg = format!("Server returned error status: {}", status);
                error!("{}", err_msg);
                Err(err_msg.into())
            }
        }
        Err(e) => {
            if e.is_timeout() {
                let msg = "Request timed out. Try selecting a smaller area.";
                error!("{}", msg);
                eprintln!("{}", format!("Error! {msg}").red().bold());
                Err(msg.into())
            } else if e.is_connect() {
                let msg = "No internet connection.";
                error!("{}", msg);
                eprintln!("{}", format!("Error! {msg}").red().bold());
                Err(msg.into())
            } else {
                #[cfg(feature = "gui")]
                send_log(
                    TelemetryLogLevel::Error,
                    &format!("Request error in download_with_reqwest: {e}"),
                );
                let err_msg = format!("Request failed: {e:.52}");
                error!("{}", err_msg);
                eprintln!("{}", format!("Error! {e:.52}").red().bold());
                Err(err_msg.into())
            }
        }
    }
}

/// Function to download data using `curl`
fn download_with_curl(url: &str, query: &str) -> io::Result<String> {
    let output: std::process::Output = Command::new("curl")
        .arg("-s") // Add silent mode to suppress output
        .arg(format!("{url}?data={query}"))
        .output()?;

    if !output.status.success() {
        Err(io::Error::other("Curl command failed"))
    } else {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Function to download data using `wget`
fn download_with_wget(url: &str, query: &str) -> io::Result<String> {
    let output: std::process::Output = Command::new("wget")
        .arg("-qO-") // Use `-qO-` to output the result directly to stdout
        .arg(format!("{url}?data={query}"))
        .output()?;

    if !output.status.success() {
        Err(io::Error::other("Wget command failed"))
    } else {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Load OSM data from a JSON file with proper error context
pub fn fetch_data_from_file(file_path: &str) -> Result<OsmData, Box<dyn std::error::Error>> {
    info!("[1/7] Loading data from file: {}", file_path);
    emit_gui_progress_update(1.0, "Loading data from file...");

    let file =
        File::open(file_path).map_err(|e| format!("Failed to open file '{}': {}", file_path, e))?;

    let reader = BufReader::new(file);
    let mut deserializer = serde_json::Deserializer::from_reader(reader);

    let data: OsmData = OsmData::deserialize(&mut deserializer)
        .map_err(|e| format!("Failed to parse JSON in file '{}': {}", file_path, e))?;

    info!("Successfully loaded {} elements from file", data.len());

    Ok(data)
}

/// Main function to fetch data from Overpass API
///
/// Attempts to fetch data from Overpass API servers with fallback options.
/// Will retry with fallback servers if the primary servers fail.
pub fn fetch_data_from_overpass(
    bbox: LLBBox,
    debug: bool,
    download_method: &str,
    save_file: Option<&str>,
) -> Result<OsmData, Box<dyn std::error::Error>> {
    info!("[1/7] Fetching data from Overpass API");
    emit_gui_progress_update(1.0, "Fetching data...");

    // List of Overpass API servers
    let api_servers: Vec<&str> = vec![
        "https://overpass-api.de/api/interpreter",
        "https://lz4.overpass-api.de/api/interpreter",
        "https://z.overpass-api.de/api/interpreter",
    ];
    let fallback_api_servers: Vec<&str> =
        vec!["https://maps.mail.ru/osm/tools/overpass/api/interpreter"];

    // Safely select a random server with fallback
    let mut url: &&str = api_servers
        .choose(&mut rand::thread_rng())
        .unwrap_or(&"https://overpass-api.de/api/interpreter");

    debug!("Selected primary API server: {}", url);

    // Generate Overpass API query for bounding box
    let query: String = format!(
        r#"[out:json][timeout:360][bbox:{},{},{},{}];
    (
        nwr["building"];
        nwr["building:part"];
        nwr["highway"];
        nwr["landuse"];
        nwr["natural"];
        nwr["leisure"];
        nwr["water"];
        nwr["waterway"];
        nwr["amenity"];
        nwr["tourism"];
        nwr["bridge"];
        nwr["railway"];
        nwr["roller_coaster"];
        nwr["barrier"];
        nwr["entrance"];
        nwr["door"];
        nwr["boundary"];
        nwr["power"];
        nwr["historic"];
        nwr["emergency"];
        nwr["advertising"];
        nwr["man_made"];
        nwr["aeroway"];
        way;
    )->.relsinbbox;
    (
        way(r.relsinbbox);
    )->.waysinbbox;
    (
        node(w.waysinbbox);
        node(w.relsinbbox);
    )->.nodesinbbox;
    .relsinbbox out body;
    .waysinbbox out body;
    .nodesinbbox out skel qt;"#,
        bbox.min().lat(),
        bbox.min().lng(),
        bbox.max().lat(),
        bbox.max().lng(),
    );

    {
        // Fetch data from Overpass API with retry logic
        let mut attempt = 0;
        let max_attempts = 2; // Try primary + one fallback
        let response: String = loop {
            info!(
                "Attempt {}: Downloading from {} using {}",
                attempt + 1,
                url,
                download_method
            );
            println!("Downloading from {url} with method {download_method}...");

            let result = match download_method {
                "requests" => download_with_reqwest(url, &query),
                "curl" => download_with_curl(url, &query).map_err(|e| e.into()),
                "wget" => download_with_wget(url, &query).map_err(|e| e.into()),
                _ => download_with_reqwest(url, &query), // Default to requests
            };

            match result {
                Ok(response) => {
                    info!("Successfully downloaded data on attempt {}", attempt + 1);
                    break response;
                }
                Err(error) => {
                    warn!("Attempt {} failed: {}", attempt + 1, error);
                    if attempt >= max_attempts - 1 {
                        return Err(format!(
                            "Failed to fetch data after {} attempts: {}",
                            max_attempts, error
                        )
                        .into());
                    }

                    println!("Request failed. Switching to fallback url...");
                    url = fallback_api_servers
                        .choose(&mut rand::thread_rng())
                        .unwrap_or(&"https://maps.mail.ru/osm/tools/overpass/api/interpreter");
                    attempt += 1;
                }
            }
        };

        // Save response to file if requested
        if let Some(save_file) = save_file {
            info!("Saving API response to: {}", save_file);
            let mut file = File::create(save_file)
                .map_err(|e| format!("Failed to create save file '{}': {}", save_file, e))?;
            file.write_all(response.as_bytes())
                .map_err(|e| format!("Failed to write to save file '{}': {}", save_file, e))?;
            println!("API response saved to: {save_file}");
        }

        // Parse JSON response
        debug!("Parsing JSON response");
        let mut deserializer =
            serde_json::Deserializer::from_reader(Cursor::new(response.as_bytes()));
        let data: OsmData = OsmData::deserialize(&mut deserializer)
            .map_err(|e| format!("Failed to parse API response: {}", e))?;

        // Check if data is empty
        if data.is_empty() {
            if let Some(remark) = data.remark.as_deref() {
                // Check if the remark mentions memory or other runtime errors
                if remark.contains("runtime error") && remark.contains("out of memory") {
                    let msg = "The query ran out of memory on the Overpass API server. Try using a smaller area.";
                    error!("{}", msg);
                    eprintln!("{}", format!("Error! {}", msg).red().bold());
                    emit_gui_error("Try using a smaller area.");
                } else {
                    // Handle other Overpass API errors if present in the remark field
                    error!("API returned error: {}", remark);
                    eprintln!("{}", format!("Error! API returned: {remark}").red().bold());
                    emit_gui_error(&format!("API returned: {remark}"));
                }
            } else {
                // General case for when there are no elements and no specific remark
                error!("API returned no data");
                eprintln!(
                    "{}",
                    "Error! API returned no data. Please try again!"
                        .red()
                        .bold()
                );
                emit_gui_error("API returned no data. Please try again!");
            }

            if debug {
                println!("Additional debug information: {data:?}");
            }

            if !is_running_with_gui() {
                std::process::exit(1);
            } else {
                return Err("Data fetch failed".into());
            }
        }

        emit_gui_progress_update(5.0, "");

        Ok(data)
    }
}

/// Fetches a short area name using Nominatim for the given lat/lon
///
/// Uses the Nominatim reverse geocoding API to get a human-readable
/// area name from coordinates. Returns None if no name can be determined.
pub fn fetch_area_name(lat: f64, lon: f64) -> Result<Option<String>, Box<dyn std::error::Error>> {
    debug!("Fetching area name for coordinates: {}, {}", lat, lon);

    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let url = format!(
        "https://nominatim.openstreetmap.org/reverse?format=jsonv2&lat={lat}&lon={lon}&addressdetails=1"
    );

    let resp = client
        .get(&url)
        .header("User-Agent", "arnis-rust")
        .send()
        .map_err(|e| format!("Failed to query Nominatim API: {}", e))?;

    if !resp.status().is_success() {
        debug!(
            "Nominatim API returned non-success status: {}",
            resp.status()
        );
        return Ok(None);
    }

    let json: Value = resp
        .json()
        .map_err(|e| format!("Failed to parse Nominatim response: {}", e))?;

    if let Some(address) = json.get("address") {
        let fields = ["city", "town", "village", "county", "borough", "suburb"];
        for field in fields.iter() {
            if let Some(name) = address.get(*field).and_then(|v| v.as_str()) {
                let mut name_str = name.to_string();

                // Remove "City of " prefix safely
                if name_str.to_lowercase().starts_with("city of ") {
                    if let Some(idx) = name_str.find(" of ") {
                        name_str = name_str[idx + 4..].to_string();
                    }
                }

                debug!("Found area name: {}", name_str);
                return Ok(Some(name_str));
            }
        }
    }

    debug!("No area name found for coordinates: {}, {}", lat, lon);
    Ok(None)
}
