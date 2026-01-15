# fgp-travel

FGP daemon for flight and hotel search via the **Kiwi/Skypicker GraphQL API** and the **Xotelo REST API**. It exposes FGP-compatible methods over a UNIX socket so agents can query travel options without paying repeated cold-start overhead.

## Features

- Flight search (one-way and round-trip)
- Hotel search + rate lookup
- Local airport/city lookup database for fast `find_location`
- Built-in response caching (5 minute TTL)

## Build

```bash
cargo build --release
```

## Running the daemon

```bash
./target/release/fgp-travel start
```

Foreground mode:

```bash
./target/release/fgp-travel start --foreground
```

Stop / status / health:

```bash
./target/release/fgp-travel stop
./target/release/fgp-travel status
./target/release/fgp-travel health
```

## Methods

| Method | Description |
|--------|-------------|
| `travel.find_location` | Search airports/cities by term |
| `travel.search_flights` | One-way flight search |
| `travel.search_roundtrip` | Round-trip flight search |
| `travel.search_hotels` | Hotel search |
| `travel.hotel_rates` | Hotel rate lookup |

## Example

```bash
echo '{"id":"1","v":1,"method":"travel.find_location","params":{"term":"SFO"}}' | nc -U ~/.fgp/services/travel/daemon.sock
```

## Notes

- The daemon uses public endpoints from Kiwi/Skypicker and Xotelo; no API keys are required.
- Socket path defaults to `~/.fgp/services/travel/daemon.sock` (override with `--socket`).
