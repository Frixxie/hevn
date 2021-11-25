# Home EnViroNment

## Motivation

This is my IoT temperature and humidity monitoring solution for where i live.
I found it cheaper to go buy sensors and hook them up to my already owned Raspberry PIs

So instead of spending 50 euros for a proprietary solution i did this.

## Where can i see this working?

Oh! Just run:

```sh
curl https://fasteraune.com/hevn
```

For pretty formatting:

```sh
curl https://fasteraune.com/hevn | jq
```

## How is this project organized?

- The collector folder contains the code for running on the RPI which are connected to a DHT11 sensor 

- The aggregator folder contains the code for running a http server which collects the results from the collectors and sends it out to the users.
