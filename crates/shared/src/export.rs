//! Serialization and data interchange engines for JSON and CSV import/export.

use crate::models::{AppState, Item, ItemCollection, Priority};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExportError {
    #[error("JSON serialization error: {0}")]
    JsonSerialize(#[from] serde_json::Error),
    #[error("CSV parsing error: {0}")]
    CsvParse(String),
    #[error("Invalid priority format: {0}")]
    InvalidPriority(String),
    #[error("Empty data provided")]
    EmptyData,
}

/// Serializes application state to formatted JSON.
pub fn export_to_json(state: &AppState) -> Result<String, ExportError> {
    serde_json::to_string_pretty(state).map_err(ExportError::JsonSerialize)
}

/// Deserializes application state from JSON string.
pub fn import_from_json(json_str: &str) -> Result<AppState, ExportError> {
    let trimmed = json_str.trim();
    if trimmed.is_empty() {
        return Err(ExportError::EmptyData);
    }
    serde_json::from_str(trimmed).map_err(ExportError::JsonSerialize)
}

/// Serializes item collection to CSV string.
pub fn export_to_csv(collection: &ItemCollection) -> Result<String, ExportError> {
    let mut csv = String::from("id,title,description,priority,completed,created_at\n");
    for item in &collection.items {
        let escaped_title = escape_csv_field(&item.title);
        let escaped_desc = escape_csv_field(&item.description);
        csv.push_str(&format!(
            "{},{},{},{},{},{}\n",
            escape_csv_field(&item.id),
            escaped_title,
            escaped_desc,
            item.priority.label(),
            item.completed,
            item.created_at
        ));
    }
    Ok(csv)
}

/// Deserializes item collection from CSV string.
pub fn import_from_csv(csv_str: &str) -> Result<ItemCollection, ExportError> {
    let trimmed = csv_str.trim();
    if trimmed.is_empty() {
        return Err(ExportError::EmptyData);
    }

    let mut items = Vec::new();
    let mut lines = trimmed.lines();

    // Verify or skip header
    if let Some(header) = lines.next()
        && !header.to_lowercase().contains("title")
    {
        // Process as data if header isn't standard
        if let Some(item) = parse_csv_line(header)? {
            items.push(item);
        }
    }

    for line in lines {
        if let Some(item) = parse_csv_line(line)? {
            items.push(item);
        }
    }

    if items.is_empty() {
        return Err(ExportError::CsvParse(
            "No valid items parsed from CSV".to_string(),
        ));
    }

    Ok(ItemCollection { items })
}

fn parse_csv_line(line: &str) -> Result<Option<Item>, ExportError> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let fields = parse_csv_fields(trimmed);
    if fields.is_empty() {
        return Ok(None);
    }

    let id = fields.first().cloned().unwrap_or_default();
    let title = fields
        .get(1)
        .cloned()
        .unwrap_or_else(|| "Untitled".to_string());
    let description = fields.get(2).cloned().unwrap_or_default();
    let priority_str = fields.get(3).map(|s| s.as_str()).unwrap_or("Medium");
    let completed_str = fields.get(4).map(|s| s.as_str()).unwrap_or("false");
    let created_at_str = fields.get(5).map(|s| s.as_str()).unwrap_or("0");

    let priority = match priority_str.to_lowercase().as_str() {
        "low" => Priority::Low,
        "medium" => Priority::Medium,
        "high" => Priority::High,
        "critical" => Priority::Critical,
        other => return Err(ExportError::InvalidPriority(other.to_string())),
    };

    let completed = completed_str.eq_ignore_ascii_case("true") || completed_str == "1";
    let created_at = created_at_str.parse::<u64>().unwrap_or(0);

    Ok(Some(Item {
        id: if id.is_empty() {
            format!("item-{}", items_hash(&title))
        } else {
            id
        },
        title,
        description,
        priority,
        completed,
        created_at,
    }))
}

fn items_hash(val: &str) -> u32 {
    let mut h: u32 = 5381;
    for b in val.bytes() {
        h = ((h << 5).wrapping_add(h)).wrapping_add(b as u32);
    }
    h
}

use spodeian_export::{escape_csv_field, parse_csv_line as parse_csv_fields};

/// Exports the entire application state into compressed BSON binary bytes (Zlib-compressed BSON).
pub fn export_to_compressed_bson(state: &AppState) -> Result<Vec<u8>, String> {
    spodeian_export::export_to_compressed_bson(state)
        .map_err(|e| format!("BSON serialization failed: {e}"))
}

/// Imports and restores an AppState from a compressed (or raw) BSON slice.
pub fn import_from_compressed_bson(bytes: &[u8]) -> Result<AppState, String> {
    spodeian_export::import_from_compressed_bson(bytes)
        .map_err(|e| format!("BSON deserialization failed: {e}"))
}
