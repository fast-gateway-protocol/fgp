//! Flight data models.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/14/2026 - Initial implementation (Claude)

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A single flight segment within an itinerary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub carrier: String,
    pub carrier_name: Option<String>,
    pub flight_number: Option<String>,
    pub departure_time: DateTime<Utc>,
    pub arrival_time: DateTime<Utc>,
    pub origin: String,
    pub origin_name: Option<String>,
    pub destination: String,
    pub destination_name: Option<String>,
    pub duration_minutes: u32,
    pub cabin_class: Option<String>,
}

/// A complete flight option (may have multiple segments/stops).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flight {
    pub id: String,
    pub price: f64,
    pub currency: String,
    pub departure_time: DateTime<Utc>,
    pub arrival_time: DateTime<Utc>,
    pub origin: String,
    pub origin_city: Option<String>,
    pub destination: String,
    pub destination_city: Option<String>,
    pub duration_minutes: u32,
    pub stops: u32,
    pub segments: Vec<Segment>,
    pub deep_link: Option<String>,
}

impl Flight {
    /// Format duration as "Xh Ym".
    pub fn duration_formatted(&self) -> String {
        let hours = self.duration_minutes / 60;
        let mins = self.duration_minutes % 60;
        format!("{}h {}m", hours, mins)
    }

    /// Get unique carrier names for this flight.
    pub fn carriers(&self) -> Vec<String> {
        let mut seen = HashSet::new();
        self.segments
            .iter()
            .filter_map(|s| s.carrier_name.as_ref().or(Some(&s.carrier)))
            .filter(|&name| seen.insert(name.clone()))
            .cloned()
            .collect()
    }

    /// Get layover airport codes.
    pub fn layover_airports(&self) -> Vec<String> {
        if self.segments.len() <= 1 {
            return vec![];
        }
        self.segments[..self.segments.len() - 1]
            .iter()
            .map(|s| s.destination.clone())
            .collect()
    }

    /// Human-readable stops label.
    pub fn stops_label(&self) -> String {
        match self.stops {
            0 => "Direct".into(),
            1 => "1 stop".into(),
            n => format!("{} stops", n),
        }
    }
}

/// A round-trip flight itinerary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundTrip {
    pub id: String,
    pub price: f64,
    pub currency: String,
    pub outbound: Flight,
    pub inbound: Flight,
    pub booking_url: Option<String>,
    pub checked_bag_price: Option<f64>,
    pub destination_country: Option<String>,
    pub destination_city: Option<String>,
}

impl RoundTrip {
    /// Number of days between outbound and inbound departure.
    pub fn trip_days(&self) -> i64 {
        (self.inbound.departure_time.date_naive() - self.outbound.departure_time.date_naive())
            .num_days()
    }

    /// Total price including one checked bag.
    pub fn price_with_bag(&self) -> f64 {
        self.price + self.checked_bag_price.unwrap_or(0.0)
    }

    /// Primary destination.
    pub fn destination(&self) -> &str {
        &self.outbound.destination
    }

    /// Origin airport.
    pub fn origin(&self) -> &str {
        &self.outbound.origin
    }
}

/// Search parameters for flight queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightSearchParams {
    pub origin: String,
    pub destination: String,
    pub departure_from: NaiveDate,
    pub departure_to: NaiveDate,
    pub return_from: Option<NaiveDate>,
    pub return_to: Option<NaiveDate>,
    pub adults: u8,
    pub children: u8,
    pub infants: u8,
    pub cabin_class: CabinClass,
    pub max_stops: Option<u8>,
    pub sort_by: SortBy,
    pub limit: u32,
    pub max_price: Option<f64>,
    pub min_price: Option<f64>,
}

impl Default for FlightSearchParams {
    fn default() -> Self {
        Self {
            origin: String::new(),
            destination: "anywhere".into(),
            departure_from: chrono::Utc::now().date_naive(),
            departure_to: chrono::Utc::now().date_naive(),
            return_from: None,
            return_to: None,
            adults: 1,
            children: 0,
            infants: 0,
            cabin_class: CabinClass::Economy,
            max_stops: None,
            sort_by: SortBy::Price,
            limit: 10,
            max_price: None,
            min_price: None,
        }
    }
}

/// Cabin class for flight search.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CabinClass {
    #[default]
    Economy,
    PremiumEconomy,
    Business,
    First,
}

impl CabinClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            CabinClass::Economy => "ECONOMY",
            CabinClass::PremiumEconomy => "PREMIUM_ECONOMY",
            CabinClass::Business => "BUSINESS",
            CabinClass::First => "FIRST",
        }
    }
}

/// Sort order for flight results.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SortBy {
    #[default]
    Price,
    Quality,
    Duration,
    Popularity,
}

