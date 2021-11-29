# Home EnViroNment

## Motivation

This is my IoT temperature and humidity monitoring solution for where i live.
I found it cheaper to go buy DHT11 sensors and hook them up to my already owned Raspberry PIs

So instead of spending 50 euros for a proprietary solution i did this.

## Where can i see this working?

Oh you can run the lumberjack program:

```sh
cargo run --bin lumberjack
```

or alternativly:

```sh
curl https://fasteraune.com/hevn
```

## How is this project organized?

- `collector` project contains the code for running on the RPI which are connected to a DHT11 sensor, same applies to the `collector_py`

- `aggregator` project contains the code for running a http server which collects the results from the collectors and sends it out to the users.

- `lumberjack` (the logger, haha get it?) is a simple program for getting data from the aggregator.

- `util` contains utility functions and structs used in the project.
