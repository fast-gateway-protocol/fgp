//! FGP service implementation for travel search.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Added response caching for all API methods (Claude)
//! 01/14/2026 - Initial implementation (Claude)

use anyhow::{Context, Result};
use chrono::NaiveDate;
use fgp_daemon::service::{HealthStatus, MethodInfo, ParamInfo};
use fgp_daemon::FgpService;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::api::flights::FlightsClient;
use crate::api::hotels::HotelsClient;
use crate::cache::TtlCache;
use crate::models::flight::{CabinClass, FlightSearchParams, SortBy};
use crate::models::hotel::HotelSearchParams;

/// FGP service for flight and hotel search.
pub struct TravelService {
    flights: Arc<FlightsClient>,
    hotels: Arc<HotelsClient>,
    cache: Arc<TtlCache<String, Value>>,
    runtime: Runtime,
}

impl TravelService {
    /// Create a new TravelService.
    pub fn new() -> Result<Self> {
        let flights = FlightsClient::new().context("Failed to create flights client")?;
        let hotels = HotelsClient::new().context("Failed to create hotels client")?;
        let runtime = Runtime::new().context("Failed to create tokio runtime")?;

        Ok(Self {
            flights: Arc::new(flights),
            hotels: Arc::new(hotels),
            cache: Arc::new(TtlCache::new(100, 300)), // 100 entries, 5 min TTL
            runtime,
        })
    }

    // ========================================================================
    // Helper functions
    // ========================================================================

