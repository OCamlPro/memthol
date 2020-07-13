#! /bin/bash

target_path=`echo "$(cd "$(dirname "target")"; pwd -P)/$(basename "target")"`

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
echo "done"