use anyhow::Result;
use clap::Parser;

mod animate;
mod image_fetch;
mod weather;

use animate::{animate_weather, Weather};
use image_fetch::{download_image, get_city_image_url};
use tokio::sync::watch;
use weather::get_weather;

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

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

    /// Simulate a specific weather condition
    #[arg(long)]
    simulate: Option<u32>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = Args::parse();
    let city = args.city.join(" ");
    if city.is_empty() {
        eprintln!("Error: City not provided.");
        std::process::exit(1);
    }

    let (image_url, weather_str) =
        tokio::try_join!(get_city_image_url(&city), get_weather(&city, args.simulate),)?;

    let Some(url) = image_url else {
        eprintln!("Error: Could not find city: '{city}'.");
        std::process::exit(1);
    };

    if args.colorful {
        args.grayscale = false;
    } else if weather_str.contains("fog") || weather_str.contains("Fog") {
        args.grayscale = true;
    }

    let img = download_image(&url).await?;
    let weather = Weather::from_str(&weather_str);

    let (tx, rx) = watch::channel(false);
    tokio::spawn(async move {
        enable_raw_mode().unwrap();
        loop {
            if let Event::Key(key) = event::read().unwrap() {
                if key.code == KeyCode::Char('q') {
                    tx.send(true).unwrap();
                    break;
                }
            }
        }
        disable_raw_mode().unwrap();
    });

    animate_weather(&img, &weather, &weather_str, args.grayscale, rx).await?;

    Ok(())
}
