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


# Quick Note About Dump Files

Memthol-ui works on a user-provided directory, in which it expects to find

- an *init file* `init.memthol`
- some *diff files* `*.memthol.diff`

Memthol-ui relies on the **time of last modification** to sort the diff files, **not** the
lexicographical order.

Also, note that a "run" of the program you are profiling is represented by the init file, which
specifies the time at which the run started (amongst other things). Memthol-ui takes the time of
last modification of the init file as the starting point of the run, which entails that any diff
file *older* than the init file **is ignored**.

> **NB:** the reason memthol-ui works this way is because it is designed to work live, *i.e.* as the
> program is running. More precisely, memthol-ui supports stopping and re-launching the program
> while memthol-ui is running. Meaning, in general, there will be leftover diff files in the dump
> directory. These files need to be ignored for the new run, which is why memthol-ui takes the time
> of last modification of the init file as its reference point.


# Build

## Pre-requisites

Since memthol's UI is browser-based, it has a client written in [Rust] that compiles to [web
assembly] (wasm). To do this, you need to have the [rust toolchain].

- add the wasm target for rustup:

    ```bash
    rustup target add wasm32-unknown-unknown
    ```

- make sure everything is up to date with

    ```bash
    rustup update
    ```

- make sure you have [wasm-pack] installed so that Rust can compile the client

    installation instructions [here][install wasm-pack]

## From sources

> **NB**: this workflow may fail because the BUI's repository is currently private. The commands
> above will fail if git's `credential.helper` does not contain the appropriate credentials.

To build memthol's BUI from the sources, clone this repository and enter it. Then, run either `cargo
build` to compile the BUI in *debug* mode or `cargo build --release` for *release* mode. The
resulting binary will be in `target/debug/memthol` in *debug* mode and `target/release/memthol` in
*release* mode.

## Using cargo

You can install memthol's BUI by running

```bash
cargo install --git <memthol's BUI repository>
```

To update, run the same command with `--force`

```bash
cargo install --force --git <memthol's BUI repository>
```

# Testing

First, make sure you run the bash script that prepares all the test-profiling-dumps:

```bash
./rsc/scripts/prepare_dumps.sh
```

Run memthol-ui on the test files located on this repository, in `rsc/ackermann_with_sets` to make
sure it works:

- if your binary is called `memthol` and is in you path, and you are at the root of this repository:

    ```bash
    memthol rsc/ackermann_with_sets
    ```

- if you are at the root of the repo and want build-and-run the sources:

    ```bash
    cargo build -- rsc/ackermann_with_sets
    ```


# Other Resources

- [browser compatibility]
- [todo list]

# Icons

All icons come from the [bootstrap library][bootstrap].

[OCaml]: https://ocaml.org/ (OCaml official page)
[web assembly]: https://webassembly.org/ (Web Assembly official page)
[Rust]: https://www.rust-lang.org/ (Rust official page)
[rust toolchain]: https://www.rust-lang.org/tools/install (Rust installation instructions)
[wasm-pack]: https://crates.io/crates/cargo-web (Cargo-web on crates.io)
[browser compatibility]: ./docs/compatibility.md (Browser compatibility discussion)
[todo list]: ./todo.md (Todo list)
[install wasm-pack]: https://rustwasm.github.io/wasm-pack/installer (wasm-pack install instructions)
[bootstrap]: https://icons.getbootstrap.com (the bootstrap library)