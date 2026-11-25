//! Läser in och kontrollerar det användaren skriver.

use std::io::Write;

use chrono::{Days, NaiveDate, Utc};
use chrono_tz::Europe::Stockholm;

/// Swedavias tio flygplatser (IATA-kod, namn).
pub const AIRPORTS: [(&str, &str); 10] = [
    ("ARN", "Stockholm Arlanda"),
    ("GOT", "Göteborg Landvetter"),
    ("BMA", "Stockholm Bromma"),
    ("MMX", "Malmö"),
    ("LLA", "Luleå"),
    ("UME", "Umeå"),
    ("OSD", "Östersund"),
    ("VBY", "Visby"),
    ("RNB", "Ronneby"),
    ("KRN", "Kiruna"),
];

/// Skriver en fråga och returnerar svaret. `None` = inmatningen tog slut.
pub fn ask(question: &str) -> Option<String> {
    print!("{question}");
    std::io::stdout().flush().ok();

    let mut line = String::new();

    match std::io::stdin().read_line(&mut line) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(line.trim().to_string()),
    }
}

/// Frågar tills svaret är en giltig flygplats.
pub fn choose_airport() -> Option<&'static str> {
    println!("\n  Tillgängliga flygplatser:");

    for (code, name) in AIRPORTS {
        println!("    {code} – {name}");
    }

    loop {
        let answer = ask("  Ange IATA-kod eller stadsnamn (t.ex. ARN eller Visby): ")?;

        match parse_airport(&answer) {
            Some(code) => return Some(code),
            None => println!("  ⚠ Okänd flygplats, försök igen"),
        }
    }
}

/// Frågar tills svaret är ett giltigt datum.
pub fn choose_date() -> Option<(NaiveDate, bool)> {
    loop {
        let answer = ask("  Ange datum (YYYY-MM-DD / nu / idag / imorgon / igår): ")?;

        match parse_date(&answer, today()) {
            Some(choice) => return Some(choice),
            None => println!("  ⚠ Ogiltigt datum, försök igen"),
        }
    }
}

/// "arn", "ARN", "visby" eller "arlanda" → "ARN"/"VBY"/... ; okänd → None.
pub fn parse_airport(input: &str) -> Option<&'static str> {
    let input = input.trim().to_lowercase();

    if input.is_empty() {
        return None;
    }

    for (code, name) in AIRPORTS {
        let same_code = code.to_lowercase() == input;
        let in_name = name.to_lowercase().contains(&input);

        if same_code || in_name {
            return Some(code);
        }
    }

    None
}

/// "ARN" → "Stockholm Arlanda".
pub fn airport_name(code: &str) -> &'static str {
    for (c, name) in AIRPORTS {
        if c == code {
            return name;
        }
    }

    "?"
}

/// Dagens datum i Sverige.
pub fn today() -> NaiveDate {
    Utc::now().with_timezone(&Stockholm).date_naive()
}

/// Returnerar datumet och om bara kommande flyg ska visas.
/// "nu" = idag, men bara flyg som inte har landat/lyft än.
pub fn parse_date(input: &str, today: NaiveDate) -> Option<(NaiveDate, bool)> {
    let input = input.trim().to_lowercase();

    match input.as_str() {
        "nu" => Some((today, true)),
        "idag" | "" => Some((today, false)),
        "imorgon" => Some((today + Days::new(1), false)),
        "igår" => Some((today - Days::new(1), false)),
        other => {
            let date = NaiveDate::parse_from_str(other, "%Y-%m-%d").ok()?;
            Some((date, false))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flygplats_med_kod_eller_stad() {
        assert_eq!(parse_airport("arn"), Some("ARN"));
        assert_eq!(parse_airport("Visby"), Some("VBY"));
        assert_eq!(parse_airport("CPH"), None);
    }

    #[test]
    fn datum() {
        let day = NaiveDate::from_ymd_opt(2026, 11, 25).unwrap();
        let tomorrow = NaiveDate::from_ymd_opt(2026, 11, 26).unwrap();
        let christmas = NaiveDate::from_ymd_opt(2026, 12, 24).unwrap();

        assert_eq!(parse_date("nu", day), Some((day, true)));
        assert_eq!(parse_date("imorgon", day), Some((tomorrow, false)));
        assert_eq!(parse_date("2026-12-24", day), Some((christmas, false)));
        assert_eq!(parse_date("24/12", day), None);
    }
}
