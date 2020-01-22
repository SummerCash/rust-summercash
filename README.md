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
