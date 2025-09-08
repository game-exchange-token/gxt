# GXT Specification

## Overview
GXT (Game Exchange Token) is a compact, copy-paste friendly format for sharing signed messages between players and mods.
A GXT is a string token that starts with `gxt:` and encodes a compressed CBOR record with an embedded signature and stable id.

```
token = "gxt:" + Base58btc( Brotli( CBOR([ v, pk, payload, id, sig ]) ) )
```

- `v` — protocol version (currently `1`).
- `pk` — 32-byte Ed25519 public key of the signer.
- `payload` — either `["id", meta]` or `["msg", { parent?, body? }]`.
- `id` — 32-byte BLAKE3 hash of `bytes0` (see below).
- `sig` — 64-byte Ed25519 signature over `b"GXT1" || bytes0`.

### bytes0
`bytes0` is the canonical CBOR encoding of the **same** 5-element array but with `id` and `sig` set to empty byte strings:

```
bytes0 = CBOR([ v, pk, payload, id="", sig="" ])
id    = BLAKE3(bytes0)
sig   = Ed25519(signing_key, b"GXT1" || bytes0)
```

This guarantees determinism and a stable content address (`id`).

## Payloads
### Identity
```
payload = ["id",  meta]
```
- `meta` is any CBOR/JSON value (object, array, string, number, bool, null). It is opaque to the protocol.

### Message
```
payload = ["msg", { parent?: <32-byte id>, body?: any }]
```
- `parent` is optional and can point to any previous token `id`.
- `body` is any CBOR/JSON value. The protocol does not interpret it.

## Encoding Details
- **CBOR** — Canonical emission via `serde_cbor`.
- **Compression** — Brotli with `quality=5`, `lgwin=20`.
- **Transport** — Base58btc, prefixed with `gxt:`.

## Verification
To verify a token:
1. Strip `gxt:` and Base58-decode, then Brotli-decompress to raw CBOR.
2. Parse as a 5-element CBOR array `[v, pk, payload, id, sig]`.
3. Assert:
   - `v == 1`
   - `pk.len == 32`, `id.len == 32`, `sig.len == 64`
   - `payload` is `["id", meta]` *or* `["msg", map]`
4. Rebuild `bytes0 = CBOR([v, pk, payload, "", ""])` and check:
   - `BLAKE3(bytes0) == id`
   - Verify Ed25519 signature over `b"GXT1" || bytes0` with public key `pk`.

If all checks pass, the token is valid.

## Design Rationale
- **Simplicity** — No clocks, expiry, audience, or delegation chains. Mods decide semantics using `body` and `parent`.
- **Determinism** — Using an array instead of a map avoids key-ordering ambiguity.
- **Portability** — Base58 strings paste cleanly into chats; Brotli keeps them short.
- **Security** — Domain-separated signatures and content addressing prevent mixups and make replay tracking explicit.

## Interop Notes
- The protocol does not forbid floats in `meta`/`body`, but tools may avoid them for determinism. Prefer integers/strings/booleans.
- `parent` is optional and advisory; mods can interpret it as a thread or dependency edge.
- Tokens are immutable; any edit produces a new `id`.
