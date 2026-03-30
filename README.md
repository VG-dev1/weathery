# weathery

[![Crates.io](https://img.shields.io/crates/v/weathery.svg)](https://crates.io/crates/weathery)
[![Downloads](https://img.shields.io/crates/d/weathery.svg)](https://crates.io/crates/weathery)
[![License](https://img.shields.io/crates/l/weathery.svg)](https://github.com/VG-dev1/weathery/blob/main/LICENSE)

weathery is a terminal weather app with dynamically animated ANSI cityscapes.

It fetches a cityscape from Wikipedia, renders it in ANSI art, fetches the weather from Open Meteo, and adds animations according to the weather and the intensity of the weather.

## Demo

### Copenhagen (heavy rain)

![Copenhagen heavy rain demo](assets/copenhagen-heavy-rain.gif)

## Installation

### Via Cargo

```bash
cargo install weathery
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
    - Thunderstorm: same blue rain blocks as regular rain but much faster and denser, combined with periodic lightning flashes
    - Clear: static image with no particles or effects
- Different speed of droplets depending on the intensity of the weather
    - Light intensity: slower spawn rate and slower particle movement
    - Moderate intensity: medium spawn rate and speed
    - Heavy intensity: rapid spawning and fast particles
- In case of foggy weather, the image is grayscale; otherwise it's colorful (this can be overriden with `--grayscale` and `--colorful` flags)
- Frame timing stays consistent across all weather types by measuring elapsed time and sleeping just enough to hit the target frame rate, ensuring smooth animation regardless of terminal speed

## CLI Options

### Simulate weather conditions

```bash
# Heavy rain
weathery "Copenhagen" --simulate 65

# Clear
weathery "Copenhagen" --simulate 0
```

Weather codes:

```rust
0 => "☀️ Clear sky",
1 => "🌤 Mainly clear",
2 => "⛅ Partly cloudy",
3 => "☁️ Overcast",
45 => "🌫 Foggy",
48 => "🌫 Depositing rime fog",
51 => "🌧 Light drizzle",
53 => "🌧 Moderate drizzle",
55 => "🌧 Dense drizzle",
61 => "🌧 Slight rain",
63 => "🌧 Moderate rain",
65 => "🌧 Heavy rain",
71 => "❄️ Slight snow",
73 => "❄️ Moderate snow",
75 => "❄️ Heavy snow",
77 => "❄️ Snow grains",
80 => "🌧 Slight rain showers",
81 => "🌧 Moderate rain showers",
82 => "🌧 Violent rain showers",
85 => "❄️ Slight snow showers",
86 => "❄️ Heavy snow showers",
95 => "⛈ Thunderstorm (slight/moderate)",
96 => "⛈ Thunderstorm with slight hail",
99 => "⛈ Thunderstorm with heavy hail",
```

### Force colorful/grayscale image

```bash
# Colorful
weathery "Copenhagen" --colorful

# Grayscale
weathery "Copenhagen" --grayscale
```

### Use imperial units

```bash
weathery "Copenhagen" --imperial
```

## Keyboard controls

- `q` - Quit
- `u` - Toggle units

## Roadmap

- [ ] More key bindings
- [x] Imperial unit
- [ ] Auto-detect user's location
- [ ] More animation settings
- [ ] Installation via other package managers

More coming soon!

## License

GPL-3.0-or-later
