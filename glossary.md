# Glossary

- **GXT** — *Game Exchange Token.* The overall format + tools for sharing signed and encrypted payloads as short strings.
- **Token** — A string that starts with `gxt:` and contains Base58btc-encoded, Brotli-compressed CBOR bytes.
- **bytes0** — The canonical CBOR encoding of the top-level array `[v, vk, pk, kind, payload, parent, id, sig]` **with `id` and `sig` set to empty byte strings**. This is what we hash and sign.
- **`v`** — Protocol version (currently `1`). Stored *inside* the CBOR array; the outside prefix stays `gxt:`.
- **`id`** — A 32-byte BLAKE3 hash of `bytes0`. Serves as a stable, content-addressed identifier for the token.
- **`parent`** — A 32-byte BLAKE3 hash of `bytes0`. Serves as a stable, content-addressed identifier for the token.
- **`sig`** — A 64-byte Ed25519 signature over `b\"GXT1\" || bytes0` (domain-separated to avoid cross-protocol reuse).
- **`vk`** — The senders verification key, which can be used to verify the signature.
- **`pk`** — The senders public key, which can be used to encrypt messages for the sender.
- **`kind`** — The senders public key.
- **`payload`** — Opaque CBOR/JSON payload of a message token..
- **ID card** — An unencrypted token for sharing ones verification and public key for further communication. Can contain optional meta data about the person that created the id card.
- **Message** — An encrypted token containing an opaque payload. Can only be decrypted by the receiver.
- **Self-addressed** — The token includes an `id` derived from its own canonical content (content addressing). This id can be used as parent for further message, allowing to build simple message chains.
