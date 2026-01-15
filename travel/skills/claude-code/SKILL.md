---
name: travel-fgp
description: Fast flight and hotel search via FGP daemon (Kiwi/Xotelo APIs)
tools: ["Bash"]
triggers:
  - "flight"
  - "flights"
  - "hotel"
  - "hotels"
  - "travel"
  - "airfare"
  - "book a trip"
  - "find flights"
  - "cheapest flight"
  - "airport"
---

# Travel FGP Skill

Fast flight and hotel search using the FGP daemon protocol. Token-optimized responses for efficient LLM interactions.

## Prerequisites

1. **FGP daemon running**: `fgp start travel` or build with `cargo build --release`
2. **No API keys required**: Uses public Kiwi/Skypicker and Xotelo APIs

## Available Methods

### Standard Methods

| Method | Description | Tokens |
|--------|-------------|--------|
| `travel.find_location` | Search airports/cities (instant, local DB) | 90-700 |
| `travel.search_flights` | One-way flight search | 500-3,500 |
| `travel.search_roundtrip` | Round-trip flight search | 600-6,000 |
| `travel.search_hotels` | Hotel search by city | 750-1,500 |
| `travel.hotel_rates` | Real-time hotel rates | 200-800 |

### Efficiency Methods (Token-Optimized)

| Method | Description | Token Savings |
|--------|-------------|---------------|
| `travel.price_check` | Ultra-light price check (~55 tokens) | **10x** vs search_flights |
| `travel.search_cheapest_day` | Find cheapest day in date range | **30x** for month search |
| `travel.search_cheapest_route` | Find cheapest destination | **5x** vs N searches |
| `travel.search_flexible_dates` | Search ±N days around date | **7x** vs N searches |
| `travel.search_direct_only` | Non-stop flights only | **2-3x** (fewer segments) |
| `travel.batch_search` | Multiple searches in one call | Reduces overhead |

---

## Method Details

### travel.find_location - Airport/City Search

Instant lookup from local database (7,988 locations). No API call required.

**Parameters:**
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `term` | string | Yes | - | Search term (airport code, city name) |
| `limit` | integer | No | 10 | Maximum results |

```bash
fgp call travel.find_location -p '{"term": "SFO"}'
fgp call travel.find_location -p '{"term": "San Francisco", "limit": 5}'
```

**Response:** (~70 tokens/location)
```json
{
  "locations": [
    {
      "id": "SFO",
      "name": "San Francisco International",
      "city": "San Francisco",
      "country": "US",
      "type": "airport"
    }
  ],
  "count": 1
}
```

---

### travel.price_check - Ultra-Light Price Check

Get just the price without flight details. **10x more token-efficient** than search_flights.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `origin` | string | Yes | Origin airport code |
| `destination` | string | Yes | Destination airport code |
| `date` | string | Yes | Departure date (YYYY-MM-DD) |

```bash
fgp call travel.price_check -p '{"origin": "SFO", "destination": "LAX", "date": "2026-02-15"}'
```

**Response:** (~55 tokens)
```json
{
  "origin": "SFO",
  "destination": "LAX",
  "date": "2026-02-15",
  "price": 83.0,
  "stops": 0
}
```

---

### travel.search_flights - One-Way Flight Search

Full flight search with airline, times, and segment details.

**Parameters:**
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `origin` | string | Yes | - | Origin airport code |
| `destination` | string | Yes | - | Destination airport code |
| `departure_from` | string | Yes | - | Departure date (YYYY-MM-DD) |
| `departure_to` | string | No | same as from | Date range end |
| `limit` | integer | No | 5 | Maximum results |

```bash
fgp call travel.search_flights -p '{"origin": "SFO", "destination": "BER", "departure_from": "2026-02-15", "limit": 5}'
```

**Response:** (~500 tokens/flight)
```json
{
  "flights": [
    {
      "id": "abc123",
      "price": 450.0,
      "currency": "USD",
      "departure": "2026-02-15T08:00:00",
      "arrival": "2026-02-16T06:30:00",
      "duration_hours": 14.5,
      "stops": 1,
      "airlines": ["United", "Lufthansa"],
      "segments": [...]
    }
  ],
  "count": 5
}
```

---

### travel.search_roundtrip - Round-Trip Flight Search

