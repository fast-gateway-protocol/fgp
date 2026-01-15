# FGP Travel Daemon

Fast flight and hotel search via Kiwi/Skypicker and Xotelo APIs.

## Quick Start

```bash
# Build
cargo build --release

# Start daemon
./target/release/fgp-travel start

# Test
echo '{"id":"1","v":1,"method":"travel.find_location","params":{"term":"SFO"}}' \
  | nc -U ~/.fgp/services/travel/daemon.sock
```

## Methods

### Standard Methods

| Method | Description |
|--------|-------------|
| `travel.find_location` | Search airports/cities (instant, local DB) |
| `travel.search_flights` | One-way flight search |
| `travel.search_roundtrip` | Round-trip flight search |
| `travel.search_hotels` | Hotel search by city |
| `travel.hotel_rates` | Real-time hotel rates |

### Efficiency Methods (Token-Optimized)

| Method | Description | Token Savings |
|--------|-------------|---------------|
| `travel.price_check` | Ultra-light price check (~55 tokens) | **10x** vs search_flights |
| `travel.search_cheapest_day` | Find cheapest day in date range | **30x** for month search |
| `travel.search_cheapest_route` | Find cheapest destination | **5x** vs N searches |
| `travel.search_flexible_dates` | Search ±N days around date | **7x** vs N searches |
| `travel.search_direct_only` | Non-stop flights only | **2-3x** (fewer segments) |
| `travel.batch_search` | Multiple searches in one call | Reduces API overhead |

### Utility Methods

| Method | Description |
|--------|-------------|
| `travel.cache_stats` | Cache statistics |
| `travel.cache_clear` | Clear response cache |

## Token Usage

### Standard Methods

| Method | Tokens | Per Item | Notes |
|--------|--------|----------|-------|
| `find_location` | 90-700 | ~70/location | Instant (local DB) |
| `search_flights` | 500-3,500 | ~500/flight | Includes flight segments |
| `search_roundtrip` | 600-6,000 | ~600/trip | Outbound + return |
| `search_hotels` | 750-1,500 | ~150/hotel | Basic hotel info |
| `hotel_rates` | 200-800 | ~100/rate | Real-time pricing |

### Efficiency Methods

| Method | Tokens | Comparison |
|--------|--------|------------|
| `price_check` | ~55 | vs ~500 for search_flights |
| `search_cheapest_day` (28 days) | ~470 | vs ~14,000 (28 × search_flights) |
| `search_cheapest_route` (5 dests) | ~110 | vs ~2,500 (5 × search_flights) |
| `search_flexible_dates` (±3 days) | ~185 | vs ~3,500 (7 × search_flights) |
| `search_direct_only` | ~200-500 | vs ~500-1500 (fewer segments) |
| `batch_search` (3 routes) | ~160 | Parallel execution |

### When to Use Each Method

| Use Case | Recommended Method |
|----------|-------------------|
| "What's the cheapest price?" | `price_check` |
| "Which day is cheapest?" | `search_cheapest_day` |
| "Where can I fly for <$X?" | `search_cheapest_route` |
| "Flexible on dates ±3 days" | `search_flexible_dates` |
| "Non-stop flights only" | `search_direct_only` |
| "Check multiple routes" | `batch_search` |
| "Show me flight options" | `search_flights` |

## Examples

### Price Check (Ultra-Light)

```bash
echo '{"id":"1","v":1,"method":"travel.price_check","params":{
  "origin": "SFO",
  "destination": "LAX",
  "date": "2026-02-15"
}}' | nc -U ~/.fgp/services/travel/daemon.sock
```

Response (~55 tokens):
```json
{"origin": "SFO", "destination": "LAX", "date": "2026-02-15", "price": 83.0, "stops": 0}
```

### Find Cheapest Destination

```bash
echo '{"id":"1","v":1,"method":"travel.search_cheapest_route","params":{
  "origin": "SFO",
  "destinations": ["LAX", "SEA", "DEN", "PHX", "LAS"],
  "date": "2026-02-15"
}}' | nc -U ~/.fgp/services/travel/daemon.sock | jq '.result.cheapest'
```

Response:
```json
{"destination": "PHX", "price": 50.0}
```

### Find Cheapest Day

```bash
echo '{"id":"1","v":1,"method":"travel.search_cheapest_day","params":{
  "origin": "SFO",
  "destination": "BER",
  "date_from": "2026-02-01",
  "date_to": "2026-02-28"
}}' | nc -U ~/.fgp/services/travel/daemon.sock | jq '.result.cheapest'
```

Response:
```json
{"date": "2026-02-04", "price": 225.0}
```

### Flexible Dates (±3 Days)

```bash
echo '{"id":"1","v":1,"method":"travel.search_flexible_dates","params":{
  "origin": "SFO",
  "destination": "BER",
  "date": "2026-02-15",
  "flexibility": 3
}}' | nc -U ~/.fgp/services/travel/daemon.sock | jq '.result.cheapest'
```

### Batch Search (Multiple Routes)

```bash
echo '{"id":"1","v":1,"method":"travel.batch_search","params":{
  "searches": [
    {"origin": "SFO", "destination": "LAX", "date": "2026-02-15"},
    {"origin": "SFO", "destination": "SEA", "date": "2026-02-15"},
    {"origin": "SFO", "destination": "BER", "date": "2026-02-15"}
  ]
}}' | nc -U ~/.fgp/services/travel/daemon.sock | jq '.result.results[].result.price'
```

### Direct Flights Only

```bash
echo '{"id":"1","v":1,"method":"travel.search_direct_only","params":{
  "origin": "SFO",
  "destination": "LAX",
  "date": "2026-02-15",
  "limit": 3
}}' | nc -U ~/.fgp/services/travel/daemon.sock | jq '.result.flights[].price'
```

## Local Data

The daemon embeds reference data for instant lookups:

| Data | Entries | Size | Purpose |
|------|---------|------|---------|
| Locations | 7,988 | 1.6 MB | Airport/city search |
| Hotel locations | 224 | 7 KB | TripAdvisor city codes |
| Airlines | 1,094 | 112 KB | IATA codes + names |
| Countries | 249 | 35 KB | ISO 3166 codes |

Location searches are instant (~1-10ms) with zero API calls.

## Performance

| Operation | Latency | Notes |
|-----------|---------|-------|
| Location search | 1-10ms | Local database |
| Price check | 400-600ms | Single API call |
| Flight search | 400-1000ms | Kiwi API |
| Hotel search | 300-700ms | Xotelo API |
| Cheapest day (28 days) | 1-2s | Parallel API calls |
| Cheapest route (5 dests) | 1-2s | Parallel API calls |
| Cache hit | <1ms | In-memory LRU |

### Parallel Search Performance

| Method | Items | Time | Speedup |
|--------|-------|------|---------|
| `search_cheapest_day` | 28 days | ~1.5s | 10x vs sequential |
| `search_cheapest_route` | 5 dests | ~1s | 5x vs sequential |
| `search_flexible_dates` | 7 days | ~0.5s | 7x vs sequential |
| `batch_search` | 3 routes | ~1s | 3x vs sequential |

## Configuration

Socket: `~/.fgp/services/travel/daemon.sock`

Environment variables:
- `RUST_LOG=debug` - Enable debug logging
- `RUST_LOG=fgp_travel=debug` - Debug only this daemon

## API Sources

- **Flights**: [Kiwi/Skypicker GraphQL API](https://docs.kiwi.com/)
- **Hotels**: [Xotelo REST API](https://xotelo.com/)
