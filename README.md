# weathery

[![Crates.io](https://img.shields.io/crates/v/weathery.svg)](https://crates.io/crates/weathery)
[![Downloads](https://img.shields.io/crates/d/weathery.svg)](https://crates.io/crates/weathery)
[![License](https://img.shields.io/crates/l/weathery.svg)](https://github.com/VG-dev1/weathery/blob/main/LICENSE)

weathery is a terminal weather app with dynamically animated ANSI cityscapes.

It fetches a cityscape from Wikipedia, renders it in ANSI art, pulls live weather from Open Meteo, and layers on animations that respond to weather type, intensity, and time of day.

## Demo

### Stockholm (thunderstorm with heavy hail)

![Stockholm thunderstorm demo](assets/stockholm-thunderstorm-night.gif)

## Installation

### Via Cargo

```bash
cargo install weathery
```

### Via AUR

```bash
yay -S weathery
```

### Build from Source

You need Rust installed.

```bash
git clone https://github.com/VG-dev1/weathery.git
cd weathery
cargo install --path .
```

## The animations

- Different animation appearance depending on the weather condition
    - Rain: blue ANSI blocks that fall down
    - Snow: white ANSI blocks that fall down
    - Thunderstorm: same blue rain blocks as regular rain but much faster and denser, combined with periodic lightning flashes and thunderbolts
    - Clear: static image with no particles or effects
    - Fog: grayscale image; no fog: colorful image
        - This can be overriden with `--grayscale` and `--colorful` flags
- Different speed of droplets depending on the intensity of the weather
    - Light intensity: slower spawn rate and slower particle movement
    - Moderate intensity: medium spawn rate and speed
    - Heavy intensity: rapid spawning and fast particles
- Different animation appearance depedning on the time of the day
    - Day: bright image
    - Night: dark image with stars
    - This can be overriden with `--day` and `--night` flags
- Frame timing stays consistent across all weather types by measuring elapsed time and sleeping just enough to hit the target frame rate, ensuring smooth animation regardless of terminal speed

## CLI Options

### Simulate weather conditions

```bash
# Thunderstorm with heavy hail
weathery "Stockholm" --simulate 99

# Heavy rain
weathery "Stockholm" --simulate 65

# Clear
weathery "Stockholm" --simulate 0
```

## Weather Codes

The `--simulate` flag accepts any of the following codes to preview different weather conditions and their corresponding animations:

| Code | Condition | Animation |
|------|-----------|-----------|
| 0 | ☀️ Clear sky | Static, no particles |
| 1 | 🌤 Mainly clear | Static, no particles |
| 2 | ⛅ Partly cloudy | Static, no particles |
| 3 | ☁️ Overcast | Static, no particles |
| 45 | 🌫 Foggy | Grayscale rendering |
| 48 | 🌫 Depositing rime fog | Grayscale rendering |
| 51 | 🌧 Light drizzle | Slow falling droplets |
| 53 | 🌧 Moderate drizzle | Medium falling droplets |
| 55 | 🌧 Dense drizzle | Fast falling droplets |
| 61 | 🌧 Slight rain | Slow falling droplets |
| 63 | 🌧 Moderate rain | Medium falling droplets |
| 65 | 🌧 Heavy rain | Fast falling droplets |
| 71 | ❄️ Slight snow | Slow falling snow |
| 73 | ❄️ Moderate snow | Medium falling snow |
| 75 | ❄️ Heavy snow | Fast falling snow |
| 77 | ❄️ Snow grains | Fast falling snow |
| 80 | 🌧 Slight rain showers | Slow falling droplets |
| 81 | 🌧 Moderate rain showers | Medium falling droplets |
| 82 | 🌧 Violent rain showers | Fast falling droplets |
| 85 | ❄️ Slight snow showers | Slow falling snow |
| 86 | ❄️ Heavy snow showers | Fast falling snow |
| 95 | ⛈ Thunderstorm (slight/moderate) | Fast particles + lightning |
| 96 | ⛈ Thunderstorm with slight hail | Fast particles + lightning |
| 99 | ⛈ Thunderstorm with heavy hail | Fast particles + lightning |

### Force colorful/grayscale image

```bash
# Colorful
weathery "Stockholm" --colorful

# Grayscale
weathery "Stockholm" --grayscale
```

### Force day/night mode (bright/dark image)

```bash
# Day
weathery "Stockholm" --day

# Night
weathery "Stockholm" --night
```

### Use imperial units

```bash
weathery "Stockholm" --imperial
```

## Keyboard controls

- `q` - Quit

## Roadmap

- [ ] More key bindings
- [x] Imperial unit
- [ ] Auto-detect user's location
- [x] More animation settings
- [ ] Installation via other package managers

More coming soon!

## License

GPL-3.0-or-later