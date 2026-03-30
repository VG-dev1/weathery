use anyhow::Result;
use crossterm::{
    QueueableCommand,
    cursor::{Hide, MoveTo, Show},
    execute,
    style::{Color, PrintStyledContent, ResetColor, Stylize},
    terminal::{DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen},
};
use image::{DynamicImage, GenericImageView, Rgba, imageops::grayscale};
use rand::Rng;
use std::io::{self, Write};
use std::time::{Duration, Instant};
use tokio::sync::watch::Receiver;
use tokio::time::sleep;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Weather {
    Clear,
    Rain,
    Snow,
    Thunderstorm,
}

impl Weather {
    pub fn from_str(weather_str: &str) -> Self {
        if weather_str.contains("rain")
            || weather_str.contains("drizzle")
            || weather_str.contains("rain showers")
        {
            Weather::Rain
        } else if weather_str.contains("snow") || weather_str.contains("Snow") {
            Weather::Snow
        } else if weather_str.contains("Thunderstorm") {
            Weather::Thunderstorm
        } else {
            Weather::Clear
        }
    }
}

#[derive(Debug, Clone)]
struct Particle {
    x: u16,
    y: u16,
}

#[derive(Debug, Clone, Copy)]
enum Intensity {
    Light,
    Moderate,
    Heavy,
}

impl Intensity {
    fn delay_ms(&self) -> u64 {
        match self {
            Intensity::Light => 80,
            Intensity::Moderate => 40,
            Intensity::Heavy => 15,
        }
    }

    fn spawn_probability(&self) -> f64 {
        match self {
            Intensity::Light => 0.2,
            Intensity::Moderate => 0.5,
            Intensity::Heavy => 0.8,
        }
    }
}

struct AnimationContext<'a> {
    weather_rx: &'a mut Receiver<String>,
    exit_rx: &'a mut Receiver<bool>,
    resize_rx: &'a mut Receiver<()>,
}

pub async fn animate_weather(
    image: &DynamicImage,
    weather: &Weather,
    mut weather_rx: Receiver<String>,
    mut exit_rx: Receiver<bool>,
    mut resize_rx: Receiver<()>,
    is_grayscale: bool,
) -> Result<()> {
    let (cols, rows) = get_terminal_size();

    execute!(io::stdout(), EnterAlternateScreen, Hide, DisableLineWrap)?;

    let intensity = match weather {
        Weather::Rain => Intensity::Moderate,
        Weather::Snow => Intensity::Light,
        Weather::Thunderstorm => Intensity::Heavy,
        Weather::Clear => Intensity::Light,
    };

    let mut ctx = AnimationContext {
        weather_rx: &mut weather_rx,
        exit_rx: &mut exit_rx,
        resize_rx: &mut resize_rx,
    };

    let result = match weather {
        Weather::Rain => animate_rain(image, rows, cols, intensity, is_grayscale, &mut ctx).await,
        Weather::Snow => animate_snow(image, rows, cols, intensity, is_grayscale, &mut ctx).await,
        Weather::Thunderstorm => {
            animate_thunderstorm(image, rows, cols, intensity, is_grayscale, &mut ctx).await
        }
        Weather::Clear => print_static(image, is_grayscale, &mut ctx).await,
    };

    execute!(io::stdout(), EnableLineWrap, LeaveAlternateScreen, Show)?;

    result
}

#[inline]
fn rain(rgb: Rgba<u8>) -> Rgba<u8> {
    let r = (rgb[0] as u16 + 100).min(255) as u8;
    let g = (rgb[1] as u16 + 150).min(255) as u8;
    let b = (rgb[2] as u16 + 255).min(255) as u8;

    Rgba([r, g, b, rgb[3]])
}

