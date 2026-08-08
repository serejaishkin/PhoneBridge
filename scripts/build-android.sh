#!/bin/bash
set -e
cd "$(dirname "$0")/../android"
./gradlew assembleDebug
echo "APK: app/build/outputs/apk/debug/app-debug.apk"
