use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: tds \"<from>\" \"<to>\"");
        std::process::exit(1);
    }

    let from = &args[1];
    let to = &args[2];
    // Only the car route needs a key — the train route works without one.
    let api_key = env::var("ORS_API_KEY").ok();

    let (transport_dur, drive_dur) = tokio::join!(
        transfer_duration_rail(from, to),
        car_duration(from, to, api_key.as_deref())
    );

    match transport_dur {
        Ok(val) => println!("Optimal travel time by train: {val}\n"),
        Err(e) => eprintln!("Train travel error: {e}\n"),
    }

    match drive_dur {
        Ok(val) => println!("Estimated travel time by vehicle: {val}"),
        Err(e) => eprintln!("Car travel error: {e}"),
    }

    Ok(())
}

/// Pull a human-readable message out of an OpenRouteService error response,
/// which is sometimes `{"error":"..."}` and sometimes `{"error":{"message":"..."}}`.
fn ors_error(res: &serde_json::Value, fallback: &str) -> String {
    if let Some(s) = res["error"].as_str() {
        s.to_string()
    } else if let Some(s) = res["error"]["message"].as_str() {
        s.to_string()
    } else {
        fallback.to_string()
    }
}

async fn car_duration(
    from: &str,
    to: &str,
    api_key: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = api_key.ok_or(
        "ORS_API_KEY is not set (needed for the car route) — add it to a .env file, see the README",
    )?;
    let client = reqwest::Client::new();

    // Geocode both addresses to coordinates.
    let from_coord = geo(&client, from, api_key).await?;
    let to_coord = geo(&client, to, api_key).await?;

    // Driving duration between the two points.
    let url = "https://api.openrouteservice.org/v2/directions/driving-car";
    let body = serde_json::json!({
        "coordinates": [[from_coord.0, from_coord.1], [to_coord.0, to_coord.1]]
    });

    let res: serde_json::Value = client
        .post(url)
        .header("Authorization", api_key)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    let seconds = res["routes"][0]["summary"]["duration"]
        .as_f64()
        .ok_or_else(|| ors_error(&res, "no driving route returned"))?;

    let minutes = (seconds / 60.0).round();
    Ok(format!("{minutes} min"))
}

async fn transfer_duration_rail(
    from: &str,
    to: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let v: serde_json::Value = reqwest::Client::new()
        .get("https://transport.opendata.ch/v1/connections")
        .query(&[("from", from), ("to", to), ("limit", "5")])
        .send()
        .await?
        .json()
        .await?;

    let connections = v["connections"]
        .as_array()
        .ok_or("invalid response from the transport API")?;
    if connections.is_empty() {
        return Err(format!("no connections found from \"{from}\" to \"{to}\"").into());
    }

    // The API returns Unix timestamps; the best connection is the earliest arrival.
    let best_conn = connections
        .iter()
        .min_by_key(|c| c["to"]["arrivalTimestamp"].as_i64().unwrap_or(i64::MAX))
        .ok_or("no connections found")?;

    let duration_str = best_conn["duration"].as_str().unwrap_or("?");
    let transfers = best_conn["transfers"].as_i64().unwrap_or(0);
    let dur_min = parse_duration_to_minutes(duration_str).unwrap_or(0);

    let mut out = vec![format!("{dur_min} min | Transfers: {transfers}")];

    if let Some(sections) = best_conn["sections"].as_array() {
        for section in sections {
            let dep_str = match section["departure"]["departure"].as_str() {
                Some(s) => s,
                None => continue,
            };
            let arr_str = match section["arrival"]["arrival"].as_str() {
                Some(s) => s,
                None => continue,
            };

            let dep_time = hhmm(dep_str);
            let arr_time = hhmm(arr_str);
            let from_name = section["departure"]["station"]["name"].as_str().unwrap_or("?");
            let to_name = section["arrival"]["station"]["name"].as_str().unwrap_or("?");

            if section["journey"].is_null() {
                out.push(format!("{dep_time}-{arr_time} | walk → {to_name}"));
            } else {
                let category = section["journey"]["category"].as_str().unwrap_or_default();
                let number = section["journey"]["number"].as_str().unwrap_or_default();
                let line = format!("{category} {number}").trim().to_string();
                let platform = section["departure"]["platform"].as_str().unwrap_or("-");
                out.push(format!(
                    "{dep_time}-{arr_time} | {line} | [{from_name}] → [{to_name}] | Platform: {platform}"
                ));
            }
        }
    }

    Ok(out.join("\n"))
}

/// "2026-06-25T14:26:00+0200" -> "14:26", without panicking on odd input.
fn hhmm(iso: &str) -> &str {
    iso.get(11..16).unwrap_or(iso)
}

/// OpenData duration looks like "00d03:13:00" (days d HH:MM:SS).
fn parse_duration_to_minutes(s: &str) -> Option<u32> {
    let parts: Vec<&str> = s.split('d').next_back()?.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let hours: u32 = parts[0].parse().ok()?;
    let minutes: u32 = parts[1].parse().ok()?;
    Some(hours * 60 + minutes)
}

async fn geo(
    client: &reqwest::Client,
    place: &str,
    api_key: &str,
) -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let url = "https://api.openrouteservice.org/geocode/search";
    let res: serde_json::Value = client
        .get(url)
        .query(&[("api_key", api_key), ("text", place)])
        .send()
        .await?
        .json()
        .await?;

    let coord = &res["features"][0]["geometry"]["coordinates"];
    let lon = coord[0]
        .as_f64()
        .ok_or_else(|| ors_error(&res, &format!("couldn't geocode \"{place}\"")))?;
    let lat = coord[1]
        .as_f64()
        .ok_or_else(|| ors_error(&res, &format!("couldn't geocode \"{place}\"")))?;
    Ok((lon, lat))
}
