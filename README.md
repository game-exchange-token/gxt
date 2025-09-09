# GXT (Game Exchange Token)

Minimal, signed, copy-pasteable tokens for game/mod exchanges.

See [`spec.md`](spec.md) and [`glossary.md`](glossary.md).

## Install

```bash
cargo install gxt-cli
```

## Demo

```bash
# Create keys for communication
gxt keygen -o alice.key
gxt keygen -o bob.key

# Create an id card for bob
echo '{"name":"Bob"}' | gxt id bob.key -o bob.id

# Create a message for bob using their id card and your own key
echo '{"hello":"world"}' | gxt msg alice.key bob.id -o msg_to_bob.gxt

# Verify if the message is valid and signed
gxt verify msg_to_bob.gxt

# Decrypt the message using bobs key
gxt decrypt bob.key msg_to_bob.gxt

# Try decrypting a message with a key its not intended for
gxt keygen -o charlie.key
gxt decrypt charlie.key msg_to_bob.gxt
```

## CLI