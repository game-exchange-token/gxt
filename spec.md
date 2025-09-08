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


## Encryption (optional, authenticated)
To ensure only the intended receiver can read the `body`, GXT supports an encrypted `msg` form.

The `payload` map can contain:
```
["msg", {
  "to":   <X25519 public key, 32 bytes>,
  "from": <X25519 public key, 32 bytes>,
  "enc":  { "alg":"xchacha20poly1305", "n24":<24-byte nonce>, "ct":<ciphertext bytes> },
  "parent"?: <32-byte id>
}]
```

- The sender uses their X25519 **secret** key and the receiver's X25519 **public** key to derive a shared secret.
- A 32-byte AEAD key is derived via `BLAKE3.derive_key("GXT-ENC-XCHACHA20POLY1305", shared)`.  
- The JSON/CBOR `body` is serialized to CBOR and encrypted with XChaCha20-Poly1305.
- The outer token is still **signed with Ed25519** (authentic), so the receiver can verify the sender by the outer `pk`.

Decryption requires the receiver's X25519 secret key. The receiver must verify the outer signature *before* decrypting.


### Same-key signing & encryption
GXT derives the X25519 encryption keypair **deterministically from the Ed25519 signing key**:
```text
esk, epk = DeriveX25519( ed25519_sk )
esk = X25519( BLAKE3.derive_key("GXT-ENC-X25519-FROM-ED25519", ed25519_sk_bytes) )
epk = esk * G   # X25519 scalar basepoint multiplication
```
This lets users manage a **single secret**. To allow others to encrypt to you, publish `epk`
(e.g., include it in your identity `meta`). The sender includes both `from` (their `epk`) and
`to` (your `epk`) in the encrypted message payload.
