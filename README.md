# GXT (Game Exchange Token)

Minimal, encrypted, signed and copy-pasteable tokens for manual data exchange between games.

See [`spec.md`](spec.md) and [`glossary.md`](glossary.md).

- [Rationale](#rationale)
- [About](#about)
- [Install](#install)
- [Demo](#demo)
- [CLI](#cli)
  - [General](#general)
  - [Keygen](#keygen)
  - [Id](#id)
  - [Verify](#verify)
  - [Msg](#msg)
  - [Decrypt](#decrypt)

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
gxt keygen --out alice.key
gxt keygen --out bob.key

# Create an id card for bob
echo '{"name":"Bob"}' | gxt id bob.key --out bob.id --meta -

# Create a message for bob using their id card and your own key
gxt msg --key alice.key --to bob.id --out msg_to_bob.gxt --body '{"hello":"world"}'

# Verify if the message is valid and signed
gxt verify --file msg_to_bob.gxt

# Decrypt the message using bobs key
gxt decrypt --key bob.key --file msg_to_bob.gxt

# Try decrypting a message with a key its not intended for
gxt keygen --out charlie.key
gxt decrypt --key charlie.key --file msg_to_bob.gxt
```

## CLI
### General
```
GXT (Game Exchange Token)

Usage: gxt <COMMAND>

Commands:
  keygen   Generates a new private key
  id       Generate an ID card containing the data about a peer
  verify   Verify a message
  msg      Create an encrypted message
  decrypt  Decrypt a message
  help     Print this message or the help of the given subcommand(s)

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
Generate an ID card containing the data about a peer

Usage: gxt id [OPTIONS] --meta <META> <KEY>

Arguments:
  <KEY>  The key of the person creating the id card

Options:
  -m, --meta <META>  Meta data for the id card. Can be anything, but must be set. Pass - to read from stdin    
  -o, --out <OUT>    Where to store the id card token
  -h, --help         Print help
```

### Verify
```
Verify a message

Usage: gxt.exe verify [OPTIONS] <--msg <MSG>|--file <FILE>>

Options:
  -m, --msg <MSG>    The string token containing the message. Pass - to read from stdin
  -f, --file <FILE>  The path to the encrypted message
  -j, --json         Print output as json
  -h, --help         Print help
```

### Msg
```
Create an encrypted message

Usage: gxt msg [OPTIONS] --key <KEY> --to <TO> --body <BODY>

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
Decrypt a message

Usage: gxt.exe decrypt [OPTIONS] --key <KEY> <--msg <MSG>|--file <FILE>>

Options:
  -k, --key <KEY>    The key of the receiver
  -m, --msg <MSG>    The string token containing the message. Pass - to read from stdin
  -f, --file <FILE>  The path to the encrypted message
  -j, --json         Print output as json
  -h, --help         Print help
```
