# Home EnViroNment

## Motivation

This is my IoT temperature and humidity monitoring solution for where i live.
It is based on DHT11 and RaspberryPIs.

## Where can i see this working?

Oh you can run the lumberjack program:

```sh
cargo run --bin lumberjack
```

or alternativly:

```sh
curl https://fasteraune.com/hevn/read
```

It is also possible to go to [here](https://fasteraune.com/hevn/read)

## How is this project organized?

- `collector` project contains the code for running on the RaspberryPIs, which are connected to a DHT11 sensor.

- `aggregator` project contains the code for running a http server which collects the results from the collectors and sends it out to the users.

- `lumberjack` (the logger, haha get it?) is a simple program for getting data from the aggregator.

- `util` contains utility functions and structs used in the project.
