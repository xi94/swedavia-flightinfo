# Development log

Write one entry **every time** something goes wrong or a decision is made.
This becomes Phase 4 ("Problems & solutions") of the report and the script for the video.

Format: `date – phase – Problem → Solution`

## Phase 1 – Research

- 2026-11-23 – Research – **The original app is only available as a video, not as code.**
  → Went through the video frame by frame and wrote down menus, output format and visible code (see PLAN.md §2).
- 2026-11-23 – Research – **The API key is hard-coded in the original and visible in the video.**
  → Our version reads `SWEDAVIA_API_KEY` from `.env`, which is git-ignored. `.env.example` shows the format.
- 2026-11-23 – Research – **The exact JSON field names could not be confirmed from the documentation.**
  → Guessed them from the Python code in the video (`flightId`, `scheduledUtc` ...), made every field optional,
  and planned to compare with a real response. TODO: save a real response in `tests/fixtures/`.

## Phase 2 – Implementation

- 2026-11-24 – Implementation – **reqwest 0.13 had renamed the `rustls-tls` feature**, so `cargo add` failed.
  → Read the list of available features in the error message and picked the right ones.
- 2026-11-24 – Implementation – **First call against the real API.** The guessed field names were all correct
  (confirmed with the API portal's documentation), and `/heartbeat` answers `"IsAlive"`.
  → Saved the real answer as `tests/fixtures/arrivals_arn_real.json`; a test checks that it parses.
- 2026-11-24 – Implementation – **The `/query` answer has a different shape** than arrivals/departures:
  each flight is wrapped in `{"arrival": ...}` or `{"departure": ...}`, 25 at a time with a `continuationtoken`.
  → Made a separate `QueryAnswer` struct. The filter `airport eq 'ARN' and flightType eq 'A' and scheduled eq '<date>'` works.
- 2026-11-24 – Implementation – **46 of 279 arrivals at ARN had status "Deleted", and the list is not sorted by time.**
  → I first hid them and sorted the list, but the video shows the original did not sort
  (flight 50 at 16:25, flight 100 at 18:40, flight 150 at 17:40). Since the task is to rebuild the original,
  the list is shown the same way as the original.
- 2026-11-24 – Implementation – **The first version grew too big** (12 files, async code, extra commands that the original
  did not have). It would not be possible to explain in a 5-minute presentation.
  → Cut it down to 4 files that mirror the original (`main`, `api`, `input`, `output`), removed async (`tokio`) and
  all features the original did not have.
- 2026-11-25 – Implementation – **The client had one generic `flights()` function**, which hid which endpoints exist.
  → Split it into four clear GET functions, one per endpoint: `arrivals`, `departures`, `query`, `heartbeat`.
  Code comments were also changed to Swedish, and blank lines were added so the code is easier to read.
- 2026-11-25 – Implementation – **`city_country.json` from the original was not available.**
  → Found the same source the original used: Swedavia's destination statistics (Excel) at swedavia.se → Om Swedavia → Statistik.
  Built the JSON from the 2025 file (column B = country, column C = city, sheets 1–2):
  351 cities and 72 countries. The original had 263 / 59, most likely because it used an older year.
- 2026-11-25 – Implementation – **Cities from the API did not match the Excel file**: the API writes "London LHR", Excel only "London".
  → The lookup first tries the full name, then the name without a 3-letter airport code.
  Still unmatched: names that differ by language, e.g. "Munich" (API) vs "München" (Excel). Left as it is – the original's lookup has the same limit.
- 2026-11-25 – Implementation – **The destination overview needed the same API code as the menu.**
  → Added a small `lib.rs` so both programs (`swedavia` and `destinationer`) share `api`, `input` and `output`.

## Phase 3 – Completion

## Open questions
- [x] Path of the HeartBeat endpoint → `/heartbeat`, answers `"IsAlive"`
- [x] OData filter syntax for `/query` → `airport eq 'ARN' and flightType eq 'A' and scheduled eq 'YYYY-MM-DD'`
- [x] Baggage belt field → `baggage.baggageClaimUnit`; remarks → `remarksEnglish[].text`
- [x] D/I field → `diIndicator` (D = domestic, I = international, S = Schengen)
- [x] Where did the original get `city_country.json`? → Swedavia's destination statistics (Excel), see above
