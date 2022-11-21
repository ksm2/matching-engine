# Matching Engine

This project is a prototype for a matching engine developed in Rust.


## Table of Contents

- [Installing Rust](#installing-rust)
  * [macOS](#macos)
  * [Ubuntu](#ubuntu)
- [Running](#running)
- [Configuration](#configuration)


## Installing Rust

It is recommended to use [Rustup](https://rustup.rs/).
Rustup is a service which allows maintaining different versions of the Rust toolchain.

### macOS

To install it on macOS using Homebrew:
    
    brew install rustup

Afterwards, run
    
    rustup-init

and use the default settings.

### Ubuntu

On Ubuntu, you can use:

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

Follow the default settings.


## Running

In order to run the matching engine locally, you can use **Cargo**.
Cargo should be installed by Rustup. 

    cargo run

If everything built successfully, you should be able to see an empty order book when navigating to [http://127.0.0.1:3000](http://127.0.0.1:3000).


## Configuration

Use the environment or a `.env` file to configure the matching engine.

To get started, you can use the `.env.dist` file:

    cp .env.dist .env

All available options can be seen in the [Config](./src/config/mod.rs) struct.
