Memthol is a memory profiler for [OCaml]. This repository contains the **B**rowser **U**ser
**I**nterface (*BUI*) for memthol.

# Features

- [x] real-time/offline: inspect your program as it's running or *a posteriori*
- [x] multi-client: open several tabs in your browser for the same profiling session to visualize
    the data separately
- [x] self-contained: the BUI packs all its static assets, once you have the binary you do not need
    anything else (except a browser)
- [ ] multiple metrics to plot different kinds of data: allocation lifetime, size, time of creation,
    *etc.*
- [ ] data-splitting: plot several families of data separately in the same chart by separating them
    based on size, allocation lifetime, source locations in the allocation callstack, *etc.*

# Building

## Pre-requisites

Since memthol's UI is browser-based, it has a client written in [Rust] that compiles to [web
assembly]. To do this, you need to have the [rust toolchain]. If it's installed already, run

```
rustup update stable
```

to make sure it's up to date.

Second, for the client, you will need [cargo web] so that cargo can compile it to web assembly. With
rust/cargo installed, simply run

```
cargo install cargo-web
```

to install it. Also make sure you have the web assembly target with

```
rustup target add wasm32-unknown-unknown
```

## From sources

> **NB**: this workflow may fail because the BUI's repository is currently private. The commands
> above will fail if git's `credential.helper` does not contain the appropriate credentials.

To build memthol's BUI from the sources, clone this repository and enter it. Then, run either `cargo
build` to compile the BUI in *debug* mode or `cargo build --release` for *release* mode. The
resulting binary will be in `target/debug/memthol` in *debug* mode and `target/release/memthol` in
*release* mode.

## Using cargo

You can install memthol's BUI by running

```
cargo install --git <memthol's BUI repository>
```

To update, run the same command with `--force`

```
cargo install --force --git <memthol's BUI repository>
```

# Other Resources

- [todo list]

[OCaml]: https://ocaml.org/ (OCaml official page)
[web assembly]: https://webassembly.org/ (Web Assembly official page)
[Rust]: https://www.rust-lang.org/ (Rust official page)
[rust toolchain]: https://www.rust-lang.org/tools/install (Rust installation instructions)
[cargo web]: https://crates.io/crates/cargo-web (Cargo-web on crates.io)
[todo list]: ./todo.md