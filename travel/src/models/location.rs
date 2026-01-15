//! Location models and lookup tables.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/14/2026 - Initial implementation (Claude)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

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

/// Pre-defined TripAdvisor location keys for major cities (hotels).
pub static LOCATION_KEYS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    // United States
    m.insert("new york", "g60763");
    m.insert("nyc", "g60763");
    m.insert("los angeles", "g32655");
    m.insert("la", "g32655");
    m.insert("san francisco", "g60713");
    m.insert("sf", "g60713");
    m.insert("chicago", "g35805");
    m.insert("las vegas", "g45963");
    m.insert("miami", "g34438");
    m.insert("seattle", "g60878");
    m.insert("boston", "g60745");
    m.insert("washington dc", "g28970");
    m.insert("dc", "g28970");
    m.insert("austin", "g30196");
    m.insert("denver", "g33388");
    m.insert("san diego", "g60750");
    m.insert("portland", "g52024");
    m.insert("phoenix", "g31310");
    m.insert("atlanta", "g60898");
    m.insert("nashville", "g55229");
    m.insert("new orleans", "g60864");
    m.insert("orlando", "g34515");
    m.insert("honolulu", "g60982");
    m.insert("hawaii", "g60982");
    // International
    m.insert("london", "g186338");
    m.insert("paris", "g187147");
    m.insert("tokyo", "g298184");
    m.insert("rome", "g187791");
    m.insert("barcelona", "g187497");
    m.insert("amsterdam", "g188590");
    m.insert("berlin", "g187323");
    m.insert("dubai", "g295424");
    m.insert("singapore", "g294265");
    m.insert("hong kong", "g294217");
    m.insert("bangkok", "g293916");
    m.insert("sydney", "g255060");
    m.insert("melbourne", "g255100");
    m.insert("toronto", "g155019");
    m.insert("vancouver", "g154943");
    m.insert("montreal", "g155032");
    m.insert("mexico city", "g150800");
    m.insert("cancun", "g150807");
    m.insert("lisbon", "g189158");
    m.insert("madrid", "g187514");
    m.insert("prague", "g274707");
    m.insert("vienna", "g190454");
    m.insert("dublin", "g186605");
    m.insert("edinburgh", "g186525");
    m.insert("florence", "g187895");
    m.insert("venice", "g187870");
    m.insert("milan", "g187849");
    m.insert("munich", "g187309");
    m.insert("zurich", "g188113");
    m.insert("copenhagen", "g189541");
    m.insert("stockholm", "g189852");
    m.insert("oslo", "g190479");
    m.insert("reykjavik", "g189970");
    m.insert("athens", "g189400");
    m.insert("istanbul", "g293974");
    m.insert("cairo", "g294201");
    m.insert("cape town", "g1722390");
    m.insert("marrakech", "g293734");
    m.insert("bali", "g294226");
    m.insert("seoul", "g294197");
    m.insert("taipei", "g293913");
    m.insert("kuala lumpur", "g298570");
    m.insert("buenos aires", "g312741");
    m.insert("rio de janeiro", "g303506");
    m.insert("lima", "g294316");
    m.insert("bogota", "g294074");
    m.insert("santiago", "g294305");
    // Japan
    m.insert("kyoto", "g298564");
    m.insert("osaka", "g298566");
    m
});

/// Get a TripAdvisor location key for a city name.
pub fn get_location_key(city: &str) -> Option<&'static str> {
    LOCATION_KEYS.get(city.to_lowercase().trim()).copied()
}
