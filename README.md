# GXT (Game Exchange Token)

Minimal, encrypted, signed and copy-pasteable tokens for manual data exchange between games.

For details check out [`spec.md`](https://github.com/game-exchange-token/gxt/blob/main/spec.md).

- [Rationale](#rationale)
- [About](#about)
- [Playground](#playground)
- [Install](#install)
- [Demo](#demo)
- [File Extensions \& Prefixes](#file-extensions--prefixes)
- [CLI](#cli)
  - [General](#general)
  - [Keygen](#keygen)
  - [Id](#id)
  - [Verify](#verify)
  - [Msg](#msg)
  - [Decrypt](#decrypt)
  - [UI](#ui)
- [C API](#c-api)
- [Extism API](#extism-api)
- [WASM API](#wasm-api)
- [C# API](#c-api-1)
- [Special Thanks](#special-thanks)

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
Both are written in rust. There is also a wrapper that exposes a C API called `gxt-api-c`, a wrapper that provides
the API as an [Extism](https://extism.org/) plugin and a C# wrapper (based on Extism).

## Playground
There is a web UI for trying it out which can be found here: [GXT Playground](https://game-exchange-token.github.io/gxt)

## Install
```bash
cargo install gxt-cli

# or if you want a simple (read-only) UI as well
cargo install gxt-cli -F ui
```

## Demo
```bash
# Create keys for communication
gxt keygen --out alice.gxk
gxt keygen --out bob.gxk

# Create an id card for bob
echo '{"name":"Bob"}' | gxt id bob.gxk --out bob.gxi --meta -

# Verify if the id card is valid and signed
gxt verify --file bob.gxi

# Create a message for bob using their id card and your own key
gxt msg --key alice.gxk --to bob.gxi --out msg_to_bob.gxm --payload '{"hello":"world"}'

# Verify if the message is valid and signed
gxt verify --file msg_to_bob.gxm

# Decrypt the message using bobs key
gxt decrypt --key bob.gxk --file msg_to_bob.gxm

# Try decrypting a message with a key its not intended for
gxt keygen --out charlie.gxk
gxt decrypt --key charlie.gxk --file msg_to_bob.gxm
```

## File Extensions & Prefixes
| Token Kind | Prefix | File Extension | Description                                                                                                                                                                                        |
| ---------- | ------ | -------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Key        | `gxk:` | `.gxk`         | A private key, used to sign messages. **DO NOT SHARE**. These are supposed to be private. If you want to exchange data with someone, send them an ID card.                                         |
| Id         | `gxi:` | `.gxi`         | An identity card, containing the necessary data to encrypt messages for the owner of the ID card. _This is derived from the private key._                                                          |
| Message    | `gxm:` | `.gxm`         | A message that is signed with a key and encrypted for a specified ID card. Once generated, the data inside can only be decrypted by the private key that was used to derive the specified ID card. |

## CLI
### General
```sh
GXT (Game Exchange Token)

Usage: gxt <COMMAND>

Commands:
  keygen   Generates a new private key
  id       Generate an ID card containing the data about a peer
  verify   Verify a message
  msg      Create an encrypted message
  decrypt  Decrypt a message
  # This command is only available if the cli was installed with the "ui" feature
  ui       Show a simple UI for opening messages
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Keygen
```sh
Generates a new private key

Usage: gxt keygen --out <OUT>

Options:
  -o, --out <OUT>  Where to store the key
  -h, --help       Print help
```

### Id
```sh
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
```sh
Verify a message

Usage: gxt verify [OPTIONS] <--msg <MSG>|--file <FILE>>

Options:
  -m, --msg <MSG>    The string token containing the message. Pass - to read from stdin
  -f, --file <FILE>  The path to the encrypted message
  -j, --json         Print output as json
  -h, --help         Print help
```

### Msg
```sh
Create an encrypted message

Usage: gxt msg [OPTIONS] --key <KEY> --to <TO> --payload <PAYLOAD>

Options:
  -k, --key <KEY>          The key of the sender
  -t, --to <TO>            The id card of the recipient
      --parent <PARENT>    The parent of this message
  -p, --payload <PAYLOAD>  The payload of the message. Can be anything, but must be set. Pass - to read from stdin
  -o, --out <OUT>          Where to store the message token
  -h, --help               Print help
```

### Decrypt
```sh
Decrypt a message

Usage: gxt decrypt [OPTIONS] --key <KEY> <--msg <MSG>|--file <FILE>>

Options:
  -k, --key <KEY>    The key of the receiver
  -m, --msg <MSG>    The string token containing the message. Pass - to read from stdin
  -f, --file <FILE>  The path to the encrypted message
  -j, --json         Print output as json
  -h, --help         Print help
```

### UI
**Only available if cli was installed with the "ui" feature enabled!**

```sh
Show a simple UI for opening messages

Usage: gxt.exe ui [PATH] [KEY]

Arguments:
  [PATH]  The message to decode
  [KEY]   The key, if the message is encrypted

Options:
  -h, --help  Print help
```

## C API
To use the C API, clone the repository and then build the crate `gxt-api-c`.
This will create a dynamic and a static library, as well as the corresponding include header,
inside the target directory.

```bash
git clone https://github.com/game-exchange-token/gxt
cd gxt
cargo build -p gxt-api-c --release
cd target/release
ls
```

## Extism API
By exposing the API as an Extism plugin its possible to use the library in every language that is supported
as a host language by Extism.

To build the crate as an Extism plugin, make sure you build from within the `gxt-api-extism` directory.
Otherwise it will try to use the JS backend for `getrandom`, which is defined as the default for wasm so that
the crate can be included in rust web projects without having to set the backend themselves.

## WASM API
If you want to import the crate into your web/node.js project, you can build the `gxt-wasm` crate with `wasm-pack`
for the target you need.

Make sure you run wasm-pack from inside the `gxt-wasm` directory so that it picks up all the optimization options.

## C# API
Ready to use C# DLL, which loads the library through Extism, so we don't have to deploy the native library.

Also available on nuget: https://www.nuget.org/packages/gxt-csharp

## Special Thanks
- [Daniel Kempf](https://kempfdaniel.de), for creating the UI for github pages