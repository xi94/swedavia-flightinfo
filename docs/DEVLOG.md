# Development log

## Problems & solutions – summary

| # | Problem | Solution |
|---|---|---|
| 1 | The original app was only available as a video, not as code | Went through the video frame by frame and wrote down menus, output and visible code |
| 2 | The original had its API key hard-coded and visible in the video | The key is read from a `.env` file that is never uploaded to GitHub |
| 3 | The JSON field names were unknown | Guessed them from the code in the video, then confirmed them with a real API answer |
| 4 | A library (`reqwest`) had renamed a setting, so installing it failed | Read the error message, which listed the new names, and picked the right one |
| 6 | The `/query` answer has a different shape than the other endpoints | Made a separate `QueryAnswer` struct for it |
| 7 | The API list is unsorted and includes "Deleted" flights | First sorted and hid them, then changed back because the original did not sort |
| 8 | The first version grew too big to explain in 5 minutes | Cut it down to a few files and removed everything the original did not have |
| 9 | One generic `flights()` function hid which endpoints exist | Split into four clear GET functions: `arrivals`, `departures`, `query`, `heartbeat` |
| 10 | The original's `city_country.json` was not available | Built it from the same source: Swedavia's destination statistics (Excel) |
| 11 | City names from the API did not match the Excel file ("London LHR" vs "London") | The lookup removes the airport code if the full name is not found |
| 12 | Two programs needed the same API code | A small `lib.rs` lets both programs share it |

The numbers match the sections below, which are in the order things happened (5 was a result, not a problem).

---

## Phase 1 – Research

### 1. The original app was only available as a video
*2026-11-23*
- **Problem:** There was no code to look at, only a screen recording of the app running.
- **Solution:** Went through the video frame by frame and wrote down the menus, the output format and the code that was visible (see PLAN.md §2).

### 2. The API key was hard-coded in the original
*2026-11-23*
- **Problem:** The original app has its API key written directly in the code, and it can be read in the video.
- **Solution:** Our version reads `SWEDAVIA_API_KEY` from a `.env` file, which is git-ignored. `.env.example` shows the format.

### 3. The JSON field names were unknown
*2026-11-23*
- **Problem:** The exact field names in the API answers could not be confirmed from the documentation alone.
- **Solution:** Guessed them from the Python code in the video (`flightId`, `scheduledUtc` ...) and made every field optional. Confirmed them later with a real API answer (see 5).

---

## Phase 2 – Implementation

### 4. Installing `reqwest` failed
*2026-11-24*
- **Problem:** `reqwest` 0.13 had renamed the `rustls-tls` feature, so `cargo add` failed.
- **Solution:** Read the list of available features in the error message and picked the right ones.

### 5. First call against the real API
*2026-11-24*
- **Result:** The guessed field names were all correct (also confirmed with the API portal's documentation), and `/heartbeat` answers `"IsAlive"`.
- **Solution:** Saved the real answer as `tests/fixtures/arrivals_arn_real.json`. A test checks that it can be read.

### 6. The `/query` answer has a different shape
*2026-11-24*
- **Problem:** Unlike arrivals and departures, each flight is wrapped in `{"arrival": ...}` or `{"departure": ...}`, and the answers come 25 at a time with a `continuationtoken`.
- **Solution:** Made a separate `QueryAnswer` struct. The filter `airport eq 'ARN' and flightType eq 'A' and scheduled eq '<date>'` works.

### 7. Unsorted list with "Deleted" flights
*2026-11-24*
- **Problem:** 46 of 279 arrivals at ARN had status "Deleted", and the list is not sorted by time.
- **Solution:** I first hid them and sorted the list. But the video shows the original did not sort (flight 50 at 16:25, flight 100 at 18:40, flight 150 at 17:40). Since the task is to rebuild the original, the list is now shown the same way as in the original.

### 8. The first version grew too big
*2026-11-24*
- **Problem:** The first version had 12 files, async code and extra commands that the original did not have. It would not be possible to explain in a 5-minute presentation.
- **Solution:** Cut it down to a few files that mirror the original (`main`, `api`, `input`, `output`), removed async (`tokio`) and all features the original did not have.

### 9. One generic function hid the endpoints
*2026-11-25*
- **Problem:** The client had one generic `flights()` function, which hid which endpoints exist.
- **Solution:** Split it into four clear GET functions, one per endpoint: `arrivals`, `departures`, `query`, `heartbeat`. Code comments were also changed to Swedish, and blank lines were added so the code is easier to read.

### 10. `city_country.json` was not available
*2026-11-25*
- **Problem:** The original used a `city_country.json` file that we did not have.
- **Solution:** Found the same source the original used: Swedavia's destination statistics (Excel) at swedavia.se → Om Swedavia → Statistik. Built the JSON from the 2025 file (column B = country, column C = city, sheets 1–2): 351 cities and 72 countries. The original had 263 / 59, most likely because it used an older year.

### 11. City names did not match
*2026-11-25*
- **Problem:** The API writes some cities with an airport code ("London LHR"), but the Excel file only has "London".
- **Solution:** The lookup first tries the full name, then the name without a 3-letter airport code.
- **Still open:** Names that differ by language, e.g. "Munich" (API) vs "München" (Excel), get no country. Left as it is, because the original's lookup has the same limit.

### 12. Two programs needed the same code
*2026-11-25*
- **Problem:** The destination overview needed the same API code as the menu.
- **Solution:** Added a small `lib.rs` so both programs (`swedavia` and `destinationer`) share `api`, `input` and `output`.

---

## Phase 3 – Completion

---

## Questions answered during the project
- **HeartBeat path:** `/heartbeat`, answers `"IsAlive"`
- **OData filter syntax:** `airport eq 'ARN' and flightType eq 'A' and scheduled eq 'YYYY-MM-DD'`
- **Baggage belt field:** `baggage.baggageClaimUnit`; remarks: `remarksEnglish[].text`
- **D/I field:** `diIndicator` (D = domestic, I = international, S = Schengen)
- **Source of `city_country.json`:** Swedavia's destination statistics (Excel)