impl SortBy {
    pub fn as_str(&self) -> &'static str {
        match self {
            SortBy::Price => "PRICE",
            SortBy::Quality => "QUALITY",
            SortBy::Duration => "DURATION",
            SortBy::Popularity => "POPULARITY",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn flight_helpers_format_duration_and_stops() {
        let segment = Segment {
            carrier: "AA".to_string(),
            carrier_name: Some("Airline".to_string()),
            flight_number: Some("AA1".to_string()),
            departure_time: Utc.with_ymd_and_hms(2026, 1, 1, 8, 0, 0).unwrap(),
            arrival_time: Utc.with_ymd_and_hms(2026, 1, 1, 10, 0, 0).unwrap(),
            origin: "SFO".to_string(),
            origin_name: None,
            destination: "LAX".to_string(),
            destination_name: None,
            duration_minutes: 120,
            cabin_class: None,
        };
        let flight = Flight {
            id: "f1".to_string(),
            price: 120.0,
            currency: "USD".to_string(),
            departure_time: segment.departure_time,
            arrival_time: segment.arrival_time,
            origin: "SFO".to_string(),
            origin_city: None,
            destination: "LAX".to_string(),
            destination_city: None,
            duration_minutes: 120,
            stops: 0,
            segments: vec![segment],
            deep_link: None,
        };

        assert_eq!(flight.duration_formatted(), "2h 0m");
        assert_eq!(flight.stops_label(), "Direct");
    }

    #[test]
    fn flight_carriers_and_layovers() {
        let segment1 = Segment {
            carrier: "AA".to_string(),
            carrier_name: Some("Airline".to_string()),
            flight_number: None,
            departure_time: Utc.with_ymd_and_hms(2026, 1, 1, 8, 0, 0).unwrap(),
            arrival_time: Utc.with_ymd_and_hms(2026, 1, 1, 10, 0, 0).unwrap(),
            origin: "SFO".to_string(),
            origin_name: None,
            destination: "DEN".to_string(),
            destination_name: None,
            duration_minutes: 120,
            cabin_class: None,
        };
        let segment2 = Segment {
            carrier: "UA".to_string(),
            carrier_name: None,
            flight_number: None,
            departure_time: Utc.with_ymd_and_hms(2026, 1, 1, 11, 0, 0).unwrap(),
            arrival_time: Utc.with_ymd_and_hms(2026, 1, 1, 13, 0, 0).unwrap(),
            origin: "DEN".to_string(),
            origin_name: None,
            destination: "LAX".to_string(),
            destination_name: None,
            duration_minutes: 120,
            cabin_class: None,
        };
        let flight = Flight {
            id: "f2".to_string(),
            price: 200.0,
            currency: "USD".to_string(),
            departure_time: segment1.departure_time,
            arrival_time: segment2.arrival_time,
            origin: "SFO".to_string(),
            origin_city: None,
            destination: "LAX".to_string(),
            destination_city: None,
            duration_minutes: 300,
            stops: 1,
            segments: vec![segment1, segment2],
            deep_link: None,
        };

        let carriers = flight.carriers();
        assert_eq!(carriers.len(), 2);
        assert!(carriers.contains(&"Airline".to_string()));
        assert!(carriers.contains(&"UA".to_string()));

        let layovers = flight.layover_airports();
        assert_eq!(layovers, vec!["DEN".to_string()]);
        assert_eq!(flight.stops_label(), "1 stop");
    }

    #[test]
    fn round_trip_helpers() {
        let outbound = Flight {
            id: "out".to_string(),
            price: 100.0,
            currency: "USD".to_string(),
            departure_time: Utc.with_ymd_and_hms(2026, 1, 1, 8, 0, 0).unwrap(),
            arrival_time: Utc.with_ymd_and_hms(2026, 1, 1, 10, 0, 0).unwrap(),
            origin: "SFO".to_string(),
            origin_city: None,
            destination: "LAX".to_string(),
            destination_city: None,
            duration_minutes: 120,
            stops: 0,
            segments: vec![],
            deep_link: None,
        };
        let inbound = Flight {
            id: "in".to_string(),
            price: 100.0,
            currency: "USD".to_string(),
            departure_time: Utc.with_ymd_and_hms(2026, 1, 5, 8, 0, 0).unwrap(),
            arrival_time: Utc.with_ymd_and_hms(2026, 1, 5, 10, 0, 0).unwrap(),
            origin: "LAX".to_string(),
            origin_city: None,
            destination: "SFO".to_string(),
            destination_city: None,
            duration_minutes: 120,
            stops: 0,
            segments: vec![],
            deep_link: None,
        };
        let trip = RoundTrip {
            id: "rt".to_string(),
            price: 200.0,
            currency: "USD".to_string(),
            outbound,
            inbound,
            booking_url: None,
            checked_bag_price: Some(30.0),
            destination_country: None,
            destination_city: None,
        };

        assert_eq!(trip.trip_days(), 4);
        assert_eq!(trip.price_with_bag(), 230.0);
        assert_eq!(trip.destination(), "LAX");
        assert_eq!(trip.origin(), "SFO");
    }

    #[test]
    fn cabin_class_and_sort_as_str() {
        assert_eq!(CabinClass::Economy.as_str(), "ECONOMY");
        assert_eq!(CabinClass::Business.as_str(), "BUSINESS");
        assert_eq!(SortBy::Price.as_str(), "PRICE");
        assert_eq!(SortBy::Duration.as_str(), "DURATION");
    }
}
