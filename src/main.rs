//! Swedavia FlightInfo v2 – en Rust-version av en Python-app för terminalen.
//! Starta: `cargo run`

use anyhow::Result;

use swedavia::api::{Client, Flight};
use swedavia::input::{airport_name, ask, choose_airport, choose_date, today};
use swedavia::output::{fmt_time, is_upcoming, is_valid, print_flight, print_paged};

const LINE: &str = "════════════════════════════════════════════════";

fn main() {
    let client = match Client::from_env() {
        Ok(client) => client,
        Err(e) => return println!("Fel: {e}"),
    };

    print_menu();

    while let Some(choice) = ask("\nVälj [1-6] eller q: ") {
        let result = match choice.as_str() {
            "1" => show_flights(&client, "arrivals"),
            "2" => show_flights(&client, "departures"),
            "3" => search_flight(&client),
            "4" => odata_query(&client),
            "5" => heartbeat(&client),
            "6" => demo(&client),
            "q" => break,
            _ => continue,
        };

        // Vid fel visas felet och vi går tillbaka till menyn i stället för att krascha.
        if let Err(e) = result {
            println!("  ⚠ Fel: {e}");
        }

        print_menu();
    }
}

fn print_menu() {
    println!("\n{LINE}");
    println!("  Swedavia FlightInfo API v2  –  Rust-klient");
    println!("{LINE}");
    println!("  1. Ankomster (arrivals)  för en flygplats & datum");
    println!("  2. Avgångar  (departures) för en flygplats & datum");
    println!("  3. Sök specifikt flightnummer");
    println!("  4. OData-förfrågan (query endpoint)");
    println!("  5. HeartBeat – hälsokontroll av API");
    println!("  6. Demonstrera alla endpoints automatiskt");
    println!("  q. Avsluta");
    println!("{LINE}");
}

/// Meny 1 och 2: visar ankomster (GET arrivals) eller avgångar (GET departures).
fn show_flights(client: &Client, direction: &str) -> Result<()> {
    let Some(airport) = choose_airport() else {
        return Ok(());
    };

    let Some((date, only_upcoming)) = choose_date() else {
        return Ok(());
    };

    let list = if direction == "arrivals" {
        client.arrivals(airport, date)?
    } else {
        client.departures(airport, date)?
    };

    // Ta bort spökposter, och vid "nu" även flyg som redan landat/lyft.
    let flights: Vec<&Flight> = list
        .flights
        .iter()
        .filter(|f| is_valid(f))
        .filter(|f| !only_upcoming || is_upcoming(f))
        .collect();

    let name = airport_name(airport);
    println!("\n  {airport} – {name} {date}: {} flights\n", flights.len());

    print_paged(&flights, airport, direction);
    Ok(())
}

/// Meny 3: letar efter flightnumret bland både ankomster och avgångar.
fn search_flight(client: &Client) -> Result<()> {
    let Some(wanted) = ask("  Ange flightnummer (t.ex. SK532): ") else {
        return Ok(());
    };
    let wanted = wanted.replace(' ', "").to_uppercase();

    let Some(airport) = choose_airport() else {
        return Ok(());
    };

    let Some((date, _)) = choose_date() else {
        return Ok(());
    };

    let arrivals = client.arrivals(airport, date)?.flights;
    let departures = client.departures(airport, date)?.flights;

    let mut found = 0;

    for (direction, flights) in [("arrivals", arrivals), ("departures", departures)] {
        for f in flights {
            if f.flight_id.as_deref() == Some(wanted.as_str()) {
                found += 1;
                print_flight(&f, found, airport, direction);
            }
        }
    }

    if found == 0 {
        println!("  Hittade inget flyg {wanted} på {airport} {date}.");
    }

    Ok(())
}

/// Ett exempelfilter: dagens ankomster till Arlanda.
fn example_filter() -> String {
    let date = today();
    format!("airport eq 'ARN' and flightType eq 'A' and scheduled eq '{date}'")
}

/// Meny 4: användaren skriver ett OData-filter (flightType A = ankomster, D = avgångar).
fn odata_query(client: &Client) -> Result<()> {
    println!("\n  Exempel: {}", example_filter());

    let Some(answer) = ask("  Ange filter (Enter = exemplet): ") else {
        return Ok(());
    };

    let filter = if answer.is_empty() {
        example_filter()
    } else {
        answer
    };

    let result = client.query(&filter)?;
    println!("\n  {} flights i svaret:\n", result.flights.len());

    // En rad per flyg: flightnummer, stad, planerad tid, status.
    for q in &result.flights {
        let Some(f) = q.arrival.as_ref().or(q.departure.as_ref()) else {
            continue;
        };

        let id = f.flight_id.as_deref().unwrap_or("N/A");
        let place = f
            .departure_airport_english
            .as_deref()
            .or(f.arrival_airport_english.as_deref())
            .unwrap_or("");
        let time = fmt_time(f.times().and_then(|t| t.scheduled_utc.as_deref()));
        let status = f
            .location_and_status
            .as_ref()
            .and_then(|s| s.flight_leg_status_english.as_deref())
            .unwrap_or("");

        println!("  {id:<8} {place:<22} {time:<24} {status}");
    }

    if result.continuationtoken.is_some() {
        println!("\n  (Fler resultat finns – API:et skickar 25 åt gången.)");
    }

    Ok(())
}

/// Meny 5: hälsokontroll av API:et (GET heartbeat).
fn heartbeat(client: &Client) -> Result<()> {
    let answer = client.heartbeat()?;
    println!("  API svarar: {answer}");
    Ok(())
}

/// Meny 6: anropar alla fyra endpoints efter varandra.
fn demo(client: &Client) -> Result<()> {
    println!("\n── 1/4 HeartBeat ──");
    heartbeat(client)?;

    println!("\n── 2/4 Ankomster ARN idag ──");
    let arrivals = client.arrivals("ARN", today())?;
    println!("  {} flights", arrivals.flights.len());

    println!("\n── 3/4 Avgångar ARN idag ──");
    let departures = client.departures("ARN", today())?;
    println!("  {} flights", departures.flights.len());

    println!("\n── 4/4 OData-förfrågan ──");
    let query = client.query(&example_filter())?;
    println!("  {} flights i första svaret", query.flights.len());

    println!("\n  Alla 4 endpoints fungerar.");
    Ok(())
}
