#!/usr/bin/env bash
source ~/.bash_profile
dir=`pwd`

NDK_STANDALONE=$HOME/ndk-standalone

if ! [[ -d ${NDK_STANDALONE} ]]
then
    rustup default nightly
    rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android
    mkdir -p ${NDK_STANDALONE}
    ${NDK_HOME}/build/tools/make_standalone_toolchain.py --api 26 --arch arm64\
        --install-dir ${NDK_STANDALONE}/arm64
    ${NDK_HOME}/build/tools/make_standalone_toolchain.py --api 26 --arch arm\
        --install-dir ${NDK_STANDALONE}/arm
    ${NDK_HOME}/build/tools/make_standalone_toolchain.py --api 26 --arch x86\
        --install-dir ${NDK_STANDALONE}/x86
fi

node ./gen-config.js
cd `pwd`/app/rust

rm -f ./target/x86/librust.a
rm -f ./target/arm64-v8a/librust.a
rm -f ./target/armeabi-v7a/librust.a

cargo +nightly build --target aarch64-linux-android --release
cargo +nightly build --target armv7-linux-androideabi --release
cargo +nightly build --target i686-linux-android --release

mkdir -p ./target/x86
mkdir -p ./target/arm64-v8a
mkdir -p ./target/armeabi-v7a

cp ./target/aarch64-linux-android/release/librust.a ./target/arm64-v8a/librust.a
cp ./target/armv7-linux-androideabi/release/librust.a ./target/armeabi-v7a/librust.a
cp ./target/i686-linux-android/release/librust.a ./target/x86/librust.a

rustc +nightly --target wasm32-unknown-unknown -O src/main.rs
wasm-gc main.wasm wa.wasm && rm main.wasm

cp ./wa.wasm "${dir}/app/src/main/assets/deps" && rm ./wa.wasm
