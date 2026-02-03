//! Coordinate System Module
//!
//! This module handles all coordinate transformations between geographic (latitude/longitude)
//! and Cartesian (Minecraft X/Z) coordinate systems. It provides the core spatial infrastructure
//! for converting real-world OpenStreetMap data into Minecraft world coordinates.
//!
//! # Architecture
//!
//! The coordinate system is split into two main components:
//!
//! - **Geographic**: Handles latitude/longitude coordinates (WGS84)
//!   - `LLPoint`: Single point in lat/lon space
//!   - `LLBBox`: Bounding box in lat/lon space
//!
//! - **Cartesian**: Handles Minecraft block coordinates
//!   - `XZPoint`: Single point in block coordinates
//!   - `XZBBox`: Bounding box in block coordinates
//!
//! - **Transformation**: Converts between coordinate systems
//!   - `CoordTransformer`: Main transformation engine
//!
//! # Example Usage
//!
//! ```rust
//! use crate::coordinate_system::geographic::{LLBBox, LLPoint};
//! use crate::coordinate_system::transformation::CoordTransformer;
//!
//! // Define a geographic bounding box
//! let llbbox = LLBBox::new(48.0, 11.0, 49.0, 12.0).unwrap();
//!
//! // Convert to Minecraft coordinates with scale factor
//! let (transformer, xzbbox) = CoordTransformer::llbbox_to_xzbbox(&llbbox, 1.0).unwrap();
//!
//! // Transform individual points
//! let xz_point = transformer.transform_point(&LLPoint::new(48.5, 11.5).unwrap());
//! ```

pub mod cartesian;
pub mod geographic;
pub mod transformation;
