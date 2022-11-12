# facepalmcoin

A simple memecoin illustrating smart contracts in Rust/CosmWasm.

![Picard](/picard-facepalm.gif "Picard")

## Supported messages

```json
{ "instantiate": { "burn_address": "Addr", "initial_balance": Uint128 } }

{ "transfer" { "receiver": "Addr", "amount": Uint128 } }
{ "burn" { "amount": Uint128 } }

{ "get_balance": { "address": "Addr" } }
```

## Building

```bash
cargo build
```

## Generating wasm

```bash
cargo install cargo-run-script # you only need to do this once
cargo run-script optimize      # this script is defined in cargo.toml
```
