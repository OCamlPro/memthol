# Compile Stats in Release Mode

- times given in seconds
- compiling time is the time for a cold build, *including compiling the wasm*; roughly,

    ```bash
    > cargo clean
    > ./rsc/scripts/release
    ```
- size is given for the macos binary, in `MB`

| revision   | time | size |
| ---        | ---  | ---  |
| [7c21abe2] | 443  | 6.7  |

[7c21abe2]: https://gitlab.ocamlpro.com/OCamlPro/memthol_ui/-/commit/7c21abe25fec6bd7d1d6fe13c8ffc60c398f22f5