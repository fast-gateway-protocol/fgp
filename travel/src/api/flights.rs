//! Flights API client using Kiwi/Skypicker GraphQL.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/14/2026 - Fixed price parsing: amount returned as string, needed string→f64 (Claude)
//! 01/14/2026 - Initial implementation (Claude)

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json::{json, Value};

use crate::api::graphql::*;
use crate::models::flight::*;
use crate::models::location::Location;

const SKYPICKER_GRAPHQL: &str = "https://api.skypicker.com/umbrella/v2/graphql";

/// Client for Kiwi/Skypicker flight search API.
pub struct FlightsClient {
    client: Client,
}

impl FlightsClient {
    /// Create a new flights client.
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("fgp-travel/0.1.0")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client })
    }

    /// Search for airport/city/country locations.
    pub async fn find_location(
        &self,
        term: &str,
        location_types: Option<Vec<String>>,
        limit: u32,
    ) -> Result<Vec<Location>> {
        let mut variables = json!({
            "search": {"term": term},
            "first": limit,
        });

        if let Some(types) = location_types {
            variables["filter"] = json!({"types": types});
        }

        let response = self.graphql(PLACES_QUERY, variables, None).await?;

        let places = response["data"]["places"]
            .as_object()
            .context("Invalid places response")?;

        if places.get("__typename").and_then(|v| v.as_str()) == Some("AppError") {
            bail!(
                "Location search failed: {}",
                places
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
            );
        }

        let empty_vec = vec![];
        let edges = places
            .get("edges")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);

        let locations: Vec<Location> = edges
            .iter()
            .filter_map(|edge| self.parse_location(&edge["node"]).ok())
            .collect();

        Ok(locations)
    }

    /// Search for one-way flights.
    pub async fn search_flights(&self, params: &FlightSearchParams) -> Result<Vec<Flight>> {
        let variables = self.build_oneway_variables(params);
        let response = self
            .graphql(
                ONEWAY_SEARCH_QUERY,
                variables,
                Some("SearchOneWayItinerariesQuery"),
            )
            .await?;

        self.parse_flight_results(&response)
    }

    /// Search for round-trip flights.
    pub async fn search_roundtrip(&self, params: &FlightSearchParams) -> Result<Vec<RoundTrip>> {
        let variables = self.build_roundtrip_variables(params);
        let response = self
            .graphql(
                ROUNDTRIP_SEARCH_QUERY,
                variables,
                Some("SearchReturnItinerariesQuery"),
            )
            .await?;

        self.parse_roundtrip_results(&response)
    }

    /// Check API health by attempting a simple location search.
    pub async fn ping(&self) -> Result<bool> {
        let locations = self.find_location("SFO", None, 1).await?;
        Ok(!locations.is_empty())
    }

    // ========================================================================
    // Private helpers
    // ========================================================================

    async fn graphql(
        &self,
        query: &str,
        variables: Value,
        feature_name: Option<&str>,
    ) -> Result<Value> {
        let mut url = SKYPICKER_GRAPHQL.to_string();
        if let Some(name) = feature_name {
            url = format!("{}?featureName={}", url, name);
        }

        let body = json!({
            "query": query,
            "variables": variables,
        });

        tracing::debug!("GraphQL request URL: {}", url);
        tracing::debug!("GraphQL variables: {}", serde_json::to_string_pretty(&variables).unwrap_or_default());

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send GraphQL request")?;

        if !response.status().is_success() {
            bail!("GraphQL request failed: {}", response.status());
        }

        let json_response: Value = response.json().await.context("Failed to parse JSON response")?;
        tracing::debug!("GraphQL response size: {} bytes", serde_json::to_string(&json_response).unwrap_or_default().len());

        Ok(json_response)
    }

    fn build_oneway_variables(&self, params: &FlightSearchParams) -> Value {
        let mut filter = json!({
            "allowChangeInboundDestination": true,
            "allowChangeInboundSource": true,
            "allowDifferentStationConnection": true,
            "enableSelfTransfer": true,
            "enableThrowAwayTicketing": true,
            "enableTrueHiddenCity": true,
            "transportTypes": ["FLIGHT"],
            "contentProviders": ["KIWI", "FRESH", "KAYAK"],
            "flightsApiLimit": params.limit,
            "limit": params.limit,
        });

        if let Some(max_stops) = params.max_stops {
            filter["maxStopsCount"] = json!(max_stops);
        }
        if let Some(max_price) = params.max_price {
            filter["price"] = json!({"end": max_price});
        }
        if let Some(min_price) = params.min_price {
            if filter.get("price").is_some() {
                filter["price"]["start"] = json!(min_price);
            } else {
                filter["price"] = json!({"start": min_price});
            }
        }

        json!({
            "search": {
                "itinerary": {
                    "source": {"ids": [&params.origin]},
                    "destination": {"ids": [&params.destination]},
                    "outboundDepartureDate": {
                        "start": format!("{}T00:00:00", params.departure_from),
                        "end": format!("{}T23:59:59", params.departure_to),
                    }
                },
                "passengers": {
                    "adults": params.adults,
                    "children": params.children,
                    "infants": params.infants,
                    "adultsHoldBags": 0,
                    "adultsHandBags": 0,
                    "childrenHoldBags": [],
                    "childrenHandBags": [],
                },
                "cabinClass": {
                    "cabinClass": params.cabin_class.as_str(),
                    "applyMixedClasses": false
                }
            },
            "filter": filter,
            "options": {
                "sortBy": params.sort_by.as_str(),
                "mergePriceDiffRule": "INCREASED",
                "currency": "USD",
                "locale": "en",
                "partner": "skypicker",
                "affilID": "skypicker",
                "storeSearch": false,
                "searchStrategy": "REDUCED",
            }
        })
    }

    fn build_roundtrip_variables(&self, params: &FlightSearchParams) -> Value {
        let return_from = params
            .return_from
            .unwrap_or(params.departure_from + chrono::Duration::days(7));
        let return_to = params.return_to.unwrap_or(return_from);

        let mut filter = json!({
            "allowChangeInboundDestination": true,
            "allowChangeInboundSource": true,
            "allowDifferentStationConnection": true,
            "enableSelfTransfer": true,
            "enableThrowAwayTicketing": true,
            "enableTrueHiddenCity": true,
            "transportTypes": ["FLIGHT"],
            "contentProviders": ["KIWI", "FRESH", "KAYAK"],
            "flightsApiLimit": params.limit,
            "limit": params.limit,
        });

        if let Some(max_stops) = params.max_stops {
            filter["maxStopsCount"] = json!(max_stops);
        }
        if let Some(max_price) = params.max_price {
            filter["price"] = json!({"end": max_price});
        }

        json!({
            "search": {
                "itinerary": {
                    "source": {"ids": [&params.origin]},
                    "destination": {"ids": [&params.destination]},
                    "outboundDepartureDate": {
                        "start": format!("{}T00:00:00", params.departure_from),
                        "end": format!("{}T23:59:59", params.departure_to),
                    },
                    "inboundDepartureDate": {
                        "start": format!("{}T00:00:00", return_from),
                        "end": format!("{}T23:59:59", return_to),
                    }
                },
                "passengers": {
                    "adults": params.adults,
                    "children": params.children,
                    "infants": params.infants,
                    "adultsHoldBags": 0,
                    "adultsHandBags": 0,
                    "childrenHoldBags": [],
                    "childrenHandBags": [],
                },
                "cabinClass": {
                    "cabinClass": params.cabin_class.as_str(),
                    "applyMixedClasses": false
                }
            },
            "filter": filter,
            "options": {
                "sortBy": params.sort_by.as_str(),
                "mergePriceDiffRule": "INCREASED",
                "currency": "USD",
                "locale": "en",
                "partner": "skypicker",
                "affilID": "skypicker",
                "storeSearch": false,
                "searchStrategy": "REDUCED",
            }
        })
    }

    fn parse_flight_results(&self, response: &Value) -> Result<Vec<Flight>> {
        tracing::debug!("Flight search response keys: {:?}", response.as_object().map(|o| o.keys().collect::<Vec<_>>()));

        let result = &response["data"]["onewayItineraries"];

        tracing::debug!("onewayItineraries typename: {:?}", result.get("__typename"));

        if result.get("__typename").and_then(|v| v.as_str()) == Some("AppError") {
            let error_msg = result
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            tracing::error!("Flight search API error: {}", error_msg);
            bail!("Flight search failed: {}", error_msg);
        }

        let empty_vec = vec![];
        let itineraries = result["itineraries"].as_array().unwrap_or(&empty_vec);

        tracing::debug!("Found {} raw itineraries in response", itineraries.len());

        let flights: Vec<Flight> = itineraries
            .iter()
            .filter_map(|itin| {
                match self.parse_oneway_itinerary(itin) {
                    Ok(f) => Some(f),
                    Err(e) => {
                        tracing::warn!("Failed to parse itinerary: {}", e);
                        None
                    }
                }
            })
            .collect();

        Ok(flights)
    }

    fn parse_roundtrip_results(&self, response: &Value) -> Result<Vec<RoundTrip>> {
        let result = &response["data"]["returnItineraries"];

        if result.get("__typename").and_then(|v| v.as_str()) == Some("AppError") {
            bail!(
                "Flight search failed: {}",
                result
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
            );
        }

        let empty_vec = vec![];
        let itineraries = result["itineraries"].as_array().unwrap_or(&empty_vec);

        let roundtrips: Vec<RoundTrip> = itineraries
            .iter()
            .filter_map(|itin| self.parse_roundtrip_itinerary(itin).ok())
            .collect();

        Ok(roundtrips)
    }

    fn parse_oneway_itinerary(&self, itin: &Value) -> Result<Flight> {
        // Debug: log the raw itinerary structure
        tracing::debug!("Raw itinerary keys: {:?}", itin.as_object().map(|o| o.keys().collect::<Vec<_>>()));
        tracing::debug!("Raw price value: {:?}", itin.get("price"));

        let id = itin["id"]
            .as_str()
            .context("Missing id")?
            .to_string();

        // Handle price - may be string or number
        let price = itin["price"]["amount"]
            .as_f64()
            .or_else(|| {
                // Try parsing from string
                itin["price"]["amount"]
                    .as_str()
                    .and_then(|s| s.parse::<f64>().ok())
            })
            .context("Missing price")?;

        let sector = &itin["sector"];
        let duration_minutes = sector["duration"]
            .as_u64()
            .map(|d| (d / 60) as u32)
            .unwrap_or(0);

        let segments = self.parse_segments(&sector["sectorSegments"])?;

        let (origin, origin_city) = if let Some(first) = segments.first() {
            (first.origin.clone(), first.origin_name.clone())
        } else {
            (String::new(), None)
        };

        let (destination, destination_city) = if let Some(last) = segments.last() {
            (last.destination.clone(), last.destination_name.clone())
        } else {
            (String::new(), None)
        };

        let (departure_time, arrival_time) = if let (Some(first), Some(last)) =
            (segments.first(), segments.last())
        {
            (first.departure_time, last.arrival_time)
        } else {
            (Utc::now(), Utc::now())
        };

        let stops = if segments.is_empty() {
            0
        } else {
            (segments.len() - 1) as u32
        };

        let deep_link = itin["bookingOptions"]["edges"]
            .as_array()
            .and_then(|e| e.first())
            .and_then(|e| e["node"]["bookingUrl"].as_str())
            .map(|s| s.to_string());

        Ok(Flight {
            id,
            price,
            currency: "USD".into(),
            departure_time,
            arrival_time,
            origin,
            origin_city,
            destination,
            destination_city,
            duration_minutes,
            stops,
            segments,
            deep_link,
        })
    }

    fn parse_roundtrip_itinerary(&self, itin: &Value) -> Result<RoundTrip> {
        let id = itin["id"]
            .as_str()
            .context("Missing id")?
            .to_string();
        let price = itin["price"]["amount"]
            .as_f64()
            .context("Missing price")?;

        let outbound = self.parse_sector_as_flight(&itin["outbound"], &id, "out")?;
        let inbound = self.parse_sector_as_flight(&itin["inbound"], &id, "in")?;

        // Extract checked bag price
        let checked_bag_price = itin["bagsInfo"]["checkedBagTiers"]
            .as_array()
            .and_then(|tiers| tiers.first())
            .and_then(|tier| tier["tierPrice"]["amount"].as_f64());

        // Extract destination country from outbound final segment
        let destination_country = outbound
            .segments
            .last()
            .and_then(|_| {
                itin["outbound"]["sectorSegments"]
                    .as_array()
                    .and_then(|segs| segs.last())
                    .and_then(|seg| {
                        seg["segment"]["destination"]["station"]["city"]["country"]["code"].as_str()
                    })
            })
            .map(|s| s.to_string());

        let destination_city = outbound
            .segments
            .last()
            .and_then(|_| {
                itin["outbound"]["sectorSegments"]
                    .as_array()
                    .and_then(|segs| segs.last())
                    .and_then(|seg| {
                        seg["segment"]["destination"]["station"]["city"]["name"].as_str()
                    })
            })
            .map(|s| s.to_string());

        let booking_url = itin["bookingOptions"]["edges"]
            .as_array()
            .and_then(|e| e.first())
            .and_then(|e| e["node"]["bookingUrl"].as_str())
            .map(|s| s.to_string());

        Ok(RoundTrip {
            id,
            price,
            currency: "USD".into(),
            outbound,
            inbound,
            booking_url,
            checked_bag_price,
            destination_country,
            destination_city,
        })
    }

    fn parse_sector_as_flight(&self, sector: &Value, base_id: &str, suffix: &str) -> Result<Flight> {
        let duration_minutes = sector["duration"]
            .as_u64()
            .map(|d| (d / 60) as u32)
            .unwrap_or(0);

        let segments = self.parse_segments(&sector["sectorSegments"])?;

        let (origin, origin_city) = if let Some(first) = segments.first() {
            (first.origin.clone(), first.origin_name.clone())
        } else {
            (String::new(), None)
        };

        let (destination, destination_city) = if let Some(last) = segments.last() {
            (last.destination.clone(), last.destination_name.clone())
        } else {
            (String::new(), None)
        };

        let (departure_time, arrival_time) = if let (Some(first), Some(last)) =
            (segments.first(), segments.last())
        {
            (first.departure_time, last.arrival_time)
        } else {
            (Utc::now(), Utc::now())
        };

        let stops = if segments.is_empty() {
            0
        } else {
            (segments.len() - 1) as u32
        };

        Ok(Flight {
            id: format!("{}-{}", base_id, suffix),
            price: 0.0, // Price is at itinerary level
            currency: "USD".into(),
            departure_time,
            arrival_time,
            origin,
            origin_city,
            destination,
            destination_city,
            duration_minutes,
            stops,
            segments,
            deep_link: None,
        })
    }

    fn parse_segments(&self, sector_segments: &Value) -> Result<Vec<Segment>> {
        let empty_vec = vec![];
        let segments_array = sector_segments.as_array().unwrap_or(&empty_vec);

        let segments: Vec<Segment> = segments_array
            .iter()
            .filter_map(|ss| self.parse_segment(&ss["segment"]).ok())
            .collect();

        Ok(segments)
    }

    fn parse_segment(&self, seg: &Value) -> Result<Segment> {
        let carrier = seg["carrier"]["code"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let carrier_name = seg["carrier"]["name"].as_str().map(|s| s.to_string());

        let origin = seg["source"]["station"]["code"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let origin_name = seg["source"]["station"]["name"]
            .as_str()
            .map(|s| s.to_string());

        let destination = seg["destination"]["station"]["code"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let destination_name = seg["destination"]["station"]["name"]
            .as_str()
            .map(|s| s.to_string());

        let departure_time = self.parse_datetime(&seg["source"]["localTime"])?;
        let arrival_time = self.parse_datetime(&seg["destination"]["localTime"])?;

        let duration_minutes = seg["duration"]
            .as_u64()
            .map(|d| (d / 60) as u32)
            .unwrap_or(0);

        Ok(Segment {
            carrier,
            carrier_name,
            flight_number: None,
            departure_time,
            arrival_time,
            origin,
            origin_name,
            destination,
            destination_name,
            duration_minutes,
            cabin_class: None,
        })
    }

    fn parse_datetime(&self, value: &Value) -> Result<DateTime<Utc>> {
        let s = value.as_str().context("Expected datetime string")?;
        // Parse ISO format like "2026-02-01T10:30:00"
        let dt = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
            .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f"))
            .context("Failed to parse datetime")?;
        Ok(DateTime::from_naive_utc_and_offset(dt, Utc))
    }

    fn parse_location(&self, node: &Value) -> Result<Location> {
        let id = node["id"]
            .as_str()
            .or_else(|| node["code"].as_str())
            .or_else(|| node["legacyId"].as_str())
            .context("Missing location id")?
            .to_string();

        let name = node["name"]
            .as_str()
            .context("Missing location name")?
            .to_string();

        let slug = node["slug"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let location_type = node["type"]
            .as_str()
            .or_else(|| node["__typename"].as_str())
            .unwrap_or("UNKNOWN")
            .to_string();

        let city = node["city"]["name"].as_str().map(|s| s.to_string());
        let country = node["city"]["country"]["name"]
            .as_str()
            .or_else(|| node["country"]["name"].as_str())
            .map(|s| s.to_string());
        let country_code = node["city"]["country"]["code"]
            .as_str()
            .or_else(|| node["country"]["code"].as_str())
            .map(|s| s.to_string());

        let latitude = node["gps"]["lat"].as_f64();
        let longitude = node["gps"]["lng"].as_f64();

        Ok(Location {
            id,
            name,
            slug,
            location_type,
            city,
            country,
            country_code,
            latitude,
            longitude,
        })
    }
}
