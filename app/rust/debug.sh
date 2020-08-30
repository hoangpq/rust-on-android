#!/usr/bin/env bash

cd $NDK20/prebuilt/android-x86/gdbserver
adb push gdbserver /data/local/tmp
adb shell "chmod 777 /data/local/tmp/gdbserver"

adb root
adb forward tcp:1337 tcp:1337

adb shell "su"
adb shell "set enforce 0"
adb shell "/data/local/tmp/gdbserver :1337 --attach $(ps -A | grep com.node.sample | awk '{print $2}')"
