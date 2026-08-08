# PhoneBridge Setup Guide

## PC (Rust)

### Requirements
- Rust 1.70+
- Windows: `tray-icon` works out of the box
- BLE: built-in Bluetooth 4.0+ adapter

### Build
```bash
cd pc
cargo build --release
```

### iOS AirPlay on Windows
Install [ShairportQt](https://github.com/Frank-Friemel/ShairportQt) to `C:\Program Files\ShairportQt\`.
PhoneBridge auto-detects it.

## Android

### Requirements
- Android Studio Hedgehog+
- NDK 25+
- libopus prebuilt `.a` for all ABIs

### Build libopus for Android
```bash
cd android/app/src/main/cpp
chmod +x download_opus.sh
./download_opus.sh
```

### Build APK
```bash
cd android
./gradlew assembleDebug
```

### Permissions
Grant: Bluetooth, Location (for BLE scan on older Android), Microphone, Notifications.

## Network
- Default ports: UDP 5001 (PC←Android), UDP 5003 (PC→Android), WS 5000, AirPlay 5002
- BLE advertising: PhoneBridge device broadcasts its Wi-Fi IP + port 5003
