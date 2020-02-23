# rust-summercash

A rust implementation of SummerTech's in-house decentralized, zero-fee, instant digital currency.

## Getting Started

This repository contains a few tools you can use to get started playing around with SummerCash:
* `summercash` - A Rust library for interacting with the SummerCash network. Gives you full access to all SummerCash features.
* `smcd` - The SummerCash daemon. This is a service that is designed to run with zero use interaction, all while maintaining a constant, up-to-date connection to the global SummerCash network.
* `smcli` - The SummerCash command-line client. Lets you give directions to a `smcd` instance (e.g. create an account, issue a transaction, etc...)

The latter of these two tools can be installed with `cargo install summercash --bin smcd` / `cargo install summercash --bin smcli`, while the SummerCash library can be installed by simply adding SummerCash as a dependency in your `Cargo.toml` file as such:

```toml
summercash = "0.1.0"
```

Alternatively, install all three of these tools by running the following commands:

```bash
git clone https://github.com/SummerCash/rust-summercash && cd rust-summercash
cargo install --path .
```

After running this sequence of commands, `smcd` and `smcli` will be available for use from the `$PATH`, provided that `~/.cargo/bin` is in such an environment variable.

### Hello, SummerCash!

To get started with rust-summercash, make sure you've got an up-to-date installation of `smcd` and `smcli` installed locally. Then, start the SummerCash node daemon by running:

```bash
smcd
```

and create an account by calling `smcli create account`. If you wish to encrypt your private key file, run `smcli lock account <address>`.

Should one wish to send a transaction from this new account, use `smcli create transaction <address created in last step> <recipient address> <number of finks> <message>`.
Keep in mind, values of SMC are expressed in finks, where `1000000000000000000 finks = 1 SMC`.

After having created a transaction, one must first sign and then publish this transaction. This can be achieved through the following sequence of commands:

```bash
smcli sign transaction <hash> <sender address>
smcli publish transaction <hash>
```