Search for outbound and return flights together.

**Parameters:**
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `origin` | string | Yes | - | Origin airport code |
| `destination` | string | Yes | - | Destination airport code |
| `departure_from` | string | Yes | - | Outbound date (YYYY-MM-DD) |
| `return_from` | string | Yes | - | Return date (YYYY-MM-DD) |
| `limit` | integer | No | 5 | Maximum results |

```bash
fgp call travel.search_roundtrip -p '{"origin": "SFO", "destination": "BER", "departure_from": "2026-02-15", "return_from": "2026-02-22", "limit": 5}'
```

---

### travel.search_cheapest_day - Find Cheapest Day

Search entire date range in parallel to find the cheapest travel day. **30x more efficient** than individual searches.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `origin` | string | Yes | Origin airport code |
| `destination` | string | Yes | Destination airport code |
| `date_from` | string | Yes | Start of date range |
| `date_to` | string | Yes | End of date range |

```bash
fgp call travel.search_cheapest_day -p '{"origin": "SFO", "destination": "BER", "date_from": "2026-02-01", "date_to": "2026-02-28"}'
```

**Response:** (~470 tokens for 28 days)
```json
{
  "cheapest": {
    "date": "2026-02-04",
    "price": 225.0
  },
  "prices": [
    {"date": "2026-02-01", "price": 350.0},
    {"date": "2026-02-02", "price": 280.0},
    ...
  ],
  "days_searched": 28
}
```

---

### travel.search_cheapest_route - Find Cheapest Destination

Compare prices to multiple destinations in parallel.

**Parameters:**
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `origin` | string | Yes | - | Origin airport code |
| `destinations` | array | Yes | - | List of destination codes (max 20) |
| `date` | string | Yes | - | Travel date |

```bash
fgp call travel.search_cheapest_route -p '{"origin": "SFO", "destinations": ["LAX", "SEA", "DEN", "PHX", "LAS"], "date": "2026-02-15"}'
```

**Response:** (~110 tokens for 5 destinations)
```json
{
  "cheapest": {
    "destination": "PHX",
    "price": 50.0
  },
  "routes": [
    {"destination": "PHX", "price": 50.0},
    {"destination": "LAS", "price": 65.0},
    {"destination": "LAX", "price": 83.0},
    {"destination": "SEA", "price": 120.0},
    {"destination": "DEN", "price": 145.0}
  ]
}
```

---

### travel.search_flexible_dates - Flexible Date Search

Search ±N days around a target date for price flexibility.

**Parameters:**
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `origin` | string | Yes | - | Origin airport code |
| `destination` | string | Yes | - | Destination airport code |
| `date` | string | Yes | - | Target date (YYYY-MM-DD) |
| `flexibility` | integer | No | 3 | Days before/after to search |

```bash
fgp call travel.search_flexible_dates -p '{"origin": "SFO", "destination": "BER", "date": "2026-02-15", "flexibility": 3}'
```

**Response:** (~185 tokens for ±3 days)
```json
{
  "cheapest": {
    "date": "2026-02-13",
    "price": 420.0
  },
  "prices": [
    {"date": "2026-02-12", "price": 480.0},
    {"date": "2026-02-13", "price": 420.0},
    {"date": "2026-02-14", "price": 450.0},
    {"date": "2026-02-15", "price": 490.0},
    {"date": "2026-02-16", "price": 460.0},
    {"date": "2026-02-17", "price": 440.0},
    {"date": "2026-02-18", "price": 510.0}
  ],
  "target_date": "2026-02-15",
  "flexibility": 3
}
```

---

### travel.search_direct_only - Non-Stop Flights

Search for direct (non-stop) flights only. Smaller response since no connecting segments.

**Parameters:**
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `origin` | string | Yes | - | Origin airport code |
| `destination` | string | Yes | - | Destination airport code |
| `date` | string | Yes | - | Travel date |
| `limit` | integer | No | 5 | Maximum results |

```bash
fgp call travel.search_direct_only -p '{"origin": "SFO", "destination": "LAX", "date": "2026-02-15", "limit": 3}'
```

---

### travel.batch_search - Multiple Searches in One Call

Execute multiple independent price checks in a single call.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `searches` | array | Yes | List of search objects (max 10) |

