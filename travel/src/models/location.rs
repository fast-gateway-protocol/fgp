//! Location models and lookup tables.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Load hotel locations from JSON file (224 cities) (Claude)
//! 01/14/2026 - Initial implementation (Claude)

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Embedded hotel location keys JSON (TripAdvisor g-codes).
const HOTEL_LOCATIONS_JSON: &str = include_str!("../../data/hotel_locations.json");

/// Airport, city, or country location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub name: String,
    pub slug: String,
    /// AIRPORT, CITY, COUNTRY, etc.
    #[serde(rename = "type")]
    pub location_type: String,
    pub city: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl Location {
    /// Format for display.
    pub fn display(&self) -> String {
        if self.location_type == "AIRPORT" {
            format!(
                "{} ({}) - {}, {}",
                self.name,
                self.id,
                self.city.as_deref().unwrap_or(""),
                self.country.as_deref().unwrap_or("")
            )
        } else {
            format!("{} ({})", self.name, self.location_type)
        }
    }
}

/// Hotel location keys database loaded from JSON.
pub struct HotelLocationDb {
    locations: HashMap<String, String>,
}

impl HotelLocationDb {
    /// Load hotel locations from embedded JSON.
    fn load() -> Self {
        let data: Value = serde_json::from_str(HOTEL_LOCATIONS_JSON)
            .expect("Failed to parse hotel_locations.json");

        let mut locations = HashMap::new();

        if let Some(locs) = data.get("locations").and_then(|v| v.as_object()) {
            for (city, code) in locs {
                if let Some(code_str) = code.as_str() {
                    locations.insert(city.to_lowercase(), code_str.to_string());
                }
            }
        }

        tracing::info!("Loaded {} hotel location keys", locations.len());
        Self { locations }
    }

    /// Get the singleton instance.
    pub fn instance() -> &'static Self {
        static INSTANCE: Lazy<HotelLocationDb> = Lazy::new(HotelLocationDb::load);
        &INSTANCE
    }

    /// Look up a TripAdvisor location key by city name.
    pub fn get(&self, city: &str) -> Option<&str> {
        self.locations.get(&city.to_lowercase()).map(|s| s.as_str())
    }

    /// Get the number of locations in the database.
    pub fn len(&self) -> usize {
        self.locations.len()
    }

    /// Check if database is empty.
    pub fn is_empty(&self) -> bool {
        self.locations.is_empty()
    }
}

/// Get a TripAdvisor location key for a city name.
pub fn get_location_key(city: &str) -> Option<&'static str> {
    HotelLocationDb::instance().get(city)
}
