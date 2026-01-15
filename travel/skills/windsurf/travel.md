# Travel Workflow

Fast flight and hotel search via FGP daemon. Token-optimized responses for efficient LLM interactions.

## Available Methods

### Standard Methods

| Method | Description | Tokens |
|--------|-------------|--------|
| `travel.find_location` | Search airports/cities (instant) | 90-700 |
| `travel.search_flights` | One-way flight search | 500-3,500 |
| `travel.search_roundtrip` | Round-trip flight search | 600-6,000 |
| `travel.search_hotels` | Hotel search by city | 750-1,500 |
| `travel.hotel_rates` | Real-time hotel rates | 200-800 |

### Efficiency Methods (Token-Optimized)

| Method | Description | Savings |
|--------|-------------|---------|
| `travel.price_check` | Ultra-light price check | **10x** |
| `travel.search_cheapest_day` | Find cheapest day in range | **30x** |
| `travel.search_cheapest_route` | Find cheapest destination | **5x** |
| `travel.search_flexible_dates` | Search ±N days | **7x** |
| `travel.search_direct_only` | Non-stop flights only | **2-3x** |
| `travel.batch_search` | Multiple searches in one | Reduced overhead |

## Commands

### travel.find_location - Airport/City Search

Instant lookup from local database (7,988 locations).

**Parameters:**
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `term` | string | Yes | - | Search term |
| `limit` | integer | No | 10 | Maximum results |

```bash
fgp call travel.find_location -p '{"term": "SFO"}'
```

**Response:**
```json
{
  "locations": [{"id": "SFO", "name": "San Francisco International", "city": "San Francisco"}],
  "count": 1
}
```

---

### travel.price_check - Ultra-Light Price Check

**10x more efficient** than search_flights. Returns just price, no flight details.

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
{"origin": "SFO", "destination": "LAX", "date": "2026-02-15", "price": 83.0, "stops": 0}
```

---

### travel.search_flights - One-Way Flight Search

Full flight search with airline, times, and segment details.

**Parameters:**
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `origin` | string | Yes | - | Origin airport code |
| `destination` | string | Yes | - | Destination airport code |
| `departure_from` | string | Yes | - | Departure date |
| `limit` | integer | No | 5 | Maximum results |

```bash
fgp call travel.search_flights -p '{"origin": "SFO", "destination": "BER", "departure_from": "2026-02-15", "limit": 5}'
```

---

### travel.search_cheapest_day - Find Cheapest Day

Search entire date range in parallel. **30x more efficient** than individual searches.

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

**Response:**
```json
{
  "cheapest": {"date": "2026-02-04", "price": 225.0},
  "prices": [...],
  "days_searched": 28
}
```

---

### travel.search_cheapest_route - Find Cheapest Destination

Compare prices to multiple destinations in parallel.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `origin` | string | Yes | Origin airport code |
| `destinations` | array | Yes | List of destination codes (max 20) |
| `date` | string | Yes | Travel date |

```bash
fgp call travel.search_cheapest_route -p '{"origin": "SFO", "destinations": ["LAX", "SEA", "DEN", "PHX", "LAS"], "date": "2026-02-15"}'
```

**Response:**
```json
{
  "cheapest": {"destination": "PHX", "price": 50.0},
  "routes": [...]
}
```

---

### travel.search_flexible_dates - Flexible Date Search

Search ±N days around a target date.

**Parameters:**
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `origin` | string | Yes | - | Origin airport code |
| `destination` | string | Yes | - | Destination airport code |
| `date` | string | Yes | - | Target date |
| `flexibility` | integer | No | 3 | Days before/after |

```bash
fgp call travel.search_flexible_dates -p '{"origin": "SFO", "destination": "BER", "date": "2026-02-15", "flexibility": 3}'
```

---

### travel.search_direct_only - Non-Stop Flights

Search for direct flights only. Smaller response.

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

### travel.batch_search - Multiple Searches

Execute multiple price checks in a single call.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `searches` | array | Yes | List of search objects (max 10) |

```bash
fgp call travel.batch_search -p '{"searches": [
  {"origin": "SFO", "destination": "LAX", "date": "2026-02-15"},
  {"origin": "SFO", "destination": "SEA", "date": "2026-02-15"}
]}'
```

---

### travel.search_hotels - Hotel Search

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

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `hotel_key` | string | Yes | Hotel identifier |
| `checkin` | string | Yes | Check-in date |
| `checkout` | string | Yes | Check-out date |

```bash
fgp call travel.hotel_rates -p '{"hotel_key": "abc123", "checkin": "2026-02-15", "checkout": "2026-02-18"}'
```

## When to Use Each Method

| User Request | Best Method |
|--------------|-------------|
| "What's the cheapest price?" | `price_check` |
| "Which day is cheapest?" | `search_cheapest_day` |
| "Where can I fly for <$X?" | `search_cheapest_route` |
| "Flexible on dates" | `search_flexible_dates` |
| "Non-stop only" | `search_direct_only` |
| "Check multiple routes" | `batch_search` |
| "Show flight options" | `search_flights` |

## Workflow Steps

1. **User requests travel info**
2. **Choose most efficient method** (see table above)
3. **Run `fgp call travel.*` command**
4. **Parse JSON response**
5. **Present results to user**

## Troubleshooting

| Issue | Check | Fix |
|-------|-------|-----|
| Daemon not running | `fgp status travel` | `fgp start travel` |
| Location not found | Try IATA code | Use 3-letter airport code |
| No flights | Check date format | Use YYYY-MM-DD |

## Performance

- Location search: 1-10ms (local database)
- Price check: 400-600ms
- Flight search: 400-1000ms
- Hotel search: 300-700ms
- Cheapest day (28 days): 1-2s (parallel)
- Cache hit: <1ms
