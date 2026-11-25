//! Destinationsöversikt – Rust-versionen av `destinationer.py`.
//! Hämtar ankomster eller avgångar och listar alla unika städer med land
//! och hur många flyg som gick dit. Kan filtreras på land eller stad.
//! Starta: `cargo run --bin destinationer`

use std::collections::HashMap;

use serde::Deserialize;

use swedavia::api::Client;
use swedavia::input::{ask, choose_airport, choose_date};
use swedavia::output::is_valid;

/// `data/city_country.json` byggdes från Swedavias destinationsstatistik (Excel).
/// Filen bakas in i programmet, så inga extra API-anrop behövs.
#[derive(Deserialize)]
struct CityCountry {
    city_to_country: HashMap<String, String>,
}

fn load_city_country() -> HashMap<String, String> {
    let json = include_str!("../../data/city_country.json");
    let data: CityCountry = serde_json::from_str(json).expect("trasig city_country.json");

    data.city_to_country
}

/// Slår upp landet för en stad. API:et skriver ibland flygplatskoden efter
/// staden ("London LHR"), men Excel-filen har bara "London" – då tar vi bort koden.
fn lookup_country<'a>(map: &'a HashMap<String, String>, city: &str) -> &'a str {
    let city = city.to_lowercase();

    let without_code = match city.rsplit_once(' ') {
        Some((name, code)) if code.len() == 3 => name,
        _ => city.as_str(),
    };

    let country = map.get(&city).or_else(|| map.get(without_code));

    country.map_or("", |c| c.as_str())
}

fn main() {
    let city_to_country = load_city_country();

    let client = match Client::from_env() {
        Ok(client) => client,
        Err(e) => return println!("Fel: {e}"),
    };

    // ── Frågor till användaren ──
    let Some(airport) = choose_airport() else {
        return;
    };

    let Some((date, _)) = choose_date() else {
        return;
    };

    let Some(kind) = ask("  Ankomster eller avgångar? [a/d]: ") else {
        return;
    };
    let departures = kind == "d";

    let filter = ask("  Filtrera på land eller stad (Enter = alla): ").unwrap_or_default();
    let filter = filter.to_lowercase();

    // ── GET arrivals eller GET departures ──
    let answer = if departures {
        client.departures(airport, date)
    } else {
        client.arrivals(airport, date)
    };

    let flights = match answer {
        Ok(list) => list.flights,
        Err(e) => return println!("  ⚠ Fel: {e}"),
    };

    // ── Räkna flyg per stad ──
    // Ankomster: staden de kommer från. Avgångar: staden de ska till.
    let mut count: HashMap<String, usize> = HashMap::new();

    for f in flights.iter().filter(|f| is_valid(f)) {
        let city = if departures {
            &f.arrival_airport_english
        } else {
            &f.departure_airport_english
        };

        if let Some(city) = city {
            *count.entry(city.clone()).or_default() += 1;
        }
    }

    // ── Lägg till land och filtrera ──
    let mut rows: Vec<(String, usize, &str)> = Vec::new();

    for (city, n) in count {
        let country = lookup_country(&city_to_country, &city);

        let matches = filter.is_empty()
            || country.to_lowercase() == filter
            || city.to_lowercase().contains(&filter);

        if matches {
            rows.push((city, n, country));
        }
    }

    // Flest flyg först, sedan i bokstavsordning.
    rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    // ── Skriv ut ──
    println!("\n  {} destinationer:\n", rows.len());

    for (city, n, country) in rows {
        println!("  {city:<24} ({n:>2} st)  [{country}]");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn land_hittas_aven_med_flygplatskod() {
        let map = load_city_country();

        assert_eq!(lookup_country(&map, "Frankfurt"), "Germany");
        assert_eq!(lookup_country(&map, "London LHR"), "Great Britain");
        assert_eq!(lookup_country(&map, "Okänd stad"), "");
    }
}
