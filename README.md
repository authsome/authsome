# Authsome

## Overview

Authsome implements a multi-signature wallet using the predicate system of the Fuel VM and Sway programming language.
This multi-signature wallet is then used as the basis for an pluggable auth infrastructure, similar to [Web3Auth](https://web3auth.io/docs/overview/what-is-web3auth).

Fuel predicates work in a similar way to scripts for pay-to-script-hash (P2SH) transactions in Bitcoin.
This offers a number of important advantages over implementing account abstraction on the smart contract layer:

- reduced gas cost, as signature aggregation can be done off-chain and verification can be done as a single step without smart contract function calls;
- improved flexibility, because different signature schemes can be swapped in by implementing them as on-chain smart contract libraries and using them in predicates; and
- minimal state bloat, because predicate byte code is processed once as part of the transaction validation, and then discarded - thus never entering the blockchain state.

Another use case that could be implemented in the future is the ability to pay transaction fees with a non-default token.

## Using Fuel/Sway

  1. Run your Fuel devnode locally
  cd into `authsome`
  run `fuel-core run --ip 127.0.0.1 --port 4000 --chain ./chainConfig.json --db-path ./.fueldb`

  2. cd into `contracts`
  run `forc build`
  run `forc deploy --url localhost:4000 --unsigned`

  3. cd into `client`
  run `npm run dev`

## System

### Frontend Application

The frontend application is responsible for three aspects:

1. Generation of multi-sig wallets.
2. Management of the private keys related to multi-sig wallets.
3. Spending of funds from multi-sig wallets.

The frontend application will be deployed on [Fleek](https://fleek.co/).

#### Wallet Generation

Steps:

1. Generate three ECDSA private keys (secp256k1).
2. Send request with the three corresponding public keys to the wallet creation endpoint on the backend service.
3. Store the address for the generated multi-sig wallet.

#### Key Management

The private keys for the wallet can be managed in a number of ways to enable use cases such as social recovery:

- store the private key in a text file, or print it on a sheet of physical paper;
- store the private key in the secure local storage of the web browser;
- store the key in a user account, such as a Google account; or
- store the key on the service in password-encrypted format.

See [Web3Auth](https://tech.tor.us/#gen-key-sec) to see a simple interface of their pluggable auth infrastructure, with these example methods of key management.

#### Spending funds

Steps:

1. Specificy wallet address to list wallet assets. To do that, retrieve all UTXOs for the wallet, and add up the amounts of coins for each asset ID.
2. Specify asset ID, amount of coins and destination address for transfer. Collect sufficient number of UTXOs from the retrieved list to spend given amount for given asset ID (if possible, otherwise error).
3. For each selected UTXO, concatenate and hash the transaction ID and the output index to obtain the message hash to be signed.
4. For each message hash, sign the message hash with at least two of the three private keys associated with the given wallet address.
5. Send the request for the transaction creation to the backend service, which will properly assemble the transaction and submit it to the network.
6. Store the transaction ID for the generated transaction.

For signature verification:

- msg_hash = sha256(txid + output_index)
- signature = sign(private_key, msg_hash)

### Backend Service

The backend service is written in Rust and makes two endpoints available:

- generation of multi-sig wallet; and
- spending of funds from multi-sig wallet.

The reason we need to implement this in a backend service, instead of using the TypeScript SDK directly in the frontend, is two-fold:

1. Wallet generation requires compilation of Sway code to obtain the predicate byte code, which in turn gives us the multi-sig wallet address (as a hash of the byte code). This would be hard to do in the front-end.
2. The TypeScript SDK has no support for predicates, which would make it impossible to spend funds from the multi-sig wallet. There is rudimentary support for predicates in the Rust SDK which we can buid on.

#### Wallet Generation Endpoint

The wallet generation endpoint uses simple text-substitution of placeholder with the public keys to hard-code them into the predicate, and then passes the resulting predicate Sway code through the Sway compiler to obtain the predicate byte code.

The byte code can then be loaded into the Rust SDK to easily obtain the address associated with the predicate, and thus with the multi-sig wallet.

Request:

```json
{
    "public_keys": [
        "0xa31717e6df5953d02da07d76f512047dd8f1a0a44c609a453a89e0ee8bb9420a",
        "0xbdea8d2c7dae7872fd05cb87db5173731be69b5f19119dae47c3b2e4ab4ff025",
        "0x4525be56545024dae35ae584f29ae27d3befef1431663966d7c4a8da40bc47f8"
    ]
}
```

Response:

```json
{
    "public_keys": [
        "0xa31717e6df5953d02da07d76f512047dd8f1a0a44c609a453a89e0ee8bb9420a",
        "0xbdea8d2c7dae7872fd05cb87db5173731be69b5f19119dae47c3b2e4ab4ff025",
        "0x4525be56545024dae35ae584f29ae27d3befef1431663966d7c4a8da40bc47f8"
    ],
    "address": "bc1qu8de7xwv9yq6tw7wgfa8c04pe9gfk6mk6pq63f"
}
```

#### Fund Spending Endpoint

Request:

- *might want to use utxoid instead of txid and output index here*
- *not sure about the format of the signatures, is it 256 bits too?*
```json
{
    "address": "bc1qu8de7xwv9yq6tw7wgfa8c04pe9gfk6mk6pq63f",
    "amount": 12345,
    "inputs": [
        {
            "txid": "0xec54e0a8f1c0d9d530fd2e8d673c86904c052901de5331637feb825efba56e3f",
            "output_index": 2,
            "signatures": [
                "0xf5bdc9fabef781acfb50ca61aa9976c4b750ef168488d1dff276b89f76b8b874",
                "0x7f0a914a0ccee898dc28321ae32626169c1141cc1ed9da59e776e41df9e0611f"
            ]
        },
        {
            "txid": "0xa48924918dd9065f15725c40cb6a40c5da0a4ac46bcca1f62fa1245887bbde59",
            "output_index": 7,
            "signatures": [
                "0x089ccc4f0ea9609dba0bd40ca8aa7f148204cf66d3e0f32207522709a1ab2739",
                "0x934595f321909ad6e5e16d6d057e9c8ccd53824ab38bcd776aacba4dd800474f"
            ]
        },
        {
            "txid": "0x88003bfd24d1ad49ed295f630214e30e8193fb459885e757e0228b9a84db37d0",
            "output_index": 0,
            "signatures": [
                "0x05d16114d4802da3e66be00c9b63e61533d9f753f8ed4343401f53f6c6ee5247",
                "0x78eec62c8d13b96be3aa14b8944a27e9f7fa2c7b38295494b02eb2a4905d3eb8"
            ]   
        }
    ]
}
```

Response:

```json
    "address": "bc1qu8de7xwv9yq6tw7wgfa8c04pe9gfk6mk6pq63f",
    "amount": 12345,
    "inputs": [
        {
            "txid": "0xec54e0a8f1c0d9d530fd2e8d673c86904c052901de5331637feb825efba56e3f",
            "output_index": 2
        },
        {
            "txid": "0xa48924918dd9065f15725c40cb6a40c5da0a4ac46bcca1f62fa1245887bbde59",
            "output_index": 7
        },
        {
            "txid": "0x88003bfd24d1ad49ed295f630214e30e8193fb459885e757e0228b9a84db37d0",
            "output_index": 0
        }
    ],
    "txid": "0xc56aaa25a17409858d1e6d8ea7a5e9eb606f50898385ec3eb0deac213149a199"
}
```

### Rust SDK Extension

The Rust SDK for Fuel only has limited support for predicates.
The `spend_predicate` function found on its wallet implementation only supports a single value for the predicate data for all inputs available on the wallet address.
This is meaningless, as it would mean that the signature for every input we want to spend is always going to be the same.

In order to have proper security and a meaningful application, we have to sign each input of the transaction separately, using its txID and output index.
This means we need the ability to submit distinct predicate data for each unsigned transaction output we want to spend.

#### Predicate Spend Library

We are therefore creating a dedicated library that allows spending multiple unsigned transaction outputs for a predicate wallet address, with distinct predicate data each:

https://github.com/authsome/fuel-spend-predicate

It will use the same provider as the Rust SDK wallet is based on, and implement a barebones version of creating the desired transaction.

### Sway Multisig Predicate

The predicate will be included with every input that is part of a transaction being sent by the multi-sig wallet.
In order to spend a transaction output that was sent to a predicate, we need to provide the predicate byte code, as well as the predicate inputs needed for the byte code to yield a return value of `true`.

#### Multi-sig Wallet

As a basis for the multi-sig wallet using a predicate, we base ourselves on the example from the Sway documents:

https://fuellabs.github.io/fuels-rs/v0.27.0/predicates/send-spend-predicate.html

However, we can see the predicate that verifies the signatures. However, we can see that they simply sign an empty 256-bit array (`ZERO_B256`).

This would mean that the required signature is the same _every time_ that the predicate is re-used in an input.
In order to verify that every input we are spending has been approved, we need to collect a different signature from each key for each input.

This can be accomplished by using the unique identifiers of an input: the transaction ID it originates from, and the output index at which it is found in that transaction.

#### Verifying Signatures

As stated above, we need to verify each signature against the txID and output index of the input we are spending.
This means that the predicate needs to be aware of which input it is included in, and derive the txID and output index where it was previously spend from it.

This can be accomplished with three opcode calls on the Fuel VM:

1. GM: Get Metadata
    * Doc: https://github.com/FuelLabs/fuel-specs/blob/master/specs/vm/instruction_set.md#gm-get-metadata
    * Operation: `GM_GET_VERIFYING_PREDICATE` (`0x00003`)
    * Effect: gets the input index of the input in which the currently verifying predicate was included

2. GTF: Get transaction fields
    * Doc: https://github.com/FuelLabs/fuel-specs/blob/master/specs/vm/instruction_set.md#gtf-get-transaction-fields
    * Operation: `GTF_INPUT_COIN_TX_ID` (`0x102`)
    * Effect: gets the txID that the input originates from

3. GTF: Get transaction fields
    * Doc: https://github.com/FuelLabs/fuel-specs/blob/master/specs/vm/instruction_set.md#gtf-get-transaction-fields
    * Operation: `GTF_INPUT_COIN_OUTPUT_INDEX` (`0x103`)
    * Effect: gets the output index that the input originates from

Once we have the txID and the output index, we can replicate the code from the frontend generating the signatures to create the message hash, and recover the public keys:

- msg_hash = sha256(txid + output_index)
- address = recover(signature, msg_hash)

By comparing the public keys of the multi-sig wallet, as hard-coded into the predicate upon generation of the predicate code, to the recovered public keys, we can tell if the signature is valid.

## Possible extensions

- Key backup on IPFS
- Key backup on Arweave
- Key on 'social service'
- deploy on `beta-1` testnet

## Resources

- Fleek: https://fleek.co/
- Diagrams: https://excalidraw.com/#room=63741e129ddce8400841,63K7CIJrCUbNUqGkGlaFtw 
- TypeScript SDK: https://fuellabs.github.io/fuels-ts
- Fuel Wallet: https://github.com/FuelLabs/fuels-wallet
- Predicate Example: https://fuellabs.github.io/fuels-rs/v0.27.0/predicates/send-spend-predicate.html
- Transaction Overview: https://docs.rs/fuels/latest/fuels/tx/index.html
- Sway Documents: https://fuellabs.github.io/sway/v0.27.0/index.html