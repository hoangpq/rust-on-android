#!/usr/bin/env sh
$ANDROID_SDK/emulator/emulator \
-avd Rootable_Device -writable-system -selinux permissive -qemu > /dev/null 2>&1 &