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
gxt keygen -o alice.key
gxt keygen -o bob.key

echo '{"name":"Bob"}' | gxt id bob.key -o bob.id

echo '{"hello":"world"}' | gxt msg alice.key bob.id -o msg_to_bob.gxt

gxt verify msg_to_bob.gxt

gxt decrypt bob.key msg_to_bob.gxt

gxt keygen -o charlie.key

gxt decrypt charlie.key msg_to_bob.gxt
```