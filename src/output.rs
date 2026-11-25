//! Filtrerar och skriver ut flyg – samma utseende som originalappen.

use chrono::DateTime;
use chrono_tz::Europe::Stockholm;

use crate::api::Flight;
use crate::input::ask;

const PAGE_SIZE: usize = 50;

/// "2026-02-25T10:05:00Z" → "10:05 UTC → 11:05 CET" (CEST på sommaren).
pub fn fmt_time(utc: Option<&str>) -> String {
    let Some(time) = utc.and_then(|s| DateTime::parse_from_rfc3339(s).ok()) else {
        return "–".to_string();
    };

    let utc_time = time.format("%H:%M");
    let swedish_time = time.with_timezone(&Stockholm).format("%H:%M %Z");

    format!("{utc_time} UTC → {swedish_time}")
}

/// Falskt för "spökposter" där alla viktiga fält är tomma.
pub fn is_valid(f: &Flight) -> bool {
    let has_id = f.flight_id.is_some();
    let has_place = f.departure_airport_english.is_some() || f.arrival_airport_english.is_some();
    let has_time = f.times().is_some_and(|t| t.scheduled_utc.is_some());

    has_id || has_place || has_time
}

/// Inte landat/lyft än = ingen faktisk tid.
pub fn is_upcoming(f: &Flight) -> bool {
    f.times().is_some_and(|t| t.actual_utc.is_none())
}

/// Visar texten, eller "N/A" om fältet saknas.
fn text(value: Option<&String>) -> &str {
    value.map_or("N/A", |s| s.as_str())
}

/// Skriver ut ett flyg som ett kort. `home` är flygplatsen vi frågade om.
pub fn print_flight(f: &Flight, index: usize, home: &str, direction: &str) {
    let airline = f.airline_operator.as_ref();
    let status = f.location_and_status.as_ref();
    let baggage = f.baggage.as_ref();
    let times = f.times();

    // Ankomst: från en annan stad till oss. Avgång: från oss till en annan stad.
    let home = format!("({home})");
    let (from, to) = if direction == "arrivals" {
        (text(f.departure_airport_english.as_ref()), home.as_str())
    } else {
        (home.as_str(), text(f.arrival_airport_english.as_ref()))
    };

    let id = text(f.flight_id.as_ref());
    let airline_name = text(airline.and_then(|a| a.name.as_ref()));
    let airline_code = text(airline.and_then(|a| a.iata.as_ref()));
    let state = text(status.and_then(|s| s.flight_leg_status_english.as_ref()));
    let terminal = text(status.and_then(|s| s.terminal.as_ref()));
    let gate = text(status.and_then(|s| s.gate.as_ref()));
    let belt = text(baggage.and_then(|b| b.baggage_claim_unit.as_ref()));
    let scheduled = fmt_time(times.and_then(|t| t.scheduled_utc.as_deref()));
    let estimated = fmt_time(times.and_then(|t| t.estimated_utc.as_deref()));
    let actual = fmt_time(times.and_then(|t| t.actual_utc.as_deref()));
    let di = text(f.di_indicator.as_ref());

    println!("[{index}] → {id}  |  {airline_name}  ({airline_code})");
    println!("   From    : {from}");
    println!("   To      : {to}");
    println!("   Status  : {state}");
    println!("   Terminal: {terminal}   Gate: {gate}");
    println!("   Baggage : {belt}");
    println!("   Sched   : {scheduled}");
    println!("   Est     : {estimated}");
    println!("   Actual  : {actual}");
    println!("   D/I     : {di}");

    for remark in f.remarks_english.iter().filter_map(|r| r.text.as_ref()) {
        println!("   Remarks : {remark}");
    }

    println!("──────────────────────────────────────────────");
}

/// Skriver ut 50 flyg åt gången med paus mellan sidorna.
pub fn print_paged(flights: &[&Flight], home: &str, direction: &str) {
    let total = flights.len();
    let mut show_all = false;

    for start in (0..total).step_by(PAGE_SIZE) {
        let end = (start + PAGE_SIZE).min(total);

        for (i, f) in flights[start..end].iter().enumerate() {
            print_flight(f, start + i + 1, home, direction);
        }

        // Sista sidan, eller användaren har valt "visa alla": ingen fråga.
        if end == total || show_all {
            continue;
        }

        println!("\n  ── Visar {end}/{total} ──  ({} kvar)", total - end);
        let answer = ask("  [Enter] nästa sida  |  [a] visa alla  |  [q] tillbaka till menyn: ");

        match answer.as_deref() {
            Some("a") => show_all = true,
            Some("q") | None => {
                println!("  Avbröt efter {end} flights.");
                return;
            }
            _ => {} // Enter = nästa sida
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vintertid_och_sommartid() {
        let winter = fmt_time(Some("2026-02-25T10:05:00Z"));
        let summer = fmt_time(Some("2026-07-01T10:05:00Z"));

        assert_eq!(winter, "10:05 UTC → 11:05 CET");
        assert_eq!(summer, "10:05 UTC → 12:05 CEST");
        assert_eq!(fmt_time(None), "–");
    }

    #[test]
    fn spokpost_ar_inte_giltig() {
        assert!(!is_valid(&Flight::default()));
    }
}
