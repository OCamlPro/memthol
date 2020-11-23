(**************************************************************************)
(*                                                                        *)
(*  Copyright (c) 2020 Albin Coquereau <albin.coquereau@ocamlpro.com>     *)
(*                                                                        *)
(*  All rights reserved.                                                  *)
(*  This file is distributed under the terms of the GNU Lesser General    *)
(*  Public License version 2.1, with the special exception on linking     *)
(*  described in the LICENSE.md file in the root directory.               *)
(*                                                                        *)
(**************************************************************************)


(* If you delete or rename this file, you should add
   'src/min_memtrace_lib/main.ml' to the 'skip' field in "drom.toml" *)

let main () =
  Memtrace.trace_if_requested ();
   Printf.printf "Hello world!\n"
