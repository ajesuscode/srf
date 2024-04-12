use anyhow::Result;
use chrono::{NaiveDate, NaiveDateTime, Timelike, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};
// Add this line to import the missing type

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the surf spot
    /// If spot contains spaces, it should be enclosed in quotes "Banzai Pipeline"
    #[arg(value_parser)]
    spot: String,
    /// Lists a weekly surf forecast
    #[arg(short, long)]
    week: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct SpotCoordinates {
    lat: String,
    lon: String,
    name: String,
    display_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct HourlyData {
    time: Vec<String>,
    wave_height: Option<Vec<serde_json::Value>>,
    wave_direction: Option<Vec<serde_json::Value>>,
    wave_period: Option<Vec<serde_json::Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct HourlySpotForecast {
    hourly: HourlyData,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let cur_date = Utc::now().naive_utc().date();

    let spot_coordinates = match get_coordinates(&args.spot).await {
        Ok(coordinates) => {
            if coordinates.is_empty() {
                eprintln!("No coordinates found. Maybe you misspelled the spot name?");
                return;
            }
            coordinates
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    let first_coord = &spot_coordinates[0];

    if args.week {
        match spot_forecast(&first_coord.lat, &first_coord.lon).await {
            Ok(forecast_data) => week_forecast(forecast_data, cur_date).await,
            Err(e) => {
                eprintln!("Error fetching forecast: {}", e);
            }
        }
    } else {
        match spot_forecast(&first_coord.lat, &first_coord.lon).await {
            Ok(forecast_data) => today_forecast(forecast_data, cur_date).await,
            Err(e) => {
                eprintln!("Error fetching forecast: {}", e);
            }
        }
    }
}

async fn get_coordinates(spot: &String) -> Result<Vec<SpotCoordinates>> {
    let url = format!(
        "https://nominatim.openstreetmap.org/search?q={}&format=json",
        spot
    );
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "surf-forecast")
        .send()
        .await?;
    match response.status() {
        reqwest::StatusCode::OK => response
            .json::<Vec<SpotCoordinates>>()
            .await
            .map_err(Into::into),
        _ => Err(anyhow::anyhow!(
            "Unexpected status code: {:?}",
            response.status()
        )),
    }
}

async fn spot_forecast(lat: &String, lon: &String) -> Result<HourlySpotForecast> {
    let url = format!("https://marine-api.open-meteo.com/v1/marine?latitude={}&longitude={}&hourly=wave_height,wave_direction,wave_period", lat, lon);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "surf-forecast")
        .send()
        .await?;
    match response.status() {
        reqwest::StatusCode::OK => response
            .json::<HourlySpotForecast>()
            .await
            .map_err(Into::into),
        _ => Err(anyhow::anyhow!(
            "Status code: {:?}. Reason: {}",
            response.status(),
            response.text().await?.to_string()
        )),
    }
}

async fn week_forecast(forecast_data: HourlySpotForecast, cur_date: NaiveDate) {
    let mut week = cur_date;
    for _ in 0..7 {
        let day = week.format("%a %e");
        println!("DAY: {}", day);
        for (index, time) in forecast_data.hourly.time.iter().enumerate() {
            let days = NaiveDateTime::parse_from_str(time, "%Y-%m-%dT%H:%M").unwrap();

            if week == days.date() {
                let wave_height = forecast_data
                    .hourly
                    .wave_height
                    .as_ref()
                    .and_then(|v| v.get(index))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let wave_period = forecast_data
                    .hourly
                    .wave_period
                    .as_ref()
                    .and_then(|v| v.get(index))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                println!(
                    "⏰: {}, 🌊: {}, ⏱️: {}",
                    days.hour(),
                    wave_height,
                    wave_period
                );
            }
        }
        week = week.succ();
    }
}

async fn today_forecast(forecast_data: HourlySpotForecast, cur_date: NaiveDate) {
    let today = cur_date.format("%a %e");
    println!("TODAY: {}", today);
    for (index, time) in forecast_data.hourly.time.iter().enumerate() {
        let days = NaiveDateTime::parse_from_str(time, "%Y-%m-%dT%H:%M").unwrap();

        if cur_date == days.date() {
            let wave_height = forecast_data
                .hourly
                .wave_height
                .as_ref()
                .and_then(|v| v.get(index))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let wave_period = forecast_data
                .hourly
                .wave_period
                .as_ref()
                .and_then(|v| v.get(index))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            println!(
                "⏰: {}, 🌊: {}, ⏱️: {}",
                days.hour(),
                wave_height,
                wave_period
            );
        }
    }
}
