# ministerium

A lightweight orchestration and deployment agent triggered by GitHub webhooks.

## Description

`ministerium` is a small Rust-based agent for orchestrating deployments and simple automation tasks, intended to be triggered by GitHub webhooks.

## Requirements

- Rust and Cargo (stable)

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run --release
```

If this repository builds a binary, the produced executable is located in `target/release` after a release build.

## Test

```bash
cargo test
```

## Contributing

Contributions are welcome. Please open issues or pull requests with a clear description of the change.

## License

Specify a license for this project (e.g. MIT or Apache-2.0) in `LICENSE`.
