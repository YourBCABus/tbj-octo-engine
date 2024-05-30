#!/bin/zsh

cargo build --profile release-minimum --target x86_64-unknown-linux-gnu || true

mkdir -p ./target/tiny-oengine
cp ./target/x86_64-unknown-linux-gnu/release-minimum/octo-engine ./target/tiny-oengine/octo-engine

cp ./target/tiny-oengine/octo-engine ./target/tiny-oengine/octo-engine-uncompressed

upx -9 ./target/tiny-oengine/octo-engine
du -Ah ./target/tiny-oengine/*
