[![ci](https://github.com/OkieOth/rs_observable/actions/workflows/rust.yml/badge.svg)](https://github.com/OkieOth/rs_observable/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/rs_observable.svg)](https://crates.io/crates/rs_observable)

# TL;DR;

The project contains to Oberver pattern implementations.

With the `single` feature a single threaded version is available
over the `Observable` and `ObservedValue` types

The `tokio` feature contains the types `ChObservable` and
`ChObservedValue` as pattern implementations

For the full doc ...

```bash
cargo doc -F all --open
```