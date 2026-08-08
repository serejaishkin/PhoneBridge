#!/bin/bash
# Download and build libopus for Android
set -e

OPUS_VERSION="1.4"
ANDROID_NDK_HOME=${ANDROID_NDK_HOME:-$ANDROID_NDK}

if [ -z "$ANDROID_NDK_HOME" ]; then
    echo "ERROR: ANDROID_NDK_HOME not set"
    exit 1
fi

ABIS="armeabi-v7a arm64-v8a x86 x86_64"

mkdir -p opus_prebuilt
cd opus_prebuilt

wget -q "https://downloads.xiph.org/releases/opus/opus-${OPUS_VERSION}.tar.gz"
tar xzf "opus-${OPUS_VERSION}.tar.gz"
cd "opus-${OPUS_VERSION}"

for ABI in $ABIS; do
    mkdir -p "build_${ABI}"
    cd "build_${ABI}"

    case $ABI in
        armeabi-v7a)
            ARCH="arm"
            TOOLCHAIN="arm-linux-androideabi"
            ;;
        arm64-v8a)
            ARCH="arm64"
            TOOLCHAIN="aarch64-linux-android"
            ;;
        x86)
            ARCH="x86"
            TOOLCHAIN="i686-linux-android"
            ;;
        x86_64)
            ARCH="x86_64"
            TOOLCHAIN="x86_64-linux-android"
            ;;
    esac

    cmake ..         -DCMAKE_TOOLCHAIN_FILE="${ANDROID_NDK_HOME}/build/cmake/android.toolchain.cmake"         -DANDROID_ABI="${ABI}"         -DANDROID_PLATFORM=android-26         -DCMAKE_BUILD_TYPE=Release         -DBUILD_SHARED_LIBS=OFF

    make -j$(nproc)

    mkdir -p "../../${ABI}/include"
    cp libopus.a "../../${ABI}/"
    cp -r ../include/* "../../${ABI}/include/"

    cd ..
done

echo "libopus built for: $ABIS"
