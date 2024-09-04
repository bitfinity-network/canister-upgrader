#!/usr/bin/env sh
set -e
set -x #echo on

export RUST_BACKTRACE=full

WASM_DIR="target/wasm32-unknown-unknown/release"

build_upgrader_canister() {
    echo "Building upgrader_canister"

    cargo build -p upgrader_canister --target wasm32-unknown-unknown --release
    ic-wasm $WASM_DIR/upgrader_canister.wasm -o $WASM_DIR/upgrader_canister.wasm shrink
    candid-extractor $WASM_DIR/upgrader_canister.wasm > $WASM_DIR/upgrader_canister.did
    gzip -k "$WASM_DIR/upgrader_canister.wasm" --force
}

main() {
    mkdir -p $WASM_DIR

    build_upgrader_canister

}

main "$@"