async fn animate_rain(
    image: &DynamicImage,
    mut rows: u16,
    mut cols: u16,
    intensity: Intensity,
    is_grayscale: bool,
    ctx: &mut AnimationContext<'_>,
) -> Result<()> {
    loop {
        let mut particles: Vec<Particle> = Vec::new();
        let mut rng = rand::thread_rng();
        let delay = Duration::from_millis(intensity.delay_ms());
        let spawn_prob = intensity.spawn_probability();
        let mut last_frame = Instant::now();

        let resized = image.resize_exact(
            cols as u32,
            rows.saturating_sub(2) as u32 * 2,
            image::imageops::FilterType::Lanczos3,
        );
        let resized = if is_grayscale {
            image::DynamicImage::ImageLuma8(grayscale(&resized))
        } else {
            resized
        };

        loop {
            if *ctx.exit_rx.borrow() {
                return Ok(());
            }

            if ctx.resize_rx.has_changed().unwrap_or(false) {
                let (new_cols, new_rows) = get_terminal_size();
                if new_cols != cols || new_rows != rows {
                    cols = new_cols;
                    rows = new_rows;
                    execute!(
                        io::stdout(),
                        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
                    )?;
                    break;
                }
            }

            if rng.gen_bool(spawn_prob) {
                let x = rng.gen_range(0..cols);
                particles.push(Particle { x, y: 0 });
            }

            let speed = match intensity {
                Intensity::Light => 1,
                Intensity::Moderate => 2,
                Intensity::Heavy => 3,
            };

            particles.retain_mut(|p| {
                p.y += speed;
                p.y < rows.saturating_sub(2)
            });

            let ws = ctx.weather_rx.borrow().clone();

            io::stdout()
                .queue(MoveTo(0, 0))?
                .queue(PrintStyledContent(ws.as_str().reset()))?;

            for term_y in 0..rows.saturating_sub(2) as u32 {
                io::stdout().queue(MoveTo(0, (term_y + 2) as u16))?;

                for x in 0..cols as u32 {
                    let top = resized.get_pixel(x, term_y * 2);
                    let bot = resized.get_pixel(x, term_y * 2 + 1);
                    let is_raining = particles
                        .iter()
                        .any(|p| p.x as u32 == x && p.y as u32 == term_y);
                    let top = if is_raining { rain(top) } else { top };
                    let bot = if is_raining { rain(bot) } else { bot };
                    draw_pixel(top, bot)?;
                }
                io::stdout().queue(ResetColor)?;
            }

            io::stdout().flush()?;

            let elapsed = last_frame.elapsed();
            if elapsed < delay {
                sleep(delay - elapsed).await;
            }
            last_frame = Instant::now();
        }
    }
}

#[inline]
fn snow(rgb: Rgba<u8>) -> Rgba<u8> {
    let r = (rgb[0] as u16 + 80).min(255) as u8;
    let g = (rgb[1] as u16 + 80).min(255) as u8;
    let b = (rgb[2] as u16 + 120).min(255) as u8;

    Rgba([r, g, b, rgb[3]])
}

async fn animate_snow(
    image: &DynamicImage,
    mut rows: u16,
    mut cols: u16,
    intensity: Intensity,
    is_grayscale: bool,
    ctx: &mut AnimationContext<'_>,
) -> Result<()> {
    loop {
        let mut particles: Vec<Particle> = Vec::new();
        let mut rng = rand::thread_rng();
        let delay = Duration::from_millis(intensity.delay_ms());
        let spawn_prob = intensity.spawn_probability();
        let mut last_frame = Instant::now();

        let resized = image.resize_exact(
            cols as u32,
            rows.saturating_sub(2) as u32 * 2,
            image::imageops::FilterType::Lanczos3,
        );
        let resized = if is_grayscale {
            image::DynamicImage::ImageLuma8(grayscale(&resized))
        } else {
            resized
        };

        loop {
            if *ctx.exit_rx.borrow() {
                return Ok(());
            }

            if ctx.resize_rx.has_changed().unwrap_or(false) {
                let (new_cols, new_rows) = get_terminal_size();
                if new_cols != cols || new_rows != rows {
                    cols = new_cols;
                    rows = new_rows;
                    execute!(
                        io::stdout(),
                        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
                    )?;
                    break;
                }
            }

            if rng.gen_bool(spawn_prob) {
                let x = rng.gen_range(0..cols);
                particles.push(Particle { x, y: 0 });
            }

            particles.retain_mut(|p| {
                p.y += 1;
                if rng.gen_bool(0.4) {
                    p.x = (p.x as i16 + rng.gen_range(-1..=1)).clamp(0, cols as i16 - 1) as u16;
                }
                p.y < rows.saturating_sub(2)
            });

            let ws = ctx.weather_rx.borrow().clone();

            io::stdout()
                .queue(MoveTo(0, 0))?
                .queue(PrintStyledContent(ws.as_str().reset()))?;

            for term_y in 0..rows.saturating_sub(2) as u32 {
                io::stdout().queue(MoveTo(0, (term_y + 2) as u16))?;

                for x in 0..cols as u32 {
                    let top = resized.get_pixel(x, term_y * 2);
                    let bot = resized.get_pixel(x, term_y * 2 + 1);
                    let is_snowing = particles
                        .iter()
                        .any(|p| p.x as u32 == x && p.y as u32 == term_y);

                    let top = if is_snowing { snow(top) } else { top };
                    let bot = if is_snowing { snow(bot) } else { bot };

                    draw_pixel(top, bot)?;
                }
                io::stdout().queue(ResetColor)?;
            }

            io::stdout().flush()?;

            let elapsed = last_frame.elapsed();
            if elapsed < delay {
                sleep(delay - elapsed).await;
            }
            last_frame = Instant::now();
        }
    }
}

#[inline]
fn flash(rgb: Rgba<u8>) -> Rgba<u8> {
    let r = (rgb[0] as u16 + 150).min(255) as u8;
    let g = (rgb[1] as u16 + 150).min(255) as u8;
    let b = (rgb[2] as u16 + 100).min(255) as u8;
    Rgba([r, g, b, rgb[3]])
}

#[inline]
fn storm(rgb: Rgba<u8>) -> Rgba<u8> {
    let r = (rgb[0] as u16 + 60).min(255) as u8;
    let g = (rgb[1] as u16 + 100).min(255) as u8;
    let b = (rgb[2] as u16 + 180).min(255) as u8;
    Rgba([r, g, b, rgb[3]])
}

async fn animate_thunderstorm(
    image: &DynamicImage,
    mut rows: u16,
    mut cols: u16,
    intensity: Intensity,
    is_grayscale: bool,
    ctx: &mut AnimationContext<'_>,
) -> Result<()> {
    loop {
        let mut particles: Vec<Particle> = Vec::new();
        let mut rng = rand::thread_rng();
        let delay = Duration::from_millis(intensity.delay_ms());
        let spawn_prob = intensity.spawn_probability().min(0.9);
        let mut last_frame = Instant::now();
        let mut flash_counter = 0;
        let flash_interval = 40;

        let resized = image.resize_exact(
            cols as u32,
            rows.saturating_sub(2) as u32 * 2,
            image::imageops::FilterType::Lanczos3,
        );
        let resized = if is_grayscale {
            image::DynamicImage::ImageLuma8(grayscale(&resized))
        } else {
            resized
        };

        loop {
            if *ctx.exit_rx.borrow() {
                return Ok(());
            }

            if ctx.resize_rx.has_changed().unwrap_or(false) {
                let (new_cols, new_rows) = get_terminal_size();
                if new_cols != cols || new_rows != rows {
                    cols = new_cols;
                    rows = new_rows;
                    execute!(
                        io::stdout(),
                        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
                    )?;
                    break;
                }
            }

            if rng.gen_bool(spawn_prob) {
                let x = rng.gen_range(0..cols);
                particles.push(Particle { x, y: 0 });
            }

            let speed = match intensity {
                Intensity::Light => 1,
                Intensity::Moderate => 2,
                Intensity::Heavy => 4,
            };

            particles.retain_mut(|p| {
                p.y += speed;
                p.y < rows.saturating_sub(2)
            });

            let should_flash = flash_counter % flash_interval == 0 && rng.gen_bool(0.15);

            let ws = ctx.weather_rx.borrow().clone();

            let weather_str = if should_flash {
                ws.as_str().with(Color::Rgb {
                    r: 255,
                    g: 255,
                    b: 150,
                })
            } else {
                ws.as_str().reset()
            };

            io::stdout()
                .queue(MoveTo(0, 0))?
                .queue(PrintStyledContent(weather_str))?;

            for term_y in 0..rows.saturating_sub(2) as u32 {
                io::stdout().queue(MoveTo(0, (term_y + 2) as u16))?;

                for x in 0..cols as u32 {
                    let top = resized.get_pixel(x, term_y * 2);
                    let bot = resized.get_pixel(x, term_y * 2 + 1);

                    let is_storming = particles
                        .iter()
                        .any(|p| p.x as u32 == x && p.y as u32 == term_y);

                    let top = if should_flash {
                        flash(top)
                    } else if is_storming {
                        storm(top)
                    } else {
                        top
                    };
                    let bot = if should_flash {
                        flash(bot)
                    } else if is_storming {
                        storm(bot)
                    } else {
                        bot
                    };
                    draw_pixel(top, bot)?;
                }
                io::stdout().queue(ResetColor)?;
            }

            io::stdout().flush()?;
            flash_counter += 1;

            let elapsed = last_frame.elapsed();
            if elapsed < delay {
                sleep(delay - elapsed).await;
            }
            last_frame = Instant::now();
        }
    }
}

async fn print_static(
    image: &DynamicImage,
    is_grayscale: bool,
    ctx: &mut AnimationContext<'_>,
) -> Result<()> {
    loop {
        let (cols, rows) = get_terminal_size();
        let resized = image.resize_exact(
            cols as u32,
            rows.saturating_sub(2) as u32 * 2,
            image::imageops::FilterType::Lanczos3,
        );
        let resized = if is_grayscale {
            image::DynamicImage::ImageLuma8(grayscale(&resized))
        } else {
            resized
        };

        let ws = ctx.weather_rx.borrow().clone();

        io::stdout()
            .queue(MoveTo(0, 0))?
            .queue(PrintStyledContent(ws.as_str().reset()))?;

        for term_y in 0..rows.saturating_sub(2) as u32 {
            io::stdout().queue(MoveTo(0, (term_y + 2) as u16))?;

            for x in 0..cols as u32 {
                let top = resized.get_pixel(x, term_y * 2);
                let bot = resized.get_pixel(x, term_y * 2 + 1);
                draw_pixel(top, bot)?;
            }
            io::stdout().queue(ResetColor)?;
        }

        io::stdout().flush()?;

        loop {
            tokio::select! {
                _ = ctx.exit_rx.changed() => { if *ctx.exit_rx.borrow() { return Ok(()); } }
                _ = ctx.weather_rx.changed() => {
                    let ws = ctx.weather_rx.borrow().clone();
                    io::stdout()
                        .queue(MoveTo(0, 0))?
                        .queue(PrintStyledContent(ws.as_str().reset()))?;
                    io::stdout().flush()?;
                }
                _ = ctx.resize_rx.changed() => {
                    let (new_cols, new_rows) = get_terminal_size();
                    if new_cols != cols || new_rows != rows {
                        execute!(io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
                        break;
                    }
                }
            }
        }
    }
}

#[inline]
fn draw_pixel(top: Rgba<u8>, bot: Rgba<u8>) -> Result<()> {
    let fg = Color::Rgb {
        r: bot[0],
        g: bot[1],
        b: bot[2],
    };
    let bg = Color::Rgb {
        r: top[0],
        g: top[1],
        b: top[2],
    };
    let pixel = "▄".with(fg).on(bg);
    io::stdout().queue(PrintStyledContent(pixel))?;
    Ok(())
}

fn get_terminal_size() -> (u16, u16) {
    match crossterm::terminal::size() {
        Ok((cols, rows)) => (cols, rows),
        Err(_) => (80, 24),
    }
}
