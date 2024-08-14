#!/bin/zsh

if [ -z "${CARGO_TARGET_DIR}" ]; then 
    TARGET='target'
else 
    TARGET=${CARGO_TARGET_DIR}
fi

cargo build --profile release-minimum --target x86_64-unknown-linux-gnu || true

mkdir -p $TARGET/tiny-oengine
cp $TARGET/x86_64-unknown-linux-gnu/release-minimum/octo-engine $TARGET/tiny-oengine/octo-engine

cp $TARGET/tiny-oengine/octo-engine $TARGET/tiny-oengine/octo-engine-uncompressed

upx -9 $TARGET/tiny-oengine/octo-engine
du -Ah $TARGET/tiny-oengine/*
