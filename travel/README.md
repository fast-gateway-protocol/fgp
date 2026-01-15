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

| Method | Description |
|--------|-------------|
| `travel.find_location` | Search airports/cities (instant, local DB) |
| `travel.search_flights` | One-way flight search |
| `travel.search_roundtrip` | Round-trip flight search |
| `travel.search_cheapest_day` | Find cheapest day in date range (parallel) |
| `travel.search_hotels` | Hotel search by city |
| `travel.hotel_rates` | Real-time hotel rates |
| `travel.cache_stats` | Cache statistics |
| `travel.cache_clear` | Clear response cache |

## Token Usage

Estimated tokens per method call (for LLM context planning):

| Method | Tokens | Per Item | Notes |
|--------|--------|----------|-------|
| `find_location` | 90-700 | ~70/location | Instant (local DB) |
| `search_flights` | 500-3,500 | ~500/flight | Includes flight segments |
| `search_roundtrip` | 600-6,000 | ~600/trip | Outbound + return |
| `search_cheapest_day` | 170-500 | ~17/day | Very efficient for bulk |
| `search_hotels` | 750-1,500 | ~150/hotel | Basic hotel info |
| `hotel_rates` | 200-800 | ~100/rate | Real-time pricing |
| `cache_stats` | ~50 | - | Fixed size |

### Token Efficiency Tips

**Use `search_cheapest_day` for date flexibility:**
- 28 days = ~470 tokens (single call)
- vs. 28 × `search_flights` = ~14,000 tokens
- **30x more token-efficient**

**Use `limit` parameter wisely:**
- `limit: 1` for "just get me the cheapest"
- `limit: 5` for reasonable options
- `limit: 10` only when user needs many choices

**Leverage caching:**
- Repeated queries hit cache (~50 tokens)
- Cache TTL: 5 minutes

## Examples

### Find Cheapest Day to Fly

```bash
echo '{"id":"1","v":1,"method":"travel.search_cheapest_day","params":{
  "origin": "SFO",
  "destination": "BER",
  "date_from": "2026-02-01",
  "date_to": "2026-02-28"
}}' | nc -U ~/.fgp/services/travel/daemon.sock | jq '.result.cheapest'
```

Response (~470 tokens):
```json
{
  "cheapest": {"date": "2026-02-04", "price": 225.0},
  "days_searched": 28,
  "price_calendar": [...]
}
```

### Search Flights

```bash
echo '{"id":"1","v":1,"method":"travel.search_flights","params":{
  "origin": "SFO",
  "destination": "BER",
  "departure_from": "2026-02-15",
  "limit": 3
}}' | nc -U ~/.fgp/services/travel/daemon.sock | jq '.result.flights[0]'
```

### Search Hotels

```bash
echo '{"id":"1","v":1,"method":"travel.search_hotels","params":{
  "location": "Berlin",
  "limit": 5
}}' | nc -U ~/.fgp/services/travel/daemon.sock | jq '.result.hotels[].name'
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
| Flight search | 400-1000ms | Kiwi API |
| Hotel search | 300-700ms | Xotelo API |
| Cheapest day (28 days) | 1-2s | Parallel API calls |
| Cache hit | <1ms | In-memory LRU |

### Bulk Search Performance

`search_cheapest_day` searches dates in parallel:

| Days | Sequential | Parallel | Speedup |
|------|------------|----------|---------|
| 7 | ~4s | ~0.5s | 8x |
| 28 | ~16s | ~1.5s | 10x |
| 62 | ~35s | ~3s | 12x |

## Configuration

Socket: `~/.fgp/services/travel/daemon.sock`

Environment variables:
- `RUST_LOG=debug` - Enable debug logging
- `RUST_LOG=fgp_travel=debug` - Debug only this daemon

## API Sources

- **Flights**: [Kiwi/Skypicker GraphQL API](https://docs.kiwi.com/)
- **Hotels**: [Xotelo REST API](https://xotelo.com/)
