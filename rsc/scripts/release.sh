#! /bin/bash

final_bin="./memthol_ui"

set -e

./rsc/scripts/compile_wasm.sh

echo
echo
echo "compiling memthol-ui in release..."
echo
echo

cargo build --release "$@"

cp target/release/memthol "$final_bin"

echo
echo
echo "done, final memthol-ui binary generated at '$final_bin'"
