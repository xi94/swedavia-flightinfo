# Development log

## Problems & solutions – summary

| # | Problem | Solution |
|---|---|---|
| 1 | The `/query` answer has a different shape than the other endpoints | Made a separate `QueryAnswer` struct for it |
| 2 | The API list is unsorted and includes "Deleted" flights | First sorted and hid them, then changed back because the original did not sort |
| 3 | The first version grew too big to explain in 5 minutes | Cut it down to a few files and removed everything the original did not have |
| 4 | One generic `flights()` function hid which endpoints exist | Split into four clear GET functions: `arrivals`, `departures`, `query`, `heartbeat` |
| 5 | The original's `city_country.json` was not available | Built it from the same source: Swedavia's destination statistics (Excel) |
| 6 | City names from the API did not match the Excel file ("London LHR" vs "London") | The lookup removes the airport code if the full name is not found |

The numbers match the sections below, which are in the order things happened.

---

## Phase 2 – Implementation

### 1. The `/query` answer has a different shape
*2026-11-24*
- **Problem:** Unlike arrivals and departures, each flight is wrapped in `{"arrival": ...}` or `{"departure": ...}`, and the answers come 25 at a time with a `continuationtoken`.
- **Solution:** Made a separate `QueryAnswer` struct. The filter `airport eq 'ARN' and flightType eq 'A' and scheduled eq '<date>'` works.

### 2. Unsorted list with "Deleted" flights
*2026-11-24*
- **Problem:** 46 of 279 arrivals at ARN had status "Deleted", and the list is not sorted by time.
- **Solution:** I first hid them and sorted the list. But the video shows the original did not sort (flight 50 at 16:25, flight 100 at 18:40, flight 150 at 17:40). Since the task is to rebuild the original, the list is now shown the same way as in the original.

### 3. The first version grew too big
*2026-11-24*
- **Problem:** The first version had 12 files, async code and extra commands that the original did not have. It would not be possible to explain in a 5-minute presentation.
- **Solution:** Cut it down to a few files that mirror the original (`main`, `api`, `input`, `output`), removed async (`tokio`) and all features the original did not have.

### 4. One generic function hid the endpoints
*2026-11-25*
- **Problem:** The client had one generic `flights()` function, which hid which endpoints exist.
- **Solution:** Split it into four clear GET functions, one per endpoint: `arrivals`, `departures`, `query`, `heartbeat`. Code comments were also changed to Swedish, and blank lines were added so the code is easier to read.

### 5. `city_country.json` was not available
*2026-11-25*
- **Problem:** The original used a `city_country.json` file that we did not have.
- **Solution:** Found the same source the original used: Swedavia's destination statistics (Excel) at swedavia.se → Om Swedavia → Statistik. Built the JSON from the 2025 file (column B = country, column C = city, sheets 1–2): 351 cities and 72 countries. The original had 263 / 59, most likely because it used an older year.

### 6. City names did not match
*2026-11-25*
- **Problem:** The API writes some cities with an airport code ("London LHR"), but the Excel file only has "London".
- **Solution:** The lookup first tries the full name, then the name without a 3-letter airport code.
- **Still open:** Names that differ by language, e.g. "Munich" (API) vs "München" (Excel), get no country. Left as it is, because the original's lookup has the same limit.

---

## Phase 3 – Completion

---

## Questions answered during the project
- **HeartBeat path:** `/heartbeat`, answers `"IsAlive"`
- **OData filter syntax:** `airport eq 'ARN' and flightType eq 'A' and scheduled eq 'YYYY-MM-DD'`
- **Baggage belt field:** `baggage.baggageClaimUnit`; remarks: `remarksEnglish[].text`
- **D/I field:** `diIndicator` (D = domestic, I = international, S = Schengen)
- **Source of `city_country.json`:** Swedavia's destination statistics (Excel)
