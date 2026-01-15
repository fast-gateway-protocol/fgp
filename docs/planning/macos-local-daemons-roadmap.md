# macOS Local Daemons Roadmap

**Created:** 01/15/2026 03:00 AM PST
**Status:** Planning

## Overview

This document outlines the roadmap for building FGP daemons for macOS local services. The goal is to replicate the success of the iMessage daemon (480x speedup) across other macOS data sources.

## Proven Pattern

The iMessage daemon demonstrates the architecture:

```
┌─────────────┐     UNIX Socket      ┌──────────────────┐
│  AI Agent   │ ◄──────────────────► │  FGP Daemon      │
└─────────────┘                      │  (Rust)          │
                                     └────────┬─────────┘
                                              │
                   ┌──────────────────────────┼─────────────┐
                   │                          │             │
              ┌────▼────┐           ┌─────────▼────┐  ┌─────▼─────┐
              │ SQLite  │           │ Framework    │  │ AppleScript│
              │ Direct  │           │ APIs         │  │ Fallback   │
              └─────────┘           └──────────────┘  └────────────┘
```

**Key insight:** Local daemons are fastest because they eliminate both MCP cold-start (~2.3s) AND network latency.

---

## Tier 0: Quick Wins (Direct SQLite)

### 1. Safari Daemon ✅ IMPLEMENTED

**Database Locations:**
```
~/Library/Safari/History.db           # 27MB - Browser history
~/Library/Safari/CloudTabs.db         # 1.3MB - iCloud tabs
~/Library/Safari/Bookmarks.plist      # 274KB - Bookmarks (XML)
~/Library/Safari/Favicon Cache/       # Cached favicons
```

**Permissions:** Standard file access (NO Full Disk Access needed)

**Schema (History.db):**
- `history_items` - URLs with visit counts
- `history_visits` - Individual visits with timestamps

**Proposed Methods:**
| Method | Description | Params |
|--------|-------------|--------|
| `safari.history` | Search history | `query`, `days`, `limit` |
| `safari.recent` | Recent URLs | `limit` |
| `safari.top_sites` | Most visited | `days`, `limit` |
| `safari.bookmarks` | List bookmarks | `folder`, `query` |
| `safari.cloud_tabs` | Tabs from other devices | - |

**Estimated Speedup:** ~200x
**Estimated Time:** 1-2 weeks
**Risk:** Low

---

### 2. Contacts Daemon ✅ IMPLEMENTED

**Database Location:**
```
~/Library/Application Support/AddressBook/AddressBook-v22.abcddb  # 487KB
```

**Permissions:** Full Disk Access required

**Schema (key tables):**
- `ZABCDRECORD` - Contact records
- `ZABCDEMAILADDRESS` - Email addresses
- `ZABCDPHONENUMBER` - Phone numbers
- `ZABCDPOSTALADDRESS` - Physical addresses
- `ZABCDNOTE` - Contact notes

**Proposed Methods:**
| Method | Description | Params |
|--------|-------------|--------|
| `contacts.list` | All contacts | `limit` |
| `contacts.search` | Find by name | `query`, `limit` |
| `contacts.by_email` | Lookup by email | `email` |
| `contacts.by_phone` | Lookup by phone | `phone` |
| `contacts.groups` | List groups | - |
| `contacts.recent` | Recently modified | `days`, `limit` |

**Estimated Speedup:** ~150x
**Estimated Time:** 1 week
**Risk:** Low

**Note:** Currently iMessage daemon loads contacts from JSON sync. This would provide direct access.

---

## Tier 1: Medium Lift (SQLite + Processing)

### 3. Apple Notes Daemon ✅ IMPLEMENTED

**Status:** DONE - Full CLI with 8 query methods, ~12ms average, protobuf parsing for note content

**Database Location:**
```
~/Library/Group Containers/group.com.apple.notes/NoteStore.sqlite  # 4.3MB
```

**Permissions:** Full Disk Access required

**Challenge:** Note content is stored as gzipped protobuf in `ZICNOTEDATA.ZDATA` column.

**Schema (key tables):**
- `ZICCLOUDSYNCINGOBJECT` - Note metadata (title, dates, folder)
- `ZICNOTEDATA` - Note content (gzipped protobuf)
- `ZICLOCATION` - Location data

**Content Extraction:**
1. Read `ZDATA` blob from `ZICNOTEDATA`
2. Decompress with gzip
3. Parse protobuf (proprietary format, but reverse-engineered)

**Reference implementations:**
- [apple-notes-to-sqlite](https://github.com/dogsheep/apple-notes-to-sqlite) (Python)
- [notes-import](https://github.com/ChrLipp/notes-import) (Kotlin)

**Proposed Methods:**
| Method | Description | Params |
|--------|-------------|--------|
| `notes.list` | All notes with titles | `folder`, `limit` |
| `notes.search` | Full-text search | `query`, `limit` |
| `notes.recent` | Recently modified | `days`, `limit` |
| `notes.read` | Get note content | `id` |
| `notes.folders` | List folders | - |

**Estimated Speedup:** ~80x
**Estimated Time:** 2-3 weeks
**Risk:** Medium (protobuf parsing complexity)

---

### 4. Calendar Daemon (EventKit)

**Access Method:** EventKit framework via Rust bindings

**Rust Crate:** [objc2-event-kit](https://docs.rs/objc2-event-kit/)

**Permissions:** User authorization prompt (Privacy → Calendars)

**Proposed Methods:**
| Method | Description | Params |
|--------|-------------|--------|
| `calendar.events` | List events | `start`, `end`, `calendar` |
| `calendar.today` | Today's events | - |
| `calendar.upcoming` | Next N events | `days`, `limit` |
| `calendar.search` | Find by title | `query`, `days` |
| `calendar.free_slots` | Availability | `duration`, `days` |
| `calendar.calendars` | List calendars | - |

**Note:** FGP already has a `calendar` daemon (Google Calendar API). This would be for local Apple Calendar.

**Estimated Speedup:** ~200x (vs AppleScript)
**Estimated Time:** 2 weeks
**Risk:** Medium (Objective-C FFI)

---

### 5. Reminders Daemon (EventKit)

**Access Method:** EventKit framework via Rust bindings

**Rust Crate:** [objc2-event-kit](https://docs.rs/objc2-event-kit/)

**Permissions:** User authorization prompt (Privacy → Reminders)

**Proposed Methods:**
| Method | Description | Params |
|--------|-------------|--------|
| `reminders.list` | All reminders | `list`, `limit` |
| `reminders.incomplete` | Outstanding items | `limit` |
| `reminders.today` | Due today | - |
| `reminders.overdue` | Past due | `limit` |
| `reminders.by_list` | Filter by list | `list` |
| `reminders.complete` | Mark complete | `id` |
| `reminders.create` | Create reminder | `title`, `due`, `list` |

**Estimated Speedup:** ~150x
**Estimated Time:** 2 weeks (can share EventKit code with Calendar)
**Risk:** Medium

---

## Tier 2: Complex (Framework-Only)

### 6. Photos Daemon ✅ IMPLEMENTED

**Status:** DONE - Full CLI with 9 query methods, ~42ms average

**Database Location:**
```
~/Pictures/Photos Library.photoslibrary/database/Photos.sqlite  # ~35,358 photos in test
```

**Access Methods:**
1. **Direct SQLite** ✓ TESTED - Full access to ZASSET table with 100+ columns
2. **PhotoKit framework** - Rust crate exists: [objc2-photos](https://docs.rs/objc2-photos/)

**Key Tables (tested):**
- `ZASSET` - Main photo/video records (filename, date, location, dimensions)
- `ZADDITIONALASSETATTRIBUTES` - Extended metadata
- `ZDETECTEDFACE` - Face detection data
- `ZALBUMLIST` - Albums

**Sample Query (working):**
```sql
SELECT ZDATECREATED, ZFILENAME, ZDIRECTORY, ZLATITUDE, ZLONGITUDE
FROM ZASSET ORDER BY ZDATECREATED DESC LIMIT 5;
```

**Available Fields:**
- `ZFILENAME`, `ZDIRECTORY` - File location
- `ZDATECREATED`, `ZMODIFICATIONDATE` - Timestamps (Core Data format)
- `ZLATITUDE`, `ZLONGITUDE` - GPS coordinates
- `ZWIDTH`, `ZHEIGHT` - Dimensions
- `ZKIND` - Photo/Video type
- `ZFAVORITE`, `ZHIDDEN`, `ZTRASHEDSTATE` - Status flags

**Permissions:** Full Disk Access required (same as iMessage)

**Proposed Methods:**
| Method | Description | Params |
|--------|-------------|--------|
| `photos.recent` | Recent photos/videos | `days`, `limit`, `kind` |
| `photos.search` | Search by date/location | `start`, `end`, `lat`, `lon`, `radius` |
| `photos.albums` | List albums | - |
| `photos.by_album` | Photos in album | `album`, `limit` |
| `photos.favorites` | Favorited photos | `limit` |
| `photos.faces` | Photos with detected faces | `limit` |
| `photos.stats` | Library statistics | - |

**Estimated Speedup:** ~100x (direct SQLite vs PhotoKit overhead)
**Estimated Time:** 2-3 weeks
**Risk:** Medium (Core Data timestamp format, schema may change between macOS versions)

**Note:** Schema is complex but well-indexed. Direct SQLite is faster than PhotoKit for read operations.

---

### 7. Spotlight Daemon ✅ IMPLEMENTED

**Status:** DONE - Full CLI with 8 query methods, ~30ms average

**Access Methods:**
1. **[mdquery-rs](https://docs.rs/crate/mdquery-rs/latest)** ✓ Rust crate with native bindings
2. **mdfind CLI** ✓ TESTED - 60-385ms per query (already fast!)
3. **NSMetadataQuery** - Via objc2-foundation

**Benchmark Results (tested):**
```
mdfind -onlyin ~ "kMDItemKind == 'PDF'"     → 63ms
mdfind 'kind:image date:today'              → 385ms
mdfind -count "kMDItemContentType == 'public.jpeg'" → 69ms
```

**Rust Crates Available:**
- [mdquery-rs](https://crates.io/crates/mdquery-rs) - High-level Spotlight API
- [objc2-core-spotlight](https://crates.io/crates/objc2-core-spotlight) - CoreSpotlight bindings

**mdquery-rs Features:**
- Builder pattern for query construction
- `name_like()`, `extension()`, `time()`, `is_app()` predicates
- Custom search scopes (home, computer, network)
- Returns path, display name, metadata attributes

**Proposed Methods:**
| Method | Description | Params |
|--------|-------------|--------|
| `spotlight.search` | Full-text search | `query`, `scope`, `limit` |
| `spotlight.files` | Find files by name | `name`, `extension`, `path` |
| `spotlight.kind` | Filter by content type | `kind`, `query` |
| `spotlight.recent` | Recently modified | `days`, `kind`, `limit` |
| `spotlight.apps` | Find applications | `name` |
| `spotlight.documents` | Find documents | `type`, `query` |

**Caching Strategy:**
- Cache installed apps list (refresh every 10 min)
- Cache recent files by type (refresh on FSEvents)
- Cache common query patterns

**Estimated Speedup:** ~50-100x (mdfind is already 60-400ms, daemon eliminates cold-start)
**Estimated Time:** 2 weeks
**Risk:** Low (mdquery-rs handles complexity)

**Note:** Unlike other daemons, Spotlight is already fast. Main benefit is eliminating MCP cold-start and providing caching for repeated queries.

---

### 8. System Info Cache Daemon ✅ IMPLEMENTED

**Status:** DONE - Full CLI with 10 query methods, TTL-based caching via moka

**Purpose:** Aggregate slow system queries with caching

**Data Sources:**
- `system_profiler` - Hardware info (~2-3s)
- `diskutil` - Disk info
- `networksetup` - Network config
- `launchctl` - Running services

**Proposed Methods:**
| Method | Description | Cache TTL |
|--------|-------------|-----------|
| `system.hardware` | CPU, RAM, etc. | 1 hour |
| `system.disk` | Disk usage | 5 min |
| `system.network` | WiFi, IP, etc. | 1 min |
| `system.apps` | Installed apps | 10 min |
| `system.processes` | Running processes | 30 sec |

**Estimated Speedup:** ~400x (with caching)
**Estimated Time:** 1-2 weeks
**Risk:** Low

---

## Permission Requirements Matrix

| Daemon | Full Disk Access | User Auth | Standard |
|--------|------------------|-----------|----------|
| Safari | - | - | ✓ |
| Contacts | ✓ | - | - |
| Notes | ✓ | - | - |
| Calendar | - | ✓ | - |
| Reminders | - | ✓ | - |
| Photos | ✓ or User Auth | ✓ | - |
| Spotlight | - | - | ✓ |
| System | - | - | ✓ |

---

## Implementation Priority (Updated After Research)

| Priority | Daemon | Effort | Impact | Rust Crates | Risk |
|----------|--------|--------|--------|-------------|------|
| **P0** | Safari ✅ | ~~1-2 weeks~~ DONE | High | rusqlite | Low |
| **P0** | Contacts ✅ | ~~1 week~~ DONE | High | rusqlite | Low |
| **P1** | Photos ✅ | ~~2-3 weeks~~ DONE | High | rusqlite | Medium |
| **P1** | Spotlight ✅ | ~~2 weeks~~ DONE | Medium | mdquery-rs | Low |
| **P1** | Notes ✅ | ~~2-3 weeks~~ DONE | Medium | rusqlite + prost | Medium |
| **P2** | Calendar | 2 weeks | Medium | objc2-event-kit | Medium |
| **P2** | Reminders | 2 weeks | Medium | objc2-event-kit | Medium |
| **P2** | System Cache ✅ | ~~1-2 weeks~~ DONE | Medium | moka (caching) | Low |

**Key changes after research:**
- Photos moved to P1 - Direct SQLite access confirmed working (35K+ photos accessible)
- Spotlight moved to P1 - mdquery-rs crate available, mdfind already fast (60-400ms)
- Calendar/Reminders moved to P2 - EventKit requires more complex FFI

---

## Expected Impact

| Daemon | Queries/Session | MCP Time | FGP Time | Savings |
|--------|-----------------|----------|----------|---------|
| iMessage | 5-10 | 11-23s | 50-100ms | ~200x |
| Safari | 3-5 | 7-12s | 30-60ms | ~200x |
| Contacts | 2-3 | 5-7s | 20-40ms | ~150x |
| Calendar | 2-4 | 5-9s | 25-50ms | ~200x |
| Notes | 2-3 | 5-7s | 60-90ms | ~80x |

**Aggregate:** 15-25 local queries per session → <500ms total (vs 35-60s MCP)

---

## Open Questions

1. ~~**Photos:** Is PhotoKit accessible from Rust, or do we need Swift bridge?~~ ✅ RESOLVED - Direct SQLite works
2. ~~**Spotlight:** Can we cache efficiently, or is the query space too large?~~ ✅ RESOLVED - mdquery-rs available
3. **Notes:** How stable is the protobuf format across macOS versions?
4. **Calendar/Reminders:** Can we share EventKit bindings between both?
5. **Photos timestamps:** Core Data timestamps use different epoch - need conversion

---

## Research Findings Summary

### Photos Database (Confirmed Accessible)
- Location: `~/Pictures/Photos Library.photoslibrary/database/Photos.sqlite`
- 35,358 photos in test library
- Direct SQLite access works with Full Disk Access
- Rich metadata: dates, GPS, dimensions, favorites, faces
- Rust crate: [objc2-photos](https://lib.rs/crates/objc2-photos) also available

### Spotlight (Confirmed Accessible)
- Rust crate: [mdquery-rs](https://docs.rs/crate/mdquery-rs/latest) - native Spotlight bindings
- mdfind CLI benchmarks: 60-385ms (already fast)
- Main benefit: eliminate MCP cold-start + caching
- Also: [objc2-core-spotlight](https://crates.io/crates/objc2-core-spotlight)

---

## Changelog

- 01/15/2026 07:29 AM PST - **System Cache daemon implemented** - Full CLI with hardware, disks, network, processes, apps, battery, stats, invalidate, cache, bundle methods. TTL-based caching via moka crate. Latencies: stats 15ms, disks 4ms, network 65ms, processes 45ms, bundle 321ms. No special permissions required.
- 01/15/2026 05:28 AM PST - **Notes daemon implemented** - Full CLI with list, recent, search, read, by_folder, pinned, folders, stats methods. ~12ms average latency. Parses gzipped protobuf for full note content. Requires Full Disk Access.
- 01/15/2026 04:38 AM PST - **Spotlight daemon implemented** - Full CLI with search, by_name, by_extension, by_kind, recent, apps, directories, by_size methods. ~30ms average latency. Uses mdquery-rs crate. No special permissions required.
- 01/15/2026 04:24 AM PST - **Photos daemon implemented** - Full CLI with recent, favorites, by-date, by-location, albums, album-photos, people, person-photos, stats methods. ~42ms average latency. 26K+ photos accessible. Requires Full Disk Access.
- 01/15/2026 04:05 AM PST - **Contacts daemon implemented** - Full CLI with list, search, by_email, by_phone, recent, stats methods. ~13ms cold-start (~180x faster). Requires Full Disk Access.
- 01/15/2026 03:49 AM PST - **Safari daemon implemented** - Full CLI with history, search, top_sites, stats, cloud_tabs, bundle methods. ~26ms cold-start. Ready for daemon mode testing.
- 01/15/2026 03:15 AM PST - Added detailed Photos and Spotlight research findings
- 01/15/2026 03:00 AM PST - Initial planning document created
