#!/usr/bin/env bash

source ~/.bash_profile

if ! [ -d ~/ndk-standalone ]
then
    rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android
    mkdir -p ~/ndk-standalone
    ${NDK_HOME}/build/tools/make_standalone_toolchain.py --api 26 --arch arm64\
        --install-dir ~/ndk-standalone/arm64
    ${NDK_HOME}/build/tools/make_standalone_toolchain.py --api 26 --arch arm\
        --install-dir ~/ndk-standalone/arm
    ${NDK_HOME}/build/tools/make_standalone_toolchain.py --api 26 --arch x86\
        --install-dir ~/ndk-standalone/x86
    cp ./cargo-config.toml ~/.cargo/config
fi

cd `pwd`/app/rust

rm -f ./target/x86/librust.a
rm -f ./target/arm64-v8a/librust.a
rm -f ./target/armeabi-v7a/librust.a

cargo build --target aarch64-linux-android --release
cargo build --target armv7-linux-androideabi --release
cargo build --target i686-linux-android --release

mkdir -p ./target/x86
mkdir -p ./target/arm64-v8a
mkdir -p ./target/armeabi-v7a

cp ./target/aarch64-linux-android/release/librust.a ./target/arm64-v8a/librust.a
cp ./target/armv7-linux-androideabi/release/librust.a ./target/armeabi-v7a/librust.a
cp ./target/i686-linux-android/release/librust.a ./target/x86/librust.a
