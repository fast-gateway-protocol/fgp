//! Local location database for instant airport/city lookups.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation with embedded location data (Claude)

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Location data embedded at compile time.
const LOCATIONS_JSON: &str = include_str!("../data/locations.json");

/// Pre-parsed location database.
static LOCATIONS: Lazy<LocationDb> =
    Lazy::new(|| LocationDb::load().expect("Failed to load embedded location data"));

/// A searchable location (airport, city, etc).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub name: Option<String>,
    pub slug: Option<String>,
    #[serde(rename = "type")]
    pub location_type: Option<String>,
    pub code: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
}

/// Local location database with search index.
pub struct LocationDb {
    locations: Vec<Location>,
    by_code: HashMap<String, usize>,
    by_id: HashMap<String, usize>,
}

impl LocationDb {
    /// Load and index locations from embedded JSON.
    fn load() -> Result<Self, serde_json::Error> {
        let locations: Vec<Location> = serde_json::from_str(LOCATIONS_JSON)?;

        let mut by_code = HashMap::new();
        let mut by_id = HashMap::new();

        for (i, loc) in locations.iter().enumerate() {
            if let Some(code) = &loc.code {
                by_code.insert(code.to_uppercase(), i);
            }
            by_id.insert(loc.id.clone(), i);
        }

        tracing::info!("Loaded {} locations into local database", locations.len());

        Ok(Self {
            locations,
            by_code,
            by_id,
        })
    }

    /// Get database instance.
    pub fn instance() -> &'static Self {
        &LOCATIONS
    }

    /// Search locations by term.
    /// Matches against code, name, city, and country.
    pub fn search(&self, term: &str, limit: usize) -> Vec<&Location> {
        let term_lower = term.to_lowercase();
        let term_upper = term.to_uppercase();

        // Exact code match first
        if let Some(&idx) = self.by_code.get(&term_upper) {
            let mut results = vec![&self.locations[idx]];
            // Add a few more matches
            results.extend(
                self.locations
                    .iter()
                    .filter(|l| l.code.as_deref() != Some(&term_upper))
                    .filter(|l| self.matches(l, &term_lower))
                    .take(limit.saturating_sub(1)),
            );
            return results;
        }

        // Otherwise, search all fields
        self.locations
            .iter()
            .filter(|l| self.matches(l, &term_lower))
            .take(limit)
            .collect()
    }

    /// Check if a location matches the search term.
    fn matches(&self, loc: &Location, term_lower: &str) -> bool {
        // Check code (case-insensitive)
        if let Some(code) = &loc.code {
            if code.to_lowercase().starts_with(term_lower) {
                return true;
            }
        }

        // Check name
        if let Some(name) = &loc.name {
            if name.to_lowercase().contains(term_lower) {
                return true;
            }
        }

        // Check city
        if let Some(city) = &loc.city {
            if city.to_lowercase().contains(term_lower) {
                return true;
            }
        }

        // Check country
        if let Some(country) = &loc.country {
            if country.to_lowercase().contains(term_lower) {
                return true;
            }
        }

        // Check slug
        if let Some(slug) = &loc.slug {
            if slug.contains(term_lower) {
                return true;
            }
        }

        false
    }

    /// Get location by exact code (e.g., "SFO").
    pub fn get_by_code(&self, code: &str) -> Option<&Location> {
        self.by_code
            .get(&code.to_uppercase())
            .map(|&i| &self.locations[i])
    }

    /// Get location by ID.
    pub fn get_by_id(&self, id: &str) -> Option<&Location> {
        self.by_id.get(id).map(|&i| &self.locations[i])
    }

    /// Total number of locations.
    pub fn len(&self) -> usize {
        self.locations.len()
    }

    /// Check if database is empty.
    pub fn is_empty(&self) -> bool {
        self.locations.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_by_code() {
        let db = LocationDb::instance();
        let results = db.search("SFO", 5);
        assert!(!results.is_empty());
        assert!(
            results[0].code.as_deref() == Some("SFO")
                || results[0]
                    .name
                    .as_ref()
                    .map(|n| n.contains("San Francisco"))
                    .unwrap_or(false)
        );
    }

    #[test]
    fn test_search_by_city() {
        let db = LocationDb::instance();
        let results = db.search("Tokyo", 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_get_by_code() {
        let db = LocationDb::instance();
        let loc = db.get_by_code("LAX");
        assert!(loc.is_some());
    }
}
