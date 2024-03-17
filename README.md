# Stonehold

An action game about escaping a dungeon.

This was made for a gamejam.  Code quality is generally poor.

## Dev Environment Setup

- I am developing this on a WSL instance of Ubuntu.
- I have installed rust via [rustup](https://rustup.rs/)
- I use the [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) VSCode extension, **NOT** the "rust" extension, which is known to suck.
  - I enable format on save - the rust formatter is very speedy and good
- This project targets WASM: `rustup target add wasm32-unknown-unknown`
- I run my dev server with cargo watch and basic-http-server: `cargo install cargo-watch basic-http-server`

## Running on Dev Server

You can use cargo watch to rebuild and restart the dev server on every change.

```
cargo watch -x build -s basic-http-server
```

Navigate to localhost:4000 to test.