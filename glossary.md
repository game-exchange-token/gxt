# Glossary

- **GXT** — *Game Exchange Token.* The overall format + tools for sharing signed and encrypted payloads as short strings.
- **Token** — A string that starts with `gxt:` and contains Base58btc-encoded, Brotli-compressed CBOR bytes.
- **bytes0** — The canonical CBOR encoding of the top-level array `[v, vk, pk, kind, payload, id, sig]` **with `id` and `sig` set to empty byte strings**. This is what we hash and sign.
- **`v`** — Protocol version (currently `1`). Stored *inside* the CBOR array; the outside prefix stays `gxt:`.
- **`id`** — A 32-byte BLAKE3 hash of `bytes0`. Serves as a stable, content-addressed identifier for the token.
- **`sig`** — A 64-byte Ed25519 signature over `b\"GXT1\" || bytes0` (domain-separated to avoid cross-protocol reuse).
- **`vk`** — The senders verification key.
- **`pk`** — The senders public key.
- **`payload`** — A two-slot CBOR array of the form `["id", meta]` or `["msg", { parent?, body? }]`.
  - **`meta`** — Opaque CBOR/JSON value carried by an identity token.
  - **`parent`** — Optional 32-byte `id` of a previous token (threading or referencing).
  - **`body`** — Opaque CBOR/JSON payload of a message token.
- **Identity token** — A token whose payload is `["id", meta]`. Used to share a public identity with optional metadata.
- **Message token** — A token whose payload is `["msg", { parent?, body? }]`. Used for trades, quests, etc.
- **Domain separation** — Prefixing the signature preimage with a constant (`GXT1`) to prevent signature reuse across contexts.
- **Self-addressed** — The token includes an `id` derived from its own canonical content (content addressing).


- **X25519** — Diffie-Hellman key agreement over Curve25519, used to derive a shared secret for encryption.
- **XChaCha20-Poly1305** — An AEAD cipher (encryption + authentication) with a 24-byte nonce; used to encrypt `body`.
- **Encryption keypair (ek/esk/epk)** — A separate X25519 keypair for encryption (esk=secret, epk=public). Keep `esk` private.


- **Same-key signing & encryption** — We keep one user secret (Ed25519). The X25519 encryption secret/public (`esk`,`epk`) are **derived** deterministically from this secret using BLAKE3, so users don’t manage a second key.
