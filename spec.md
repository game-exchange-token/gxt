# GXT Specification

## Overview
GXT (Game Exchange Token) is a compact, copy-paste friendly format for sharing encrypted messages between two parties.
A GXT is a string token that starts with `gxt:` and encodes a compressed CBOR record with an embedded signature,
payload and stable id.

```
token = "gxt:" + Base58btc( Brotli( CBOR([ version, verification_key, encryption_key, kind, payload, parent, id, signature ]) ) )
```

- `version` — protocol version (currently `1`).
- `verification_key` — 32-byte Ed25519 public key of the signer for signature verification. Sent as hex string.
- `encryption_key` — 32-byte X25519 public key of the signer for encrypting messages addressed to the signer. Sent as hex string.
- `payload` — An opaque CBOR/JSON payload.
- `parent` — 32-byte BLAKE3 hash of the `id` of the parent message. Sent as hex string.
- `id` — 32-byte BLAKE3 hash of the `canonical representation` (see below). Sent as hex string.
- `signature` — 64-byte Ed25519 signature over the `b"GXT" + canonical representation`. Sent as hex string.

### canonical representation
The canonical CBOR encoding of the **same** 8-element array but with `parent`, `id` and `signature` set to empty strings:

```
canonical = CBOR([ version, verification_key, encryption_key, kind, payload, parent="", id="", signature="" ])
id        = BLAKE3(canonical)
signature = Ed25519(signing_key, b"GXT" || canonical)
```

This guarantees determinism and a stable content address (`id`).

## Payload
The payload is any JSON value. The protocol does not interpret it.

## Encoding Details
- **CBOR** — Canonical emission via `serde_cbor`.
- **Compression** — Brotli with `quality=5`, `lgwin=20`.
- **Transport** — Base58btc, prefixed with `gxt:`.

## Verification
To verify a token:
1. Strip `gxt:` and Base58-decode, then Brotli-decompress to raw CBOR.
2. Parse as a 8-element CBOR array `[version, verification_key, encryption_key, kind, payload, parent, id, signature]`.
3. Assert:
   - `version == 1`
   - `verification_key.len == 32`, `encryption_key.len == 32`, `parent.len == 32 || 0`, `id.len == 32 || 0`, `signature.len == 64 || 0`
4. Rebuild `canonical = CBOR([version, verification_key, encryption_key, kind, payload, "", "", ""])` and check:
   - `BLAKE3(canonical) == id`
   - Verify Ed25519 signature over `b"GXT" + canonical` with public key `verification_key`.

If all checks pass, the token is valid.

## Encryption
To ensure only the intended receiver can read the `payload`, messages are encrypted with the `encryption_key` of the receiver.

The `payload` of an encrypted message contains:
```
{
  "to":   <X25519 public key, 32 bytes>,
  "from": <X25519 public key, 32 bytes>,
  "alg":  "xchacha20poly1305",
  "n24":  <24-byte nonce>,
  "ct":   <ciphertext bytes>,
}
```

- The sender uses their X25519 **secret** key and the receiver's X25519 **public** key to derive a shared secret.
- A 32-byte AEAD key is derived via `BLAKE3.derive_key("GXT-ENC-XCHACHA20POLY1305", shared)`.
- The JSON/CBOR `payload` is serialized to CBOR and encrypted with XChaCha20-Poly1305.
- The outer token is still **signed with Ed25519** (authentic), so the receiver can verify the sender by the outer `verification_key`.

Decryption requires the receiver's X25519 secret key.
