Memthol is a memory profiler for [OCaml]. This repository contains the **B**rowser **U**ser
**I**nterface (*BUI*) for memthol.

# Features

- [x] real-time/offline: inspect your program as it's running or *a posteriori*
- [x] multi-client: open several tabs in your browser for the same profiling session to visualize
    the data separately
- [x] self-contained: the BUI packs all its static assets, once you have the binary you do not need
    anything else (except a browser)
- [x] data-splitting: plot several families of data separately in the same chart by separating them
    based on size, allocation lifetime, source locations in the allocation callstack, *etc.*
- [ ] multiple metrics to plot different kinds of data: allocation lifetime, size, time of creation,
    *etc.*


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

    installation instructions [here][install wasm-pack], although `cargo install wasm-pack` should
    work

- install [`cargo-make`][cargo make] with `cargo install cargo-make`

Memthol UI's dependencies rely on some OS-level packages. Please make sure they are installed and in
your path:

- build essentials (including `cmake`)
- openssl/libssl
- freetype
- expat

## `debug`

> **NB**: memthol is quite slow in `debug` mode. If you have a lot of data to process, definitely
> build in release instead.

To build memthol in `debug` mode, run

```bash
cargo make build
```

The memthol binary will be in `target/debug/memthol`

### `release`

To build memthol in `release` mode, run

```bash
cargo make release
```

The memthol binary will be in `./memthol_ui`, also available as `target/release/memthol`.

# Testing

First, make sure you run the bash script that prepares all the test-profiling-dumps:

```bash
./rsc/scripts/prepare_dumps.sh
```

Run memthol-ui on the test files located on this repository, in `rsc/ackermann_with_sets` to make
sure it works. Assuming your binary is called `memthol` and is in you path, and you are at the root
of this repository:

```bash
memthol rsc/dumps/ackermann_with_sets
```

There is a bigger set of data you can use for testing, although we recommend you build in `release`
mode for this one:

```bash
memthol rsc/dumps/dolmen
```


# Other Resources

- [a small tutorial][tuto] (with outdated screenshots)

- [browser compatibility][compat]
- [todo list][todo]
- [memthol-ui compile time and binary size statistics][stats]
- [discussion on string-like filters in memthol-ui][string filters]

# Icons

Most icons used in memthol come from the [bootstrap library][bootstrap].

[OCaml]: https://ocaml.org/ (OCaml official page)
[web assembly]: https://webassembly.org/ (Web Assembly official page)
[Rust]: https://www.rust-lang.org/ (Rust official page)
[rust toolchain]: https://www.rust-lang.org/tools/install (Rust installation instructions)
[wasm-pack]: https://crates.io/crates/cargo-web (Cargo-web on crates.io)
[tuto]: ./rsc/docs/mini_tutorial (Small memthol tutorial)
[compat]: ./rsc/docs/compatibility.md (Browser compatibility discussion)
[todo]: ./todo.md (Todo list)
[stats]: ./rsc/docs/compile_stats.md (Compile time and binary size statistics)
[string filters]: ./rsc/docs/string_like_filters.md (String-like filters)
[install wasm-pack]: https://rustwasm.github.io/wasm-pack/installer (wasm-pack install instructions)
[bootstrap]: https://icons.getbootstrap.com (the bootstrap library)
[cargo make]: https://crates.io/crates/cargo-make (cargo-make on crates.io)