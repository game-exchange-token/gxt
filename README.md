# GXT (Game Exchange Token)

Minimal, signed, copy-pasteable tokens for game/mod exchanges.

- Prefix: `gxt:`
- Transport: Base58btc(Brotli(CBOR))
- Structure: CBOR array `[v, vk, pk, payload, id, sig]`
- `id = blake3(bytes0)`, `sig = Ed25519("GXT1" || bytes0)`

See [`spec.md`](spec.md) and [`glossary.md`](glossary.md).

## Build

```bash
cargo build --release
```

## CLI

```bash
cargo run -- keygen -o alice.key
cargo run -- keygen -o bob.key

echo '{"name":"Alice"}' | cargo run -- id bob.key -o bob.id
echo '{"hello":"world"}' | cargo run -- msg alice.key bob.id -o msg.gxt

# Verify
cargo run -- verify "$(cat msg.token)"
```
