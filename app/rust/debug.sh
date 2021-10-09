#!/usr/bin/env bash
adb push debug_script.sh /data/local/tmp

cd "$ANDROID_NDK/prebuilt/android-x86/gdbserver"
adb push gdbserver /data/local/tmp
adb shell "chmod 777 /data/local/tmp/gdbserver"
adb shell "chmod 777 /data/local/tmp/debug_script.sh"

adb root
adb forward tcp:1337 tcp:1337
# adb shell "su && set enforce 0"

echo "Starting server!!"
adb shell "/data/local/tmp/debug_script.sh"
