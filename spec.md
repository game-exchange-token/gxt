# GXT Specification

## Overview
GXT (Game Exchange Token) is a compact, copy-paste friendly format for sharing encrypted messages between two parties.
A GXT is a string token that starts with `gxt:` and encodes a compressed CBOR record with an embedded signature, payload and stable id.

```
token = "gxt:" + Base58btc( Brotli( CBOR([ v, vk, pk, kind, payload, parent, id, sig ]) ) )
```

- `v` — protocol version (currently `1`).
- `vk` — 32-byte Ed25519 public key of the signer for signature verification. Sent as hex string.
- `pk` — 32-byte X25519 public key of the signer for encrypting messages addressed to the signer. Sent as hex string.
- `payload` — An opaque CBOR/JSON payload.
- `parent` — 32-byte BLAKE3 hash of the `id` of the parent message. Sent as hex string.
- `id` — 32-byte BLAKE3 hash of `bytes0` (see below). Sent as hex string.
- `sig` — 64-byte Ed25519 signature over `b"GXT" + bytes0`. Sent as hex string.

### bytes0
`bytes0` is the canonical CBOR encoding of the **same** 8-element array but with `parent`, `id` and `sig` set to empty strings:

```
bytes0 = CBOR([ v, vk, pk, kind, payload, parent="", id="", sig="" ])
id    = BLAKE3(bytes0)
sig   = Ed25519(signing_key, b"GXT" || bytes0)
```

This guarantees determinism and a stable content address (`id`).

## Payload
The payload is any CBOR/JSON value. The protocol does not interpret it.

## Encoding Details
- **CBOR** — Canonical emission via `serde_cbor`.
- **Compression** — Brotli with `quality=5`, `lgwin=20`.
- **Transport** — Base58btc, prefixed with `gxt:`.

## Verification
To verify a token:
1. Strip `gxt:` and Base58-decode, then Brotli-decompress to raw CBOR.
2. Parse as a 5-element CBOR array `[v, vk, pk, kind, payload, parent, id, sig]`.
3. Assert:
   - `v == 1`
   - `vk.len == 32`, `pk.len == 32`, `parent.len == 32 || 0`, `id.len == 32 || 0`, `sig.len == 64 || 0`
4. Rebuild `bytes0 = CBOR([v, vk, pk, kind, payload, "", "", ""])` and check:
   - `BLAKE3(bytes0) == id`
   - Verify Ed25519 signature over `b"GXT" + bytes0` with public key `vk`.

If all checks pass, the token is valid.

## Encryption
To ensure only the intended receiver can read the `payload`, messages are encrypted with the `pk` of the receiver.

The `payload` of an encrypted message contains:
```
{
  "to":   <X25519 public key, 32 bytes>,
  "from": <X25519 public key, 32 bytes>,
  "enc":  { "alg":"xchacha20poly1305", "n24":<24-byte nonce>, "ct":<ciphertext bytes> },
}
```

- The sender uses their X25519 **secret** key and the receiver's X25519 **public** key to derive a shared secret.
- A 32-byte AEAD key is derived via `BLAKE3.derive_key("GXT-ENC-XCHACHA20POLY1305", shared)`.
- The JSON/CBOR `payload` is serialized to CBOR and encrypted with XChaCha20-Poly1305.
- The outer token is still **signed with Ed25519** (authentic), so the receiver can verify the sender by the outer `vk`.

Decryption requires the receiver's X25519 secret key. The receiver must verify the outer signature *before* decrypting.
