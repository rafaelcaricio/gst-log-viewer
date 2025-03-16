//! Parser for GStreamer logs
//!
//! This module uses the gst-log-parser crate by Guillaume Desmottes
//! https://github.com/gdesmott/gst-log-parser/

// Re-export the Entry type and other useful types from the crate
pub use gst_log_parser::Entry;

use std::io::Read;

/// Parse GStreamer log entries from a reader
pub fn parse<R: Read>(r: R) -> impl Iterator<Item = Entry> {
    // We don't initialize gstreamer here as it's done in main.rs
    // and we don't want to initialize it multiple times
    gst_log_parser::parse(r)
}

// Extended functionality for Entry from gst-log-parser
use gstreamer::Structure;
use std::str::FromStr;

// Add a utility method to convert message to GStreamer Structure if not provided by the crate
// This is currently not used but kept for potential future use
#[allow(dead_code)]
pub trait EntryExt {
    fn message_to_struct(&self) -> Option<Structure>;
}

impl EntryExt for Entry {
    fn message_to_struct(&self) -> Option<Structure> {
        Structure::from_str(&self.message).ok()
    }
}
