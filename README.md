# GXT (Game Exchange Token)

Minimal, signed, copy-pasteable tokens for game/mod exchanges.

See [`spec.md`](spec.md) and [`glossary.md`](glossary.md).

- [Rationale](#rationale)
- [About](#about)
- [Install](#install)
- [Demo](#demo)
- [CLI](#cli)
  - [General](#general)
  - [Keygen](#keygen)
  - [Id](#id)
  - [Msg](#msg)
  - [Decrypt](#decrypt)
  - [Verify](#verify)
  - [Decrypt-file](#decrypt-file)
  - [Verify-file](#verify-file)

## Rationale
I was thinking about how it could be possible to add trading
between two players to a singleplayer game as part of a mod. Mostly out of curiosity to see
if it was doable or too much work. At first I thought about having a server that manages
the trades, but then I thought that not everybody can or wants to set up a server.

Thats also when I had the idea to package the data into string tokens that can be sent
via discord and started researching how to make this somewhat secure and
easy to use and implement.

With the current design, every message is signed and encrypted for a designated receiver.
This prevents people from fulfilling a trade request and then sending the fulfillment to
50 people who all collect the rewards. Its still not as secure as server side validation,
but thats okay for me.

While working on this, I also realized that there is potential for more than just trading,
so I removed all the trade specific fields and the protocol now takes an opaque payload
that can contain any valid json value. (Strings, Numbers, Maps, etc.)

## About
The protocol uses an Ed25519 key pair for signing messages and to derive a X25519 key pair
from encryption.

The size of the token before encoding is limited to 64KB.

Because this is intended to be easy to integrate by mod authors, a library and cli are provided.
Both are written in rust, but I plan on providing wrappers for other languages as well. The library
can also compile to wasm, making it possible to use this in a web context.

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
### General
```
GXT (Game Exchange Token)

Usage: gxt <COMMAND>

Commands:
  keygen        Generates a new private key
  id            Generate an ID card that contains the public keys of the sender and some optional meta data
  msg           Create an encrypted message for a receiver
  decrypt       Decrypt a message passed as argument or read from stdin
  verify        Verify a message passed as argument or read from stdin
  decrypt-file  Decrypt a message read from a file
  verify-file   Verify a message read from a file
  help          Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Keygen
```
Generates a new private key

Usage: gxt keygen --out <OUT>

Options:
  -o, --out <OUT>  Where to store the key
  -h, --help       Print help
```

### Id
```
Generate an ID card that contains the public keys of the sender and some optional meta data

Usage: gxt.exe id [OPTIONS] --meta <META> <KEY>

Arguments:
  <KEY>  The key of the person creating the id card

Options:
  -m, --meta <META>  Meta data for the id card. Can be anything, but must be set. Pass - to read from stdin    
  -o, --out <OUT>    Where to store the id card token
  -h, --help         Print help
```

### Msg
```
Create an encrypted message for a receiver

Usage: gxt.exe msg [OPTIONS] --key <KEY> --to <TO> --body <BODY>

Options:
  -k, --key <KEY>        The key of the sender
  -t, --to <TO>          The id card of the recipient
  -p, --parent <PARENT>  The parent of this message
  -b, --body <BODY>      The body of the message. Can be anything, but must be set. Pass - to read from stdin  
  -o, --out <OUT>        Where to store the message token
  -h, --help             Print help
```

### Decrypt
```
Decrypt a message passed as argument or read from stdin

Usage: gxt.exe decrypt --key <KEY> <MSG>

Arguments:
  <MSG>  The string token containing the encrypted message. Pass - to read from stdin

Options:
  -k, --key <KEY>  The key of the receiver
  -h, --help       Print help
```

### Verify
```
Verify a message passed as argument or read from stdin

Usage: gxt.exe verify <MSG>

Arguments:
  <MSG>  The string token containing the message. Pass - to read from stdin

Options:
  -h, --help  Print help
```

### Decrypt-file
```
Decrypt a message read from a file

Usage: gxt.exe decrypt-file --key <KEY> <MSG>

Arguments:
  <MSG>  The path to the encrypted message

Options:
  -k, --key <KEY>  The key of the receiver
  -h, --help       Print help
```

### Verify-file
```
Verify a message read from a file

Usage: gxt.exe verify-file <MSG>

Arguments:
  <MSG>  The path to the message

Options:
  -h, --help  Print help
```
