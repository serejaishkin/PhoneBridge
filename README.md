# PhoneBridge V1.3

Turn your PC into a wireless headset for your phone. Answer calls, stream audio, and control everything from your computer.

## Features

| Feature | Android | iPhone | Description |
|---------|---------|--------|-------------|
| Media audio | ✅ Bluetooth A2DP | ✅ Bluetooth A2DP | Stream phone audio to PC headphones |
| Call audio | ✅ Wi-Fi + Opus/UDP | ❌ Not possible* | Answer calls from your PC |
| PC microphone | ✅ Wi-Fi + Opus/UDP | ❌ | Talk back through PC mic |
| Call notifications | ✅ | ⚠️ Limited | See who's calling on your PC screen |
| Auto-discovery | ✅ BLE | ✅ Bluetooth | Phone finds PC automatically |
| iOS AirPlay fallback | ✅ (Linux/macOS) | ✅ (Windows via ShairportQt) | Stream iOS media via AirPlay |

\* iOS restrictions prevent call interception by third-party apps.

## Architecture

```
PhoneBridge Protocol V1.3
├── Discovery: BLE advertising (Android IP + port in manufacturer data)
├── Media: Bluetooth A2DP Sink (native OS)
├── Call Audio: UDP + Opus (48kHz, 20ms frames, bidirectional)
├── Signaling: WebSocket (JSON)
└── Control: AT-like commands over WebSocket
```

## Quick Start

### PC Setup (Windows/Linux/macOS)

```bash
cd pc
cargo build --release
./target/release/phonebridge
```

### Android Setup

1. Install **PhoneBridge** app from releases
2. Pair phone to PC via Bluetooth (for media)
3. Connect to `PhoneBridge` Wi-Fi hotspot
4. Grant permissions: microphone, phone calls, Bluetooth, screen capture
5. Tap **Start Bridge** — BLE advertising starts automatically

### iOS AirPlay (optional)

- **Linux/macOS**: install `shairport-sync`
- **Windows**: install [ShairportQt](https://github.com/Frank-Friemel/ShairportQt)
- iPhone → Control Center → AirPlay → "PhoneBridge"

## Network Ports

| Port | Direction | Purpose |
|------|-----------|---------|
| 5000 | PC ↔ Android | WebSocket signaling |
| 5001 | Android → PC | Opus audio (system audio) |
| 5002 | iPhone → PC | AirPlay audio (optional) |
| 5003 | PC → Android | Opus audio (PC microphone) |

## Building from Source

### Android
```bash
cd android
./gradlew assembleDebug
```

### PC (Rust)
```bash
cd pc
cargo build --release
```

## License

MIT License — free for personal and commercial use.
