#! /bin/bash

set -e

target_path=`echo "$(cd "$(dirname "target")"; pwd -P)/$(basename "target")"`
final_bin="./memthol_ui"

echo "compiling client to wasm..."
echo
echo

wasm-pack build \
    --release \
    --target web \
    --out-name client \
    --out-dir "$target_path/client.wasm" \
    libs/client

echo
echo
echo "compiling memthol-ui in release..."
echo
echo

cargo build --release

cp target/release/memthol "$final_bin"

echo
echo
echo "done, final memthol-ui binary generated at '$final_bin'"
