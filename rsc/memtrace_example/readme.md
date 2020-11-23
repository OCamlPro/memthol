# Memtrace Example

This is a minimal example of calling the [memtrace library][memtrace git]. First, let's compile it.

```bash
> make
opam exec -- dune build @install
Done: 54/58 (jobs: 1)cp -f _build/default/src/min_memtrace/main.exe min_memtrace
> ls min_memtrace
min_memtrace
```


Now, the whole program is contained in file `src/min_memtrace_lib/main.ml`.

```ocaml
let main () =
    Memtrace.trace_if_requested ();
    Printf.printf "Hello world!\n"
```

This instructs memtrace to

- do nothing if the environment variable `MEMTRACE` is not set;
- dump profiling information to `file` if `MEMTRACE` is set to `file`.

Hence, just rust the following command to produce a (very small) profiling dump:

```bash
> MEMTRACE=dump.ctf ./min_memtrace
Hello world!
> ls -lh dump.ctf
-rw-------  1 ______  _____   311B Nov 23 17:45 dump.ctf
```

For more details, refer to the [memtrace repository][memtrace git].

[memtrace git]: https://github.com/janestreet/memtrace (Memtrace on github.com)