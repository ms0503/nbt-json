#!/usr/bin/env bash
set -euo pipefail
cd "$(realpath "$(dirname "$0")")"
TARGETS=(
    aarch64-unknown-linux-gnu
    aarch64-unknown-linux-musl
    i686-pc-windows-gnu
    i686-unknown-linux-gnu
    i686-unknown-linux-musl
    x86_64-pc-windows-gnu
    x86_64-unknown-linux-gnu
    x86_64-unknown-linux-musl
)
VERSION=$(tomlq --file Cargo.toml .package.version 2>/dev/null)
PKG_NAME_BASE=nbt-json_$VERSION
mkdir -p out
for t in "${TARGETS[@]}"; do
    printf "Building for target %s\n" "$t"
    cross build --release --target "$t"
    printf "Creating package for target %s\n" "$t"
    rm -rf "out/$t"
    mkdir -p "out/$t/nbt-json/bin"
    cp LICENSE.md README.md "out/$t/nbt-json"
    if (printf "$t" | grep -q windows); then
        PKG_NAME=${PKG_NAME_BASE}_${t}.zip
        cp "target/$t/release/nbt-json.exe" "out/$t/nbt-json/bin"
        (cd "out/$t"; zip -r "../$PKG_NAME" nbt-json)
    else
        PKG_NAME=${PKG_NAME_BASE}_${t}.tar.gz
        cp "target/$t/release/nbt-json" "out/$t/nbt-json/bin"
        (cd "out/$t"; tar cf "../$PKG_NAME" nbt-json)
    fi
done
