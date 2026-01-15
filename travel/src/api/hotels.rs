//! Hotels API client using Xotelo REST API.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/14/2026 - Fixed error check: API returns "error": null for success (Claude)
//! 01/14/2026 - Initial implementation (Claude)

use anyhow::{bail, Context, Result};
use chrono::NaiveDate;
use reqwest::Client;
use serde_json::Value;

use crate::models::hotel::*;
use crate::models::location::get_location_key;

const XOTELO_API: &str = "https://data.xotelo.com/api";

/// Client for Xotelo hotel search API.
pub struct HotelsClient {
    client: Client,
}

impl HotelsClient {
    /// Create a new hotels client.
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("fgp-travel/0.1.0 (Hotel Search)")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client })
    }

    /// Search for hotels in a location.
    pub async fn search_hotels(&self, params: &HotelSearchParams) -> Result<HotelSearchResults> {
        // Resolve location to key if city name
        let location_key = if params.location.starts_with('g') {
            params.location.clone()
        } else {
            get_location_key(&params.location)
                .ok_or_else(|| anyhow::anyhow!("Unknown location: {}", params.location))?
                .to_string()
        };

        let response = self
            .get(
                "/list",
                &[
                    ("location_key", location_key.as_str()),
                    ("limit", &params.limit.to_string()),
                    ("offset", &params.offset.to_string()),
                ],
            )
            .await?;

        tracing::debug!("Hotel search response keys: {:?}", response.as_object().map(|o| o.keys().collect::<Vec<_>>()));

        // Check for error - but "error": null means no error
        if let Some(error) = response.get("error") {
            if !error.is_null() {
                tracing::debug!("Hotel API error response: {:?}", error);
                bail!(
                    "Hotel search failed: {}",
                    error
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error")
                );
            }
        }

        let result = &response["result"];
        let hotels = self.parse_hotel_list(&result["list"])?;

        // Apply client-side filters
        let hotels: Vec<Hotel> = hotels
            .into_iter()
            .filter(|h| {
                params.min_price.map_or(true, |min| {
                    h.price_range.as_ref().map_or(false, |p| p.minimum >= min)
                }) && params.max_price.map_or(true, |max| {
                    h.price_range.as_ref().map_or(false, |p| p.maximum <= max)
                }) && params.min_rating.map_or(true, |min| {
                    h.review_summary.as_ref().map_or(false, |r| r.rating >= min)
                })
            })
            .collect();

        let total_count = result["total_count"].as_u64().unwrap_or(0) as u32;

        Ok(HotelSearchResults {
            location_key,
            total_count,
            hotels,
            offset: params.offset,
            limit: params.limit,
        })
    }

    /// Get real-time rates for a specific hotel.
    pub async fn get_rates(
        &self,
        hotel_key: &str,
        check_in: NaiveDate,
        check_out: NaiveDate,
        rooms: u8,
        adults: u8,
        currency: &str,
    ) -> Result<HotelRates> {
        let response = self
            .get(
                "/rates",
                &[
                    ("hotel_key", hotel_key),
                    ("chk_in", &check_in.to_string()),
                    ("chk_out", &check_out.to_string()),
                    ("rooms", &rooms.to_string()),
                    ("adults", &adults.to_string()),
                    ("currency", currency),
                ],
            )
            .await?;

        if let Some(error) = response.get("error") {
            if !error.is_null() {
                bail!(
                    "Rate lookup failed: {}",
                    error
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error")
                );
            }
        }

        let result = &response["result"];
        let rates = self.parse_rates(&result["rates"])?;

        Ok(HotelRates {
            hotel_key: hotel_key.to_string(),
            check_in,
            check_out,
            currency: result["currency"]
                .as_str()
                .unwrap_or(currency)
                .to_string(),
            rates,
        })
    }

    /// Check API health.
    pub async fn ping(&self) -> Result<bool> {
        // Try a simple list request for NYC
        let params = HotelSearchParams {
            location: "g60763".into(),
            limit: 1,
            ..Default::default()
        };
        let results = self.search_hotels(&params).await?;
        Ok(!results.hotels.is_empty())
    }

    // ========================================================================
    // Private helpers
    // ========================================================================

    async fn get(&self, endpoint: &str, params: &[(&str, &str)]) -> Result<Value> {
        let url = format!("{}{}", XOTELO_API, endpoint);

        tracing::debug!("Hotel API request: {} with params {:?}", url, params);

        let response = self
            .client
            .get(&url)
            .query(params)
            .send()
            .await
            .context("Failed to send REST request")?;

        tracing::debug!("Hotel API response status: {}", response.status());

        if !response.status().is_success() {
            bail!("REST request failed: {}", response.status());
        }

        let body = response.text().await.context("Failed to read response body")?;
        tracing::debug!("Hotel API response body (first 500 chars): {}", &body[..body.len().min(500)]);

        serde_json::from_str(&body).context("Failed to parse JSON response")
    }

    fn parse_hotel_list(&self, list: &Value) -> Result<Vec<Hotel>> {
        let hotels: Vec<Hotel> = list
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|data| self.parse_hotel(data).ok())
            .collect();

        Ok(hotels)
    }

    fn parse_hotel(&self, data: &Value) -> Result<Hotel> {
        let key = data["key"]
            .as_str()
            .context("Missing hotel key")?
            .to_string();

        let name = data["name"]
            .as_str()
            .context("Missing hotel name")?
            .to_string();

        let accommodation_type = data["accommodation_type"]
            .as_str()
            .unwrap_or("Hotel")
            .to_string();

        let url = data["url"].as_str().map(|s| s.to_string());
        let image_url = data["image"].as_str().map(|s| s.to_string());

        // Parse review_summary
        let review_summary = if let Some(review_data) = data.get("review_summary") {
            if let (Some(rating), Some(count)) = (
                review_data["rating"].as_f64(),
                review_data["count"].as_u64(),
            ) {
                Some(HotelReviewSummary {
                    rating,
                    count: count as u32,
                })
            } else {
                None
            }
        } else {
            None
        };

        // Parse price_ranges (note: plural in API response)
        let price_range = if let Some(price_data) = data.get("price_ranges") {
            if let (Some(min), Some(max)) = (
                price_data["minimum"].as_f64(),
                price_data["maximum"].as_f64(),
            ) {
                Some(HotelPriceRange {
                    minimum: min,
                    maximum: max,
                    currency: "USD".to_string(),
                })
            } else {
                None
            }
        } else {
            None
        };

        // Parse geo (API uses "geo" not "location")
        let location = if let Some(geo_data) = data.get("geo") {
            if let (Some(lat), Some(lng)) = (
                geo_data["latitude"].as_f64(),
                geo_data["longitude"].as_f64(),
            ) {
                Some(HotelLocation {
                    latitude: lat,
                    longitude: lng,
                })
            } else {
                None
            }
        } else {
            None
        };

        let mentions = data["mentions"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // Parse merchandising_labels (API uses this, not "labels")
        let labels = data["merchandising_labels"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(Hotel {
            key,
            name,
            accommodation_type,
            url,
            review_summary,
            price_range,
            location,
            image_url,
            mentions,
            labels,
        })
    }

    fn parse_rates(&self, rates_value: &Value) -> Result<Vec<HotelRate>> {
        let rates: Vec<HotelRate> = rates_value
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|r| self.parse_rate(r).ok())
            .collect();

        Ok(rates)
    }

    fn parse_rate(&self, data: &Value) -> Result<HotelRate> {
        // Provider can be "provider" or "vendor"
        let provider = data["provider"]
            .as_str()
            .or_else(|| data["vendor"].as_str())
            .unwrap_or("Unknown")
            .to_string();

        // Price can be "price" or "rate"
        let price = data["price"]
            .as_f64()
            .or_else(|| data["rate"].as_f64())
            .unwrap_or(0.0);

        let currency = data["currency"]
            .as_str()
            .unwrap_or("USD")
            .to_string();

        let room_type = data["room_type"].as_str().map(|s| s.to_string());
        let is_refundable = data["is_refundable"].as_bool();

        // URL can be "url" or "link"
        let url = data["url"]
            .as_str()
            .or_else(|| data["link"].as_str())
            .map(|s| s.to_string());

        Ok(HotelRate {
            provider,
            price,
            currency,
            room_type,
            is_refundable,
            url,
        })
    }
}
