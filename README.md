# Transport Duration Switzeraland

**TDS** is a command-line tool to compare travel durations between two locations in Switzerland, using both public transport and car routes. It leverages live data from [transport.opendata.ch](https://transport.opendata.ch) and [OpenRouteService](https://openrouteservice.org).

## Why use TDS?

When you need to make fast, informed decisions about how to get from point A to point B, most apps force you to sift through results and compare transport modes manually.  
TDS gives you instant, terminal-based insight: the fastest public transport route—transfers and breakdown included—side-by-side with real-time road duration.

## Features

- Compares public transport vs car travel durations
- Displays detailed rail/bus/walk segments including times and platforms
- Supports natural language locations and full addresses
- Uses real-time connection data and geocoding

## Example

**From:** Zürichstrasse 46, Winterthur  
**To:** Avenue de la Gare 1, Lausanne

```
Estimated optimal travel time by train: 193 min | Transfers: 3
14:26-14:30 | walk → Zürich, Friedrichstrasse
14:30-14:38 | B 75 → Zürich Oerlikon, Bahnhof Ost from Zürich, Friedrichstrasse (Platform )
14:38-14:43 | walk → Zürich Oerlikon
14:44-14:51 | IR 13 → Zürich HB from Zürich Oerlikon (Platform 3)
15:04-17:26 | IC 5 → Lausanne from Zürich HB (Platform 32)
17:26-17:31 | walk → Lausanne, gare
17:32-17:36 | B 1 → Lausanne, Georgette from Lausanne, gare (Platform A)
17:36-17:39 | walk → Lausanne, Av. de la Gare 1

Estimated travel time by vehicle: 166 min
```

## Installation

```sh
cargo install --path .
```

## Usage

```sh
tds "<from location>" "<to location>"
```

Example:

```sh
tds "Winterthur" "Lausanne"
```

## Environment Variables

Create a `.env` file with a free API key from OpenRouteService:

```
ORS_API_KEY=your_api_key_here
```

Only the car comparison uses this key — the train route works without one.

## License

MIT License
