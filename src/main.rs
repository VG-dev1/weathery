use anyhow::Result;
use clap::Parser;

mod animate;
mod image_fetch;
mod weather;

use animate::{Weather, animate_weather};
use image_fetch::{download_image, get_city_image_url};
use tokio::sync::watch;
use weather::{Units, get_weather};

use crossterm::{
    event::{Event, KeyCode, poll, read},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::time::Duration;

fn weather_from_code(code: u32) -> Weather {
    match code {
        51 | 53 | 55 | 61 | 63 | 65 | 80 | 81 | 82 => Weather::Rain,
        71 | 73 | 75 | 77 | 85 | 86 => Weather::Snow,
        95 | 96 | 99 => Weather::Thunderstorm,
        _ => Weather::Clear,
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "weathery",
    version,
    about = "A terminal weather app with animated cityscapes"
)]
struct Args {
    /// City to fetch the weather of
    #[arg(num_args = 1.., value_delimiter = ' ')]
    city: Vec<String>,

    /// Force a grayscale image
    #[arg(long)]
    grayscale: bool,

    /// Force a colorful image
    #[arg(long)]
    colorful: bool,

    /// Use imperial units (Fahrenheit, mph) instead of metric
    #[arg(long)]
    imperial: bool,

    /// Simulate a specific weather condition
    #[arg(long)]
    simulate: Option<u32>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let city = args.city.join(" ");
    if city.is_empty() {
        eprintln!("Error: City not provided.");
        std::process::exit(1);
    }

    let weather_data = get_weather(&city, args.simulate).await?;
    let image_url = get_city_image_url(&weather_data.city).await?;
    let image_url = if image_url.is_none() && weather_data.city != city {
        get_city_image_url(&city).await?
    } else {
        image_url
    };

    let Some(url) = image_url else {
        eprintln!("Error: Could not find city: '{city}'.");
        std::process::exit(1);
    };

    let grayscale = if args.colorful {
        false
    } else if weather_data.description.contains("fog") || weather_data.description.contains("Fog") {
        true
    } else {
        args.grayscale
    };

    let img = download_image(&url).await?;
    let weather = weather_from_code(weather_data.weather_code);

    let units = if args.imperial {
        Units::Imperial
    } else {
        Units::Metric
    };

    let (exit_tx, exit_rx) = watch::channel(false);
    let (_, weather_rx) = watch::channel(weather_data.format(units));
    let (resize_tx, resize_rx) = watch::channel(());

    tokio::spawn(async move {
        enable_raw_mode().unwrap();

        loop {
            if poll(Duration::from_millis(100)).unwrap_or(false) {
                match read() {
                    Ok(Event::Key(key)) if key.code == KeyCode::Char('q') => {
                        exit_tx.send(true).unwrap();
                        break;
                    }
                    Ok(Event::Resize(_, _)) => {
                        let _ = resize_tx.send(());
                    }
                    _ => {}
                }
            }

            if *exit_tx.borrow() {
                break;
            }
        }
        disable_raw_mode().unwrap();
    });

    animate_weather(
        &img,
        &weather,
        weather_rx,
        exit_rx,
        resize_rx,
        grayscale,
        weather_data.is_night,
    )
    .await?;
    Ok(())
}
