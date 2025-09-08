# GXT (Game Exchange Token)

Minimal, signed, copy-pasteable tokens for game/mod exchanges.

- Prefix: `gxt:`
- Transport: Base58btc(Brotli(CBOR))
- Structure: CBOR array `[v, pk, payload, id, sig]`
- `id = blake3(bytes0)`, `sig = Ed25519("GXT1" || bytes0)`

See [`spec.md`](spec.md) and [`glossary.md`](glossary.md).

## Build

```bash
cargo build --release
```

## CLI

```bash
# Keygen (prints sk-hex and pk-hex)
cargo run -- keygen

# Identity (meta from stdin; empty stdin => null)
echo '{"name":"Alice"}' | cargo run -- id <sk-hex> > alice.token

# Message (body from stdin; option parent id)
echo '{"type":"trade.offer/1"}' | cargo run -- msg <sk-hex> > msg.token

# Verify
cargo run -- verify "$(cat msg.token)"
```
