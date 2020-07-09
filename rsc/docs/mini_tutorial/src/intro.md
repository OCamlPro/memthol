# Introduction

*Memthol* is a visualizer and analyzer for program profiling. It works on memory *dumps* containing
information about the size and lifetime of (some of) the allocations performed by some execution of
a program.

This tutorial deals with the BUI (**B**rowser **U**ser **I**nterface) aspect of the profiling. How
the dumps are generated is outside of the scope of this document. Memthol is written in Rust and is composed of

- a server, written in pure Rust, and
- a client, written in Rust and compiled to web assembly (leveraging some JavaScript libraries,
  mostly for graph rendering)

The server contains the client, which it will serve (locally for now) at some address on some port
when launched.


## Running Memthol

Memthol must be given a directory containing the dump files:

```bash
> ls dump/
0000000000000.memthol.diff  0000000000380.memthol.diff  0000000000760.memthol.diff  0000000001140.memthol.diff  0000000001520.memthol.diff
0000000000020.memthol.diff  0000000000400.memthol.diff  0000000000780.memthol.diff  0000000001160.memthol.diff  0000000001540.memthol.diff
0000000000040.memthol.diff  0000000000420.memthol.diff  0000000000800.memthol.diff  0000000001180.memthol.diff  0000000001560.memthol.diff
0000000000060.memthol.diff  0000000000440.memthol.diff  0000000000820.memthol.diff  0000000001200.memthol.diff  0000000001580.memthol.diff
0000000000080.memthol.diff  0000000000460.memthol.diff  0000000000840.memthol.diff  0000000001220.memthol.diff  0000000001600.memthol.diff
0000000000100.memthol.diff  0000000000480.memthol.diff  0000000000860.memthol.diff  0000000001240.memthol.diff  0000000001620.memthol.diff
0000000000120.memthol.diff  0000000000500.memthol.diff  0000000000880.memthol.diff  0000000001260.memthol.diff  0000000001640.memthol.diff
0000000000140.memthol.diff  0000000000520.memthol.diff  0000000000900.memthol.diff  0000000001280.memthol.diff  0000000001660.memthol.diff
0000000000160.memthol.diff  0000000000540.memthol.diff  0000000000920.memthol.diff  0000000001300.memthol.diff  0000000001680.memthol.diff
0000000000180.memthol.diff  0000000000560.memthol.diff  0000000000940.memthol.diff  0000000001320.memthol.diff  0000000001700.memthol.diff
0000000000200.memthol.diff  0000000000580.memthol.diff  0000000000960.memthol.diff  0000000001340.memthol.diff  0000000001720.memthol.diff
0000000000220.memthol.diff  0000000000600.memthol.diff  0000000000980.memthol.diff  0000000001360.memthol.diff  0000000001740.memthol.diff
0000000000240.memthol.diff  0000000000620.memthol.diff  0000000001000.memthol.diff  0000000001380.memthol.diff  0000000001760.memthol.diff
0000000000260.memthol.diff  0000000000640.memthol.diff  0000000001020.memthol.diff  0000000001400.memthol.diff  0000000001780.memthol.diff
0000000000280.memthol.diff  0000000000660.memthol.diff  0000000001040.memthol.diff  0000000001420.memthol.diff  init.memthol
0000000000300.memthol.diff  0000000000680.memthol.diff  0000000001060.memthol.diff  0000000001440.memthol.diff
0000000000320.memthol.diff  0000000000700.memthol.diff  0000000001080.memthol.diff  0000000001460.memthol.diff
0000000000340.memthol.diff  0000000000720.memthol.diff  0000000001100.memthol.diff  0000000001480.memthol.diff
0000000000360.memthol.diff  0000000000740.memthol.diff  0000000001120.memthol.diff  0000000001500.memthol.diff
> memthol dump
|===| Config
| url: http://localhost:7878
| dump directory: `dump`
|===|

initializing assets...
starting data monitoring...
starting socket listeners...

```
