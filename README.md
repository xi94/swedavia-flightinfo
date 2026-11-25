# swedavia – Swedavia FlightInfo v2 in the terminal

A Rust rebuild of a Python flight-information app, made by **reverse engineering**
the original from a screen recording (APL project 2026, task 4).
It shows live arrivals and departures for Sweden's ten Swedavia airports.

See [docs/PLAN.md](docs/PLAN.md) for the plan and [docs/DEVLOG.md](docs/DEVLOG.md)
for the problems found along the way and how they were solved.

## Setup

1. Install Rust: <https://rustup.rs>
2. Get an API key at <https://apideveloper.swedavia.se> (subscribe to *FlightInfo v2*).
3. Copy `.env.example` to `.env` and put your key in it. `.env` is git-ignored.

## Run

```powershell
cargo run
```

```text
════════════════════════════════════════════════
  Swedavia FlightInfo API v2  –  Rust-klient
════════════════════════════════════════════════
  1. Ankomster (arrivals)  för en flygplats & datum
  2. Avgångar  (departures) för en flygplats & datum
  3. Sök specifikt flightnummer
  4. OData-förfrågan (query endpoint)
  5. HeartBeat – hälsokontroll av API
  6. Demonstrera alla endpoints automatiskt
  q. Avsluta
════════════════════════════════════════════════
```

The destination overview is a separate program, like `destinationer.py` in the original:

```powershell
cargo run --bin destinationer
```

```text
  Frankfurt                ( 9 st)  [Germany]
  Amsterdam                ( 9 st)  [Netherlands]
```

## Code

| File | What it does |
|---|---|
| `src/main.rs` | The menu loop and the 6 menu options |
| `src/bin/destinationer.rs` | Destination overview: unique cities, number of flights and country |
| `src/api.rs` | JSON structs and the 4 GET calls: `arrivals`, `departures`, `query`, `heartbeat` |
| `src/input.rs` | The airport list and asking for airport and date |
| `src/output.rs` | Time format (UTC → CET/CEST), ghost-entry filter, flight card, 50-per-page pager |
| `data/city_country.json` | City → country, built from Swedavia's destination statistics 2025 (`data/destinations-statistik-2025.xlsx`) |

Code comments are in Swedish.

## Tests

```powershell
cargo test
```

The tests use real API answers saved in `tests/fixtures/`, so they need no key and use no API quota.
