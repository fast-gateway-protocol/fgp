//! FGP service implementation for travel search.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Added search_cheapest_day for bulk parallel date search (Claude)
//! 01/15/2026 - Added local location database for instant lookups (Claude)
//! 01/15/2026 - Added response caching for all API methods (Claude)
//! 01/14/2026 - Initial implementation (Claude)

use anyhow::{Context, Result};
use chrono::{Duration, NaiveDate};
use fgp_daemon::service::{HealthStatus, MethodInfo, ParamInfo};
use fgp_daemon::FgpService;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::api::flights::FlightsClient;
use crate::api::hotels::HotelsClient;
use crate::cache::TtlCache;
use crate::locations::LocationDb;
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
        let limit = Self::get_u32(&params, "limit", 10) as usize;
        let types = Self::get_str_array(&params, "types");

        // Use local database - instant lookup, no API call needed
        let db = LocationDb::instance();
        let locations = db.search(term, limit);

        // Filter by type if specified
        let locations: Vec<_> = if let Some(ref type_filters) = types {
            locations
                .into_iter()
                .filter(|l| {
                    l.location_type
                        .as_ref()
                        .map(|t| type_filters.iter().any(|f| f.eq_ignore_ascii_case(t)))
                        .unwrap_or(false)
                })
                .collect()
        } else {
            locations
        };

        tracing::debug!(
            "Local location search '{}': {} results",
            term,
            locations.len()
        );

        Ok(json!({
            "locations": locations,
            "count": locations.len(),
            "source": "local",
        }))
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
            search_params
                .return_from
                .map(|d| d.to_string())
                .unwrap_or_default(),
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

    /// Search multiple dates in parallel to find the cheapest day to fly.
    fn search_cheapest_day(&self, params: HashMap<String, Value>) -> Result<Value> {
        let origin = Self::get_str(&params, "origin")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: origin"))?
            .to_string();

        let destination = Self::get_str(&params, "destination")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: destination"))?
            .to_string();

        let date_from = Self::get_date(&params, "date_from")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: date_from"))?;

        let date_to = Self::get_date(&params, "date_to")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: date_to"))?;

        let adults = Self::get_u8(&params, "adults", 1);
        let max_stops = params
            .get("max_stops")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8);

        // Generate all dates in range
        let mut dates = Vec::new();
        let mut current = date_from;
        while current <= date_to {
            dates.push(current);
            current += Duration::days(1);
        }

        if dates.is_empty() {
            anyhow::bail!("Invalid date range: date_from must be <= date_to");
        }

        if dates.len() > 62 {
            anyhow::bail!(
                "Date range too large: maximum 62 days (got {})",
                dates.len()
            );
        }

        tracing::info!(
            "Searching {} dates in parallel: {} to {}",
            dates.len(),
            date_from,
            date_to
        );

        // Search all dates in parallel
        let client = self.flights.clone();
        let cache = self.cache.clone();
        let origin_clone = origin.clone();
        let destination_clone = destination.clone();

        let results = self.runtime.block_on(async move {
            use futures::future::join_all;

            let tasks: Vec<_> = dates
                .into_iter()
                .map(|date| {
                    let client = client.clone();
                    let cache = cache.clone();
                    let origin = origin_clone.clone();
                    let destination = destination_clone.clone();

                    async move {
                        // Check cache first
                        let cache_key =
                            format!("flights:{}:{}:{}:{}:1", origin, destination, date, adults);
                        if let Some(cached) = cache.get(&cache_key) {
                            if let Some(flights) = cached.get("flights").and_then(|f| f.as_array())
                            {
                                if let Some(first) = flights.first() {
                                    let price = first.get("price").and_then(|p| p.as_f64());
                                    return (date, price, true);
                                }
                            }
                            return (date, None, true);
                        }

                        // Search for this date
                        let search_params = FlightSearchParams {
                            origin: origin.clone(),
                            destination: destination.clone(),
                            departure_from: date,
                            departure_to: date,
                            return_from: None,
                            return_to: None,
                            adults,
                            children: 0,
                            infants: 0,
                            cabin_class: CabinClass::Economy,
                            max_stops,
                            sort_by: SortBy::Price,
                            limit: 1, // Only need cheapest
                            max_price: None,
                            min_price: None,
                        };

                        match client.search_flights(&search_params).await {
                            Ok(flights) => {
                                let price = flights.first().map(|f| f.price);

                                // Cache the result
                                let result = json!({
                                    "flights": flights,
                                    "count": flights.len(),
                                });
                                cache.set(cache_key, result);

                                (date, price, false)
                            }
                            Err(e) => {
                                tracing::warn!("Failed to search date {}: {}", date, e);
                                (date, None, false)
                            }
                        }
                    }
                })
                .collect();

            join_all(tasks).await
        });

        // Find cheapest
        let mut day_prices: Vec<_> = results
            .iter()
            .filter_map(|(date, price, _)| price.map(|p| (date, p)))
            .collect();
        day_prices.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let cheapest = day_prices.first().map(|(date, price)| {
            json!({
                "date": date.to_string(),
                "price": price,
            })
        });

        let cache_hits = results.iter().filter(|(_, _, cached)| *cached).count();

        // Build price calendar
        let mut price_calendar: Vec<_> = results
            .iter()
            .map(|(date, price, cached)| {
                json!({
                    "date": date.to_string(),
                    "price": price,
                    "cached": cached,
                })
            })
            .collect();
        price_calendar.sort_by(|a, b| {
            a.get("date")
                .and_then(|d| d.as_str())
                .cmp(&b.get("date").and_then(|d| d.as_str()))
        });

        Ok(json!({
            "origin": origin,
            "destination": destination,
            "date_from": date_from.to_string(),
            "date_to": date_to.to_string(),
            "days_searched": results.len(),
            "cache_hits": cache_hits,
            "cheapest": cheapest,
            "price_calendar": price_calendar,
        }))
    }

    /// Search multiple destinations in parallel to find cheapest route.
    fn search_cheapest_route(&self, params: HashMap<String, Value>) -> Result<Value> {
        let origin = Self::get_str(&params, "origin")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: origin"))?
            .to_string();

        let destinations: Vec<String> = params
            .get("destinations")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: destinations (array)"))?;

        if destinations.is_empty() {
            anyhow::bail!("destinations array cannot be empty");
        }
        if destinations.len() > 20 {
            anyhow::bail!("Maximum 20 destinations allowed (got {})", destinations.len());
        }

        let date = Self::get_date(&params, "date")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: date"))?;

        let max_price = Self::get_f64(&params, "max_price");
        let adults = Self::get_u8(&params, "adults", 1);

        tracing::info!(
            "Searching {} destinations in parallel from {}",
            destinations.len(),
            origin
        );

        let client = self.flights.clone();
        let origin_clone = origin.clone();

        let results = self.runtime.block_on(async move {
            use futures::future::join_all;

            let tasks: Vec<_> = destinations
                .into_iter()
                .map(|dest| {
                    let client = client.clone();
                    let origin = origin_clone.clone();

                    async move {
                        let search_params = FlightSearchParams {
                            origin,
                            destination: dest.clone(),
                            departure_from: date,
                            departure_to: date,
                            return_from: None,
                            return_to: None,
                            adults,
                            children: 0,
                            infants: 0,
                            cabin_class: CabinClass::Economy,
                            max_stops: None,
                            sort_by: SortBy::Price,
                            limit: 1,
                            max_price: None,
                            min_price: None,
                        };

                        match client.search_flights(&search_params).await {
                            Ok(flights) => {
                                let price = flights.first().map(|f| f.price);
                                (dest, price, true)
                            }
                            Err(e) => {
                                tracing::warn!("Failed to search {}: {}", dest, e);
                                (dest, None, false)
                            }
                        }
                    }
                })
                .collect();

            join_all(tasks).await
        });

        // Filter by max_price if specified and sort by price
        let mut route_prices: Vec<_> = results
            .iter()
            .filter_map(|(dest, price, ok)| {
                if !ok {
                    return None;
                }
                price.and_then(|p| {
                    if max_price.map_or(true, |max| p <= max) {
                        Some((dest.clone(), p))
                    } else {
                        None
                    }
                })
            })
            .collect();
        route_prices.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let cheapest = route_prices.first().map(|(dest, price)| {
            json!({
                "destination": dest,
                "price": price,
            })
        });

        let routes: Vec<_> = route_prices
            .iter()
            .map(|(dest, price)| {
                json!({
                    "destination": dest,
                    "price": price,
                })
            })
            .collect();

        Ok(json!({
            "origin": origin,
            "date": date.to_string(),
            "destinations_searched": results.len(),
            "routes_found": routes.len(),
            "cheapest": cheapest,
            "routes": routes,
        }))
    }

    /// Search ±N days around a target date for flexibility.
    fn search_flexible_dates(&self, params: HashMap<String, Value>) -> Result<Value> {
        let origin = Self::get_str(&params, "origin")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: origin"))?
            .to_string();

        let destination = Self::get_str(&params, "destination")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: destination"))?
            .to_string();

        let target_date = Self::get_date(&params, "date")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: date"))?;

        let flexibility = Self::get_u32(&params, "flexibility", 3) as i64;
        if flexibility > 14 {
            anyhow::bail!("Maximum flexibility is 14 days (got {})", flexibility);
        }

        let adults = Self::get_u8(&params, "adults", 1);

        // Generate date range
        let date_from = target_date - Duration::days(flexibility);
        let date_to = target_date + Duration::days(flexibility);

        // Reuse search_cheapest_day logic
        let mut inner_params = HashMap::new();
        inner_params.insert("origin".to_string(), json!(origin));
        inner_params.insert("destination".to_string(), json!(destination));
        inner_params.insert("date_from".to_string(), json!(date_from.to_string()));
        inner_params.insert("date_to".to_string(), json!(date_to.to_string()));
        inner_params.insert("adults".to_string(), json!(adults));

        let mut result = self.search_cheapest_day(inner_params)?;

        // Add flexibility metadata
        if let Some(obj) = result.as_object_mut() {
            obj.insert("target_date".to_string(), json!(target_date.to_string()));
            obj.insert("flexibility_days".to_string(), json!(flexibility));
        }

        Ok(result)
    }

    /// Ultra-light price check - returns just price, no flight details.
    fn price_check(&self, params: HashMap<String, Value>) -> Result<Value> {
        let origin = Self::get_str(&params, "origin")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: origin"))?
            .to_string();

        let destination = Self::get_str(&params, "destination")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: destination"))?
            .to_string();

        let date = Self::get_date(&params, "date")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: date"))?;

        let adults = Self::get_u8(&params, "adults", 1);

        // Check cache first
        let cache_key = format!("price:{}:{}:{}:{}", origin, destination, date, adults);
        if let Some(cached) = self.cache_get(&cache_key) {
            return Ok(cached);
        }

        let search_params = FlightSearchParams {
            origin: origin.clone(),
            destination: destination.clone(),
            departure_from: date,
            departure_to: date,
            return_from: None,
            return_to: None,
            adults,
            children: 0,
            infants: 0,
            cabin_class: CabinClass::Economy,
            max_stops: None,
            sort_by: SortBy::Price,
            limit: 1,
            max_price: None,
            min_price: None,
        };

        let client = self.flights.clone();
        let flights = self
            .runtime
            .block_on(async move { client.search_flights(&search_params).await })?;

        let result = if let Some(flight) = flights.first() {
            json!({
                "origin": origin,
                "destination": destination,
                "date": date.to_string(),
                "price": flight.price,
                "currency": flight.currency,
                "stops": flight.stops,
                "available": true,
            })
        } else {
            json!({
                "origin": origin,
                "destination": destination,
                "date": date.to_string(),
                "available": false,
            })
        };

        self.cache_set(&cache_key, &result);
        Ok(result)
    }

    /// Search for direct (non-stop) flights only.
    fn search_direct_only(&self, params: HashMap<String, Value>) -> Result<Value> {
        let origin = Self::get_str(&params, "origin")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: origin"))?
            .to_string();

        let destination = Self::get_str(&params, "destination")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: destination"))?
            .to_string();

        let date = Self::get_date(&params, "date")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: date"))?;

        let adults = Self::get_u8(&params, "adults", 1);
        let limit = Self::get_u32(&params, "limit", 10);

        let search_params = FlightSearchParams {
            origin: origin.clone(),
            destination: destination.clone(),
            departure_from: date,
            departure_to: date,
            return_from: None,
            return_to: None,
            adults,
            children: 0,
            infants: 0,
            cabin_class: CabinClass::Economy,
            max_stops: Some(0), // Direct flights only
            sort_by: SortBy::Price,
            limit,
            max_price: None,
            min_price: None,
        };

        let client = self.flights.clone();
        let flights = self
            .runtime
            .block_on(async move { client.search_flights(&search_params).await })?;

        // Filter to only truly direct flights (single segment)
        let direct_flights: Vec<_> = flights
            .into_iter()
            .filter(|f| f.stops == 0 && f.segments.len() == 1)
            .collect();

        Ok(json!({
            "origin": origin,
            "destination": destination,
            "date": date.to_string(),
            "flights": direct_flights,
            "count": direct_flights.len(),
            "direct_only": true,
        }))
    }

    /// Execute multiple independent searches in one call.
    fn batch_search(&self, params: HashMap<String, Value>) -> Result<Value> {
        let searches: Vec<Value> = params
            .get("searches")
            .and_then(|v| v.as_array())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: searches (array)"))?;

        if searches.is_empty() {
            anyhow::bail!("searches array cannot be empty");
        }
        if searches.len() > 10 {
            anyhow::bail!("Maximum 10 searches per batch (got {})", searches.len());
        }

        let client = self.flights.clone();
        let cache = self.cache.clone();

        let results = self.runtime.block_on(async move {
            use futures::future::join_all;

            let tasks: Vec<_> = searches
                .into_iter()
                .enumerate()
                .map(|(idx, search)| {
                    let client = client.clone();
                    let cache = cache.clone();

                    async move {
                        let origin = search.get("origin").and_then(|v| v.as_str()).unwrap_or("");
                        let destination = search
                            .get("destination")
                            .and_then(|v| v.as_str())
                            .unwrap_or("anywhere");
                        let date_str = search.get("date").and_then(|v| v.as_str()).unwrap_or("");

                        if origin.is_empty() || date_str.is_empty() {
                            return json!({
                                "index": idx,
                                "ok": false,
                                "error": "Missing origin or date",
                            });
                        }

                        let date = match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                            Ok(d) => d,
                            Err(_) => {
                                return json!({
                                    "index": idx,
                                    "ok": false,
                                    "error": "Invalid date format",
                                });
                            }
                        };

                        // Check cache
                        let cache_key = format!("price:{}:{}:{}:1", origin, destination, date);
                        if let Some(cached) = cache.get(&cache_key) {
                            return json!({
                                "index": idx,
                                "ok": true,
                                "cached": true,
                                "result": cached,
                            });
                        }

                        let search_params = FlightSearchParams {
                            origin: origin.to_string(),
                            destination: destination.to_string(),
                            departure_from: date,
                            departure_to: date,
                            return_from: None,
                            return_to: None,
                            adults: 1,
                            children: 0,
                            infants: 0,
                            cabin_class: CabinClass::Economy,
                            max_stops: None,
                            sort_by: SortBy::Price,
                            limit: 1,
                            max_price: None,
                            min_price: None,
                        };

                        match client.search_flights(&search_params).await {
                            Ok(flights) => {
                                let result = if let Some(f) = flights.first() {
                                    json!({
                                        "origin": origin,
                                        "destination": destination,
                                        "date": date.to_string(),
                                        "price": f.price,
                                        "stops": f.stops,
                                        "available": true,
                                    })
                                } else {
                                    json!({
                                        "origin": origin,
                                        "destination": destination,
                                        "date": date.to_string(),
                                        "available": false,
                                    })
                                };

                                cache.set(cache_key, result.clone());

                                json!({
                                    "index": idx,
                                    "ok": true,
                                    "cached": false,
                                    "result": result,
                                })
                            }
                            Err(e) => {
                                json!({
                                    "index": idx,
                                    "ok": false,
                                    "error": e.to_string(),
                                })
                            }
                        }
                    }
                })
                .collect();

            join_all(tasks).await
        });

        let successful = results.iter().filter(|r| r.get("ok") == Some(&json!(true))).count();

        Ok(json!({
            "searches_requested": results.len(),
            "successful": successful,
            "results": results,
        }))
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
        let currency = Self::get_str(&params, "currency")
            .unwrap_or("USD")
            .to_string();

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
            // Flights - basic
            "travel.find_location" | "find_location" => self.find_location(params),
            "travel.search_flights" | "search_flights" => self.search_flights(params),
            "travel.search_roundtrip" | "search_roundtrip" => self.search_roundtrip(params),

            // Flights - efficiency methods
            "travel.search_cheapest_day" | "search_cheapest_day" => {
                self.search_cheapest_day(params)
            }
            "travel.search_cheapest_route" | "search_cheapest_route" => {
                self.search_cheapest_route(params)
            }
            "travel.search_flexible_dates" | "search_flexible_dates" => {
                self.search_flexible_dates(params)
            }
            "travel.price_check" | "price_check" => self.price_check(params),
            "travel.search_direct_only" | "search_direct_only" => self.search_direct_only(params),
            "travel.batch_search" | "batch_search" => self.batch_search(params),

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
            MethodInfo {
                name: "travel.search_cheapest_day".into(),
                description: "Search multiple dates in parallel to find the cheapest day to fly"
                    .into(),
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
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "date_from".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "date_to".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "adults".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(json!(1)),
                    },
                    ParamInfo {
                        name: "max_stops".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: None,
                    },
                ],
            },
            MethodInfo {
                name: "travel.search_cheapest_route".into(),
                description: "Search multiple destinations in parallel to find cheapest route"
                    .into(),
                params: vec![
                    ParamInfo {
                        name: "origin".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "destinations".into(),
                        param_type: "array".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "date".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "max_price".into(),
                        param_type: "number".into(),
                        required: false,
                        default: None,
                    },
                    ParamInfo {
                        name: "adults".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(json!(1)),
                    },
                ],
            },
            MethodInfo {
                name: "travel.search_flexible_dates".into(),
                description: "Search ±N days around target date for price flexibility".into(),
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
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "date".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "flexibility".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(json!(3)),
                    },
                    ParamInfo {
                        name: "adults".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(json!(1)),
                    },
                ],
            },
            MethodInfo {
                name: "travel.price_check".into(),
                description: "Ultra-light price check - returns just price, no flight details"
                    .into(),
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
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "date".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "adults".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(json!(1)),
                    },
                ],
            },
            MethodInfo {
                name: "travel.search_direct_only".into(),
                description: "Search for direct (non-stop) flights only".into(),
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
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "date".into(),
                        param_type: "string".into(),
                        required: true,
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
            MethodInfo {
                name: "travel.batch_search".into(),
                description: "Execute multiple independent searches in one call".into(),
                params: vec![ParamInfo {
                    name: "searches".into(),
                    param_type: "array".into(),
                    required: true,
                    default: None,
                }],
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
                checks.insert(
                    "skypicker_api".into(),
                    HealthStatus::unhealthy(e.to_string()),
                );
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
