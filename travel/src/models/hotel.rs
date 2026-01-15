//! Hotel data models.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/14/2026 - Initial implementation (Claude)

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Geographic location of a hotel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotelLocation {
    pub latitude: f64,
    pub longitude: f64,
}

/// Summary of hotel reviews.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotelReviewSummary {
    /// Average rating out of 5.
    pub rating: f64,
    /// Number of reviews.
    pub count: u32,
}

impl HotelReviewSummary {
    /// Human-readable rating label.
    pub fn rating_label(&self) -> &'static str {
        match self.rating {
            r if r >= 4.5 => "Excellent",
            r if r >= 4.0 => "Very Good",
            r if r >= 3.5 => "Good",
            r if r >= 3.0 => "Average",
            _ => "Below Average",
        }
    }
}

/// Price range for a hotel (min/max nightly rates).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotelPriceRange {
    pub minimum: f64,
    pub maximum: f64,
    pub currency: String,
}

impl HotelPriceRange {
    /// Approximate midpoint price.
    pub fn midpoint(&self) -> f64 {
        (self.minimum + self.maximum) / 2.0
    }
}

/// A hotel/accommodation from search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotel {
    /// Unique hotel key (e.g., "g60763-d23448880").
    pub key: String,
    pub name: String,
    /// Hotel, Hostel, Ryokan, etc.
    pub accommodation_type: String,
    pub url: Option<String>,
    pub review_summary: Option<HotelReviewSummary>,
    pub price_range: Option<HotelPriceRange>,
    pub location: Option<HotelLocation>,
    pub image_url: Option<String>,
    /// Tags like "Modern", "Business".
    #[serde(default)]
    pub mentions: Vec<String>,
    /// Badges like "Best seller".
    #[serde(default)]
    pub labels: Vec<String>,
}

impl Hotel {
    /// Shortcut to review rating.
    pub fn rating(&self) -> Option<f64> {
        self.review_summary.as_ref().map(|r| r.rating)
    }

    /// Shortcut to review count.
    pub fn review_count(&self) -> Option<u32> {
        self.review_summary.as_ref().map(|r| r.count)
    }

    /// Shortcut to minimum nightly price.
    pub fn min_price(&self) -> Option<f64> {
        self.price_range.as_ref().map(|p| p.minimum)
    }

    /// Extract TripAdvisor hotel ID from the key.
    pub fn tripadvisor_id(&self) -> Option<&str> {
        self.key.split("-d").nth(1)
    }

    /// Extract location ID from the key.
    pub fn location_id(&self) -> Option<&str> {
        if self.key.starts_with('g') {
            self.key.split('-').next()
        } else {
            None
        }
    }
}

/// A specific rate/price from an OTA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotelRate {
    /// OTA name like "Booking.com", "Expedia".
    pub provider: String,
    pub price: f64,
    pub currency: String,
    pub room_type: Option<String>,
    pub is_refundable: Option<bool>,
    pub url: Option<String>,
}

/// Collection of rates for a specific hotel stay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotelRates {
    pub hotel_key: String,
    pub check_in: NaiveDate,
    pub check_out: NaiveDate,
    pub currency: String,
    #[serde(default)]
    pub rates: Vec<HotelRate>,
}

impl HotelRates {
    /// Number of nights for this stay.
    pub fn nights(&self) -> i64 {
        (self.check_out - self.check_in).num_days()
    }

    /// Get the cheapest rate.
    pub fn cheapest(&self) -> Option<&HotelRate> {
        self.rates.iter().min_by(|a, b| {
            a.price
                .partial_cmp(&b.price)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Get the cheapest price.
    pub fn cheapest_price(&self) -> Option<f64> {
        self.cheapest().map(|r| r.price)
    }
}

/// Results from a hotel list/search query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotelSearchResults {
    pub location_key: String,
    pub total_count: u32,
    #[serde(default)]
    pub hotels: Vec<Hotel>,
    pub offset: u32,
    pub limit: u32,
}

impl HotelSearchResults {
    /// Check if there are more results available.
    pub fn has_more(&self) -> bool {
        self.offset + (self.hotels.len() as u32) < self.total_count
    }
}

/// Search parameters for hotel queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotelSearchParams {
    /// City name or location key.
    pub location: String,
    pub limit: u32,
    pub offset: u32,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub min_rating: Option<f64>,
    pub accommodation_types: Option<Vec<String>>,
}

impl Default for HotelSearchParams {
    fn default() -> Self {
        Self {
            location: String::new(),
            limit: 30,
            offset: 0,
            min_price: None,
            max_price: None,
            min_rating: None,
            accommodation_types: None,
        }
    }
}