    fn get_str<'a>(params: &'a HashMap<String, Value>, key: &str) -> Option<&'a str> {
        params.get(key).and_then(|v| v.as_str())
    }

    fn get_u32(params: &HashMap<String, Value>, key: &str, default: u32) -> u32 {
        params
            .get(key)
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(default)
    }

    fn get_u8(params: &HashMap<String, Value>, key: &str, default: u8) -> u8 {
        params
            .get(key)
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .unwrap_or(default)
    }

    fn get_f64(params: &HashMap<String, Value>, key: &str) -> Option<f64> {
        params.get(key).and_then(|v| v.as_f64())
    }

    fn get_date(params: &HashMap<String, Value>, key: &str) -> Option<NaiveDate> {
        params
            .get(key)
            .and_then(|v| v.as_str())
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
    }

    fn get_str_array(params: &HashMap<String, Value>, key: &str) -> Option<Vec<String>> {
        params.get(key).and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
    }

    /// Check cache and return cached value if present.
    fn cache_get(&self, key: &str) -> Option<Value> {
        self.cache.get(&key.to_string())
    }

    /// Store value in cache.
    fn cache_set(&self, key: &str, value: &Value) {
        self.cache.set(key.to_string(), value.clone());
    }

    // ========================================================================
    // Flight methods
    // ========================================================================

    fn find_location(&self, params: HashMap<String, Value>) -> Result<Value> {
        let term = Self::get_str(&params, "term")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: term"))?;
        let limit = Self::get_u32(&params, "limit", 10);
        let types = Self::get_str_array(&params, "types");

        // Check cache
        let cache_key = format!("loc:{}:{}:{:?}", term, limit, types);
        if let Some(cached) = self.cache_get(&cache_key) {
            tracing::debug!("Cache HIT for location search: {}", term);
            return Ok(cached);
        }

        let client = self.flights.clone();
        let term = term.to_string();

        let locations = self
            .runtime
            .block_on(async move { client.find_location(&term, types, limit).await })?;

        let result = json!({
            "locations": locations,
            "count": locations.len(),
        });

        // Store in cache
        self.cache_set(&cache_key, &result);
        tracing::debug!("Cache MISS for location search, stored: {}", cache_key);

        Ok(result)
    }

    fn search_flights(&self, params: HashMap<String, Value>) -> Result<Value> {
        let search_params = self.parse_flight_params(&params)?;

        // Check cache
        let cache_key = format!(
            "flights:{}:{}:{}:{}:{}",
            search_params.origin,
            search_params.destination,
            search_params.departure_from,
            search_params.adults,
            search_params.limit
        );
        if let Some(cached) = self.cache_get(&cache_key) {
            tracing::debug!("Cache HIT for flight search");
            return Ok(cached);
        }

        let client = self.flights.clone();
        let flights = self
            .runtime
            .block_on(async move { client.search_flights(&search_params).await })?;

        let result = json!({
            "flights": flights,
            "count": flights.len(),
        });

        self.cache_set(&cache_key, &result);
        tracing::debug!("Cache MISS for flight search, stored");

        Ok(result)
    }

    fn search_roundtrip(&self, params: HashMap<String, Value>) -> Result<Value> {
        let search_params = self.parse_flight_params(&params)?;

        // Check cache
        let cache_key = format!(
            "roundtrip:{}:{}:{}:{}:{}:{}",
            search_params.origin,
            search_params.destination,
            search_params.departure_from,
            search_params.return_from.map(|d| d.to_string()).unwrap_or_default(),
            search_params.adults,
            search_params.limit
        );
        if let Some(cached) = self.cache_get(&cache_key) {
            tracing::debug!("Cache HIT for roundtrip search");
            return Ok(cached);
        }

        let client = self.flights.clone();
        let roundtrips = self
            .runtime
            .block_on(async move { client.search_roundtrip(&search_params).await })?;

        let result = json!({
            "roundtrips": roundtrips,
            "count": roundtrips.len(),
        });

        self.cache_set(&cache_key, &result);
        tracing::debug!("Cache MISS for roundtrip search, stored");

        Ok(result)
    }

    fn parse_flight_params(&self, params: &HashMap<String, Value>) -> Result<FlightSearchParams> {
        let origin = Self::get_str(params, "origin")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: origin"))?
            .to_string();

        let destination = Self::get_str(params, "destination")
            .unwrap_or("anywhere")
            .to_string();

        let departure_from = Self::get_date(params, "departure_from")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: departure_from"))?;

        let departure_to = Self::get_date(params, "departure_to").unwrap_or(departure_from);

        let return_from = Self::get_date(params, "return_from");
        let return_to = Self::get_date(params, "return_to");

        let adults = Self::get_u8(params, "adults", 1);
        let children = Self::get_u8(params, "children", 0);
        let infants = Self::get_u8(params, "infants", 0);

        let cabin_class = Self::get_str(params, "cabin_class")
            .map(|s| match s.to_uppercase().as_str() {
                "ECONOMY" => CabinClass::Economy,
                "PREMIUM_ECONOMY" => CabinClass::PremiumEconomy,
                "BUSINESS" => CabinClass::Business,
                "FIRST" => CabinClass::First,
                _ => CabinClass::Economy,
            })
            .unwrap_or(CabinClass::Economy);

        let max_stops = params
            .get("max_stops")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8);

        let sort_by = Self::get_str(params, "sort_by")
            .map(|s| match s.to_uppercase().as_str() {
                "PRICE" => SortBy::Price,
                "QUALITY" => SortBy::Quality,
                "DURATION" => SortBy::Duration,
                "POPULARITY" => SortBy::Popularity,
                _ => SortBy::Price,
            })
            .unwrap_or(SortBy::Price);

        let limit = Self::get_u32(params, "limit", 10);
        let max_price = Self::get_f64(params, "max_price");
        let min_price = Self::get_f64(params, "min_price");

        Ok(FlightSearchParams {
            origin,
            destination,
            departure_from,
            departure_to,
            return_from,
            return_to,
            adults,
            children,
            infants,
            cabin_class,
            max_stops,
            sort_by,
            limit,
            max_price,
            min_price,
        })
    }

    // ========================================================================
    // Hotel methods
    // ========================================================================

    fn search_hotels(&self, params: HashMap<String, Value>) -> Result<Value> {
        let location = Self::get_str(&params, "location")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: location"))?
            .to_string();

        let limit = Self::get_u32(&params, "limit", 30);
        let offset = Self::get_u32(&params, "offset", 0);

        // Check cache (without filters - filters are applied client-side)
        let cache_key = format!("hotels:{}:{}:{}", location, limit, offset);
        if let Some(cached) = self.cache_get(&cache_key) {
            tracing::debug!("Cache HIT for hotel search");
            return Ok(cached);
        }

        let search_params = HotelSearchParams {
            location,
            limit,
            offset,
            min_price: Self::get_f64(&params, "min_price"),
            max_price: Self::get_f64(&params, "max_price"),
            min_rating: Self::get_f64(&params, "min_rating"),
            accommodation_types: Self::get_str_array(&params, "types"),
        };

        let client = self.hotels.clone();
        let results = self
            .runtime
            .block_on(async move { client.search_hotels(&search_params).await })?;

        let result = json!(results);
        self.cache_set(&cache_key, &result);
        tracing::debug!("Cache MISS for hotel search, stored");

        Ok(result)
    }

    fn get_hotel_rates(&self, params: HashMap<String, Value>) -> Result<Value> {
        let hotel_key = Self::get_str(&params, "hotel_key")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: hotel_key"))?
            .to_string();

        let check_in = Self::get_date(&params, "check_in")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: check_in"))?;

        let check_out = Self::get_date(&params, "check_out")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: check_out"))?;

        let rooms = Self::get_u8(&params, "rooms", 1);
        let adults = Self::get_u8(&params, "adults", 2);
        let currency = Self::get_str(&params, "currency").unwrap_or("USD").to_string();

        // Check cache (shorter TTL for rates - prices change frequently)
        let cache_key = format!(
            "rates:{}:{}:{}:{}:{}:{}",
            hotel_key, check_in, check_out, rooms, adults, currency
        );
        if let Some(cached) = self.cache_get(&cache_key) {
            tracing::debug!("Cache HIT for hotel rates");
            return Ok(cached);
        }

        let client = self.hotels.clone();
        let rates = self.runtime.block_on(async move {
            client
                .get_rates(&hotel_key, check_in, check_out, rooms, adults, &currency)
                .await
        })?;

        let result = json!(rates);
        self.cache_set(&cache_key, &result);
        tracing::debug!("Cache MISS for hotel rates, stored");

        Ok(result)
    }

    // ========================================================================
    // Cache methods
    // ========================================================================

    fn cache_stats(&self) -> Result<Value> {
        Ok(json!(self.cache.stats()))
    }

    fn cache_clear(&self) -> Result<Value> {
        let count = self.cache.clear();
        Ok(json!({
            "cleared": count,
        }))
    }
}

