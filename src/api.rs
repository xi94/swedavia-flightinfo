//! Pratar med Swedavias FlightInfo v2-API och beskriver JSON-datan som kommer tillbaka.
//!
//! API:et har fyra endpoints, alla anropas med GET:
//!   GET /{flygplats}/arrivals/{datum}    – ankomster
//!   GET /{flygplats}/departures/{datum}  – avgångar
//!   GET /query?filter=...                – OData-förfrågan
//!   GET /heartbeat                       – hälsokontroll

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use serde::Deserialize;
use serde::de::DeserializeOwned;

const BASE_URL: &str = "https://api.swedavia.se/flightinfo/v2";

// ── JSON-datan ───────────────────────────────────────────────────
// Alla fält är valfria (`default`) eftersom API:et ibland utelämnar fält.

/// Svaret från arrivals och departures.
#[derive(Deserialize, Default)]
#[serde(default)]
pub struct FlightList {
    pub flights: Vec<Flight>,
}

/// Svaret från query: varje flyg ligger inuti `arrival` eller `departure`,
/// och det kommer max 25 flyg per svar.
#[derive(Deserialize, Default)]
#[serde(default)]
pub struct QueryAnswer {
    pub flights: Vec<QueryFlight>,
    pub continuationtoken: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct QueryFlight {
    pub arrival: Option<Flight>,
    pub departure: Option<Flight>,
}

/// Ett flyg. Namnen är samma som i JSON, fast i snake_case.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Flight {
    pub flight_id: Option<String>,
    pub airline_operator: Option<Airline>,
    pub departure_airport_english: Option<String>, // finns på ankomster
    pub arrival_airport_english: Option<String>,   // finns på avgångar
    pub arrival_time: Option<Times>,
    pub departure_time: Option<Times>,
    pub location_and_status: Option<Status>,
    pub baggage: Option<Baggage>,
    pub di_indicator: Option<String>, // D = inrikes, I = internationellt, S = Schengen
    pub remarks_english: Vec<Remark>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct Airline {
    pub name: Option<String>,
    pub iata: Option<String>,
}

/// Tiderna kommer i UTC, t.ex. "2026-11-25T16:25:00Z".
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Times {
    pub scheduled_utc: Option<String>, // planerad
    pub estimated_utc: Option<String>, // beräknad
    pub actual_utc: Option<String>,    // faktisk
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Status {
    pub terminal: Option<String>,
    pub gate: Option<String>,
    pub flight_leg_status_english: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Baggage {
    pub baggage_claim_unit: Option<String>, // bagageband
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct Remark {
    pub text: Option<String>,
}

impl Flight {
    /// Ankomsttid för ankomster, avgångstid för avgångar.
    pub fn times(&self) -> Option<&Times> {
        self.arrival_time.as_ref().or(self.departure_time.as_ref())
    }
}

// ── Klienten ─────────────────────────────────────────────────────

pub struct Client {
    http: reqwest::blocking::Client,
    api_key: String,
}

impl Client {
    /// Läser nyckeln från `.env`, så den aldrig står i koden.
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let api_key = std::env::var("SWEDAVIA_API_KEY")
            .context("Ingen API-nyckel. Kopiera .env.example till .env och lägg in din nyckel.")?;

        Ok(Self {
            http: reqwest::blocking::Client::new(),
            api_key,
        })
    }

    /// GET /{flygplats}/arrivals/{datum} – alla ankomster en dag.
    pub fn arrivals(&self, airport: &str, date: NaiveDate) -> Result<FlightList> {
        self.get_json(&format!("/{airport}/arrivals/{date}"), &[])
    }

    /// GET /{flygplats}/departures/{datum} – alla avgångar en dag.
    pub fn departures(&self, airport: &str, date: NaiveDate) -> Result<FlightList> {
        self.get_json(&format!("/{airport}/departures/{date}"), &[])
    }

    /// GET /query?filter=... – OData-förfrågan.
    /// Exempel: `airport eq 'ARN' and flightType eq 'A' and scheduled eq '2026-11-25'`
    pub fn query(&self, filter: &str) -> Result<QueryAnswer> {
        self.get_json("/query", &[("filter", filter)])
    }

    /// GET /heartbeat – svarar "IsAlive" när API:et och nyckeln fungerar.
    pub fn heartbeat(&self) -> Result<String> {
        self.get("/heartbeat", &[])
    }

    /// Gör själva GET-anropet med API-nyckeln i headern och returnerar svaret som text.
    fn get(&self, path: &str, query: &[(&str, &str)]) -> Result<String> {
        let url = format!("{BASE_URL}{path}");
        println!("\n  → GET {url}");

        let response = self
            .http
            .get(&url)
            .header("Ocp-Apim-Subscription-Key", &self.api_key)
            .header("Accept", "application/json")
            .query(query)
            .send()?;

        // Ge ett tydligt felmeddelande i stället för en kryptisk felkod.
        match response.status().as_u16() {
            401 => bail!("API-nyckeln är fel (401)."),
            403 | 429 => bail!("API-kvoten är slut. Försök igen senare."),
            code if code >= 400 => bail!("API:et svarade med fel {code}."),
            _ => Ok(response.text()?),
        }
    }

    /// Gör om svaret från JSON till structs. Ett tomt svar (204 = inga flyg) blir en tom lista.
    fn get_json<T: DeserializeOwned + Default>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T> {
        let body = self.get(path, query)?;

        if body.trim().is_empty() {
            return Ok(T::default());
        }

        let data = serde_json::from_str(&body)?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Riktiga svar sparade från API:et 2026-11-25.
    #[test]
    fn riktigt_arrivals_svar_kan_lasas() {
        let list: FlightList =
            serde_json::from_str(include_str!("../tests/fixtures/arrivals_arn_real.json")).unwrap();
        assert_eq!(list.flights.len(), 279);
    }

    #[test]
    fn riktigt_query_svar_kan_lasas() {
        let answer: QueryAnswer =
            serde_json::from_str(include_str!("../tests/fixtures/query_arn_real.json")).unwrap();
        assert_eq!(answer.flights.len(), 25);
        assert!(answer.flights[0].arrival.is_some());
    }
}
