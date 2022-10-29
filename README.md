# Authsome

Authencation on Fuel

## Running the project locally (dev)

  1. Run your Fuel devnode locally
  cd into `ETHLisbon22`
  run `fuel-core run --ip 127.0.0.1 --port 4000 --chain ./chainConfig.json --db-path ./.fueldb`

  2. cd into `contracts`
  run `forc build`
  run `forc deploy --url localhost:4000 --unsigned`

  3. cd into `client`
  run `npm run dev`