impl FgpService for TravelService {
    fn name(&self) -> &str {
        "travel"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            // Flights
            "travel.find_location" | "find_location" => self.find_location(params),
            "travel.search_flights" | "search_flights" => self.search_flights(params),
            "travel.search_roundtrip" | "search_roundtrip" => self.search_roundtrip(params),

            // Hotels
            "travel.search_hotels" | "search_hotels" => self.search_hotels(params),
            "travel.hotel_rates" | "hotel_rates" => self.get_hotel_rates(params),

            // Cache
            "travel.cache_stats" | "cache_stats" => self.cache_stats(),
            "travel.cache_clear" | "cache_clear" => self.cache_clear(),

            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            // Flight methods
            MethodInfo {
                name: "travel.find_location".into(),
                description: "Search for airports, cities, or countries".into(),
                params: vec![
                    ParamInfo {
                        name: "term".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "limit".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(json!(10)),
                    },
                    ParamInfo {
                        name: "types".into(),
                        param_type: "array".into(),
                        required: false,
                        default: None,
                    },
                ],
            },
            MethodInfo {
                name: "travel.search_flights".into(),
                description: "Search for one-way flights".into(),
                params: vec![
                    ParamInfo {
                        name: "origin".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "destination".into(),
                        param_type: "string".into(),
                        required: false,
                        default: Some(json!("anywhere")),
                    },
                    ParamInfo {
                        name: "departure_from".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "departure_to".into(),
                        param_type: "string".into(),
                        required: false,
                        default: None,
                    },
                    ParamInfo {
                        name: "adults".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(json!(1)),
                    },
                    ParamInfo {
                        name: "cabin_class".into(),
                        param_type: "string".into(),
                        required: false,
                        default: Some(json!("ECONOMY")),
                    },
                    ParamInfo {
                        name: "max_stops".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: None,
                    },
                    ParamInfo {
                        name: "limit".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(json!(10)),
                    },
                    ParamInfo {
                        name: "max_price".into(),
                        param_type: "number".into(),
                        required: false,
                        default: None,
                    },
                ],
            },
            MethodInfo {
                name: "travel.search_roundtrip".into(),
                description: "Search for round-trip flights".into(),
                params: vec![
                    ParamInfo {
                        name: "origin".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "destination".into(),
                        param_type: "string".into(),
                        required: false,
                        default: Some(json!("anywhere")),
                    },
                    ParamInfo {
                        name: "departure_from".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "return_from".into(),
                        param_type: "string".into(),
                        required: false,
                        default: None,
                    },
                    ParamInfo {
                        name: "adults".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(json!(1)),
                    },
                    ParamInfo {
                        name: "limit".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(json!(10)),
                    },
                ],
            },
            // Hotel methods
            MethodInfo {
                name: "travel.search_hotels".into(),
                description: "Search for hotels in a location".into(),
                params: vec![
                    ParamInfo {
                        name: "location".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "limit".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(json!(30)),
                    },
                    ParamInfo {
                        name: "min_price".into(),
                        param_type: "number".into(),
                        required: false,
                        default: None,
                    },
                    ParamInfo {
                        name: "max_price".into(),
                        param_type: "number".into(),
                        required: false,
                        default: None,
                    },
                    ParamInfo {
                        name: "min_rating".into(),
                        param_type: "number".into(),
                        required: false,
                        default: None,
                    },
                ],
            },
            MethodInfo {
                name: "travel.hotel_rates".into(),
                description: "Get real-time rates for a specific hotel".into(),
                params: vec![
                    ParamInfo {
                        name: "hotel_key".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "check_in".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "check_out".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "rooms".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(json!(1)),
                    },
                    ParamInfo {
                        name: "adults".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(json!(2)),
                    },
                    ParamInfo {
                        name: "currency".into(),
                        param_type: "string".into(),
                        required: false,
                        default: Some(json!("USD")),
                    },
                ],
            },
            // Cache methods
            MethodInfo {
                name: "travel.cache_stats".into(),
                description: "Get cache statistics".into(),
                params: vec![],
            },
            MethodInfo {
                name: "travel.cache_clear".into(),
                description: "Clear the response cache".into(),
                params: vec![],
            },
        ]
    }

    fn on_start(&self) -> Result<()> {
        tracing::info!("TravelService starting, verifying API connections...");

        // Check flights API
        let flights = self.flights.clone();
        let flights_ok = self.runtime.block_on(async move { flights.ping().await });

        match flights_ok {
            Ok(true) => tracing::info!("Flights API (Kiwi/Skypicker) connection verified"),
            Ok(false) => tracing::warn!("Flights API returned empty results"),
            Err(e) => tracing::warn!("Failed to connect to Flights API: {}", e),
        }

        // Check hotels API
        let hotels = self.hotels.clone();
        let hotels_ok = self.runtime.block_on(async move { hotels.ping().await });

        match hotels_ok {
            Ok(true) => tracing::info!("Hotels API (Xotelo) connection verified"),
            Ok(false) => tracing::warn!("Hotels API returned empty results"),
            Err(e) => tracing::warn!("Failed to connect to Hotels API: {}", e),
        }

        Ok(())
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        // Check Skypicker API
        let flights = self.flights.clone();
        let start = std::time::Instant::now();
        let flights_result = self.runtime.block_on(async move { flights.ping().await });
        let latency = start.elapsed().as_secs_f64() * 1000.0;

        match flights_result {
            Ok(true) => {
                checks.insert(
                    "skypicker_api".into(),
                    HealthStatus::healthy_with_latency(latency),
                );
            }
            Ok(false) => {
                checks.insert(
                    "skypicker_api".into(),
                    HealthStatus::unhealthy("Empty response"),
                );
            }
            Err(e) => {
                checks.insert("skypicker_api".into(), HealthStatus::unhealthy(e.to_string()));
            }
        }

        // Check Xotelo API
        let hotels = self.hotels.clone();
        let start = std::time::Instant::now();
        let hotels_result = self.runtime.block_on(async move { hotels.ping().await });
        let latency = start.elapsed().as_secs_f64() * 1000.0;

        match hotels_result {
            Ok(true) => {
                checks.insert(
                    "xotelo_api".into(),
                    HealthStatus::healthy_with_latency(latency),
                );
            }
            Ok(false) => {
                checks.insert(
                    "xotelo_api".into(),
                    HealthStatus::unhealthy("Empty response"),
                );
            }
            Err(e) => {
                checks.insert("xotelo_api".into(), HealthStatus::unhealthy(e.to_string()));
            }
        }

        checks
    }
}