Each search object:
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `origin` | string | Yes | Origin airport code |
| `destination` | string | Yes | Destination airport code |
| `date` | string | Yes | Travel date |

```bash
fgp call travel.batch_search -p '{"searches": [
  {"origin": "SFO", "destination": "LAX", "date": "2026-02-15"},
  {"origin": "SFO", "destination": "SEA", "date": "2026-02-15"},
  {"origin": "SFO", "destination": "BER", "date": "2026-02-15"}
]}'
```

**Response:** (~160 tokens for 3 routes)
```json
{
  "results": [
    {"origin": "SFO", "destination": "LAX", "result": {"price": 83.0, "stops": 0}},
    {"origin": "SFO", "destination": "SEA", "result": {"price": 120.0, "stops": 0}},
    {"origin": "SFO", "destination": "BER", "result": {"price": 450.0, "stops": 1}}
  ],
  "count": 3
}
```

---

### travel.search_hotels - Hotel Search

Search hotels by city.

**Parameters:**
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `location` | string | Yes | - | City name |
| `limit` | integer | No | 10 | Maximum results |

```bash
fgp call travel.search_hotels -p '{"location": "Berlin", "limit": 5}'
```

---

### travel.hotel_rates - Hotel Rates

Get real-time rates for a specific hotel.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `hotel_key` | string | Yes | Hotel identifier from search |
| `checkin` | string | Yes | Check-in date (YYYY-MM-DD) |
| `checkout` | string | Yes | Check-out date (YYYY-MM-DD) |

```bash
fgp call travel.hotel_rates -p '{"hotel_key": "abc123", "checkin": "2026-02-15", "checkout": "2026-02-18"}'
```

---

## When to Use Each Method

| User Request | Best Method | Why |
|--------------|-------------|-----|
| "What's the cheapest price to LAX?" | `price_check` | 10x fewer tokens |
| "Which day in February is cheapest?" | `search_cheapest_day` | Parallel search, single response |
| "Where can I fly for under $100?" | `search_cheapest_route` | Compare multiple destinations |
| "I'm flexible ±3 days" | `search_flexible_dates` | Shows price calendar |
| "Non-stop flights only" | `search_direct_only` | Smaller response |
| "Check prices to 5 cities" | `batch_search` | One call, parallel execution |
| "Show me flight options" | `search_flights` | Full details needed |
| "Find airport code for..." | `find_location` | Instant, no API call |

## Performance

| Operation | Latency | Notes |
|-----------|---------|-------|
| Location search | 1-10ms | Local database (7,988 entries) |
| Price check | 400-600ms | Single API call |
| Flight search | 400-1000ms | Kiwi API |
| Hotel search | 300-700ms | Xotelo API |
| Cheapest day (28 days) | 1-2s | Parallel API calls |
| Cache hit | <1ms | In-memory LRU |

## Troubleshooting

| Issue | Check | Fix |
|-------|-------|-----|
| Daemon not running | `fgp status travel` | `fgp start travel` |
| Location not found | Try airport code | Use 3-letter IATA code |
| No flights returned | Check date format | Use YYYY-MM-DD |
| Hotel location not found | Check city spelling | 224 major cities supported |

## Example Workflows

### Plan a Trip

```bash
# Find airport codes
fgp call travel.find_location -p '{"term": "Berlin"}'

# Check cheapest day to travel
fgp call travel.search_cheapest_day -p '{"origin": "SFO", "destination": "BER", "date_from": "2026-02-01", "date_to": "2026-02-28"}'

# Get flight details for that day
fgp call travel.search_flights -p '{"origin": "SFO", "destination": "BER", "departure_from": "2026-02-04", "limit": 5}'

# Find hotels
fgp call travel.search_hotels -p '{"location": "Berlin", "limit": 10}'
```

### Budget Trip Search

```bash
# Find cheapest destination from multiple options
fgp call travel.search_cheapest_route -p '{"origin": "SFO", "destinations": ["LAX", "SEA", "PDX", "LAS", "PHX", "DEN", "SLC"], "date": "2026-02-15"}'

# Check if direct flights available
fgp call travel.search_direct_only -p '{"origin": "SFO", "destination": "PHX", "date": "2026-02-15"}'
```
