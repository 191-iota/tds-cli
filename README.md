# tds - Transport Duration Switzerland

`tds` compares how long a trip takes by public transport versus by car between two places in Switzerland, from the command line. It prints the fastest train connection — each leg with its times and platform — next to the road driving time, so you can see both side by side.

Public-transport data comes from [transport.opendata.ch](https://transport.opendata.ch); driving times and geocoding come from [OpenRouteService](https://openrouteservice.org). The two routes are looked up independently, so if one fails the other is still shown.

## Usage

```sh
tds "<from>" "<to>"
```

Locations can be plain place names or full addresses:

```sh
tds "Chur" "Lausanne"
```

```
Optimal travel time by train: 255 min | Transfers: 1
04:11-05:48 | IR 35 | [Chur] → [Zürich HB] | Platform: 5
06:04-08:26 | IC 5 | [Zürich HB] → [Lausanne] | Platform: 14

Estimated travel time by vehicle: 196 min
```

## Install

```sh
cargo install --path .
```

## API key

The car comparison uses OpenRouteService, which needs a free API key. Put it in a `.env` file in the directory you run `tds` from:

```
ORS_API_KEY=your_api_key_here
```

The train route works without a key; only the car comparison needs one.

## License

MIT.
