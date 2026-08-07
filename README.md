# PhoneBridge

Turn your PC into a wireless headset for your phone. Answer calls, stream audio, and control everything from your computer.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform: Android](https://img.shields.io/badge/Android-10%2B-green.svg)](#)
[![Platform: Windows](https://img.shields.io/badge/Windows-10%2B-blue.svg)](#)
[![Platform: macOS](https://img.shields.io/badge/macOS-13%2B-lightgrey.svg)](#)
[![Platform: Linux](https://img.shields.io/badge/Linux-PipeWire%2FPulseAudio-orange.svg)](#)

---

## What is PhoneBridge?

You're at your desk, wearing headphones connected to your PC. Your phone rings across the room. Instead of fumbling for it, you see a notification on your monitor, click "Answer," and talk through your PC's microphone and headphones.

**PhoneBridge** makes this possible — wirelessly, with minimal latency.

---

## Features

| Feature | Android | iPhone | Description |
|---------|---------|--------|-------------|
| **Media audio** (YouTube, games, music) | ✅ Bluetooth A2DP | ✅ Bluetooth A2DP | Stream phone audio to PC headphones |
| **Call audio** | ✅ Wi-Fi + app | ❌ Not possible* | Answer calls from your PC |
| **Call notifications** | ✅ | ⚠️ Limited | See who's calling on your PC screen |
| **PC microphone** | ✅ | ❌ | Talk back through PC mic |
| **Auto-discovery** | ✅ BLE / Wi-Fi | ✅ Bluetooth | Phone finds PC automatically |

\* iOS restrictions prevent call interception by third-party apps.

---

## How It Works

```
┌─────────────┐      Bluetooth A2DP       ┌─────────────┐
│   Phone     │  ───────────────────────►  │     PC      │
│  (YouTube,  │      Media audio           │  (Speakers) │
│   games)    │                            │             │
└─────────────┘                            │             │
                                           │             │
┌─────────────┐      Wi-Fi (hotspot)      │             │
│   Phone     │  ───────────────────────►  │             │
│  (Call)     │      Opus + UDP            │             │
│             │                            │             │
│  Answer ◄───┼──────────────────────────  │             │
│  End call ◄─┼──────────────────────────  │             │
└─────────────┘                            └─────────────┘
```

**Hybrid connection:**
- **Bluetooth** — for media (low latency, works natively)
- **Wi-Fi hotspot** — for calls and control (app-based, full features)

---

## Quick Start

### 1. PC Setup (Windows)

```powershell
# Enable Wi-Fi hotspot
netsh wlan set hostednetwork mode=allow ssid=PhoneBridge key=YourPassword123
netsh wlan start hostednetwork

# Or use Settings → Mobile Hotspot
```

Install [VB-Audio Virtual Cable](https://vb-audio.com/Cable/) for virtual audio device.

```bash
# Clone and run
git clone https://github.com/yourname/phonebridge.git
cd phonebridge/pc
pip install -r requirements.txt
python server.py
```

### 2. Phone Setup (Android)

1. Install **PhoneBridge** app from releases
2. Pair phone to PC via Bluetooth (for media)
3. Connect to `PhoneBridge` Wi-Fi hotspot
4. Grant permissions: microphone, phone calls, screen capture

---

## Platforms

| Platform | Media (BT A2DP) | Calls (Wi-Fi) | Notes |
|----------|-----------------|---------------|-------|
| **Windows 10/11** | ✅ | ✅ | Primary target |
| **macOS 13+** | ✅ | ✅ | No competitors here |
| **Linux (PipeWire)** | ✅ | ✅ | Native BT HFP support |
| **iPhone** | ✅ | ❌ | Media only, Apple restrictions |

---

## Architecture

```
PhoneBridge Protocol
├── Discovery: BLE advertising / mDNS / gateway detection
├── Media: Bluetooth A2DP Sink (native OS)
├── Call Audio: UDP + Opus (48kHz, 20ms frames)
├── Signaling: WebSocket (JSON)
└── Control: AT-like commands over WebSocket
```

See [PROTOCOL.md](docs/PROTOCOL.md) for details.

---

## Roadmap

- [x] Android → Windows MVP
- [ ] macOS client (Swift)
- [ ] Linux HFP native mode (BlueZ)
- [ ] BLE auto-discovery
- [ ] Jitter buffer & AEC
- [ ] Settings UI (bitrate, latency)
- [ ] B2B: call center batch deployment

---

## Building from Source

### Android
```bash
cd android
./gradlew assembleDebug
```

### PC (Python)
```bash
cd pc
python -m venv venv
source venv/bin/activate  # or venv\Scripts\activate on Windows
pip install -r requirements.txt
python server.py
```

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md).

**Priority areas:**
- iOS research (CallKit limitations)
- Windows Bluetooth HFP driver investigation
- Low-latency audio optimization

---

## License

MIT License — free for personal and commercial use.

---

## Acknowledgments

- Inspired by KDE Connect architecture
- Uses Opus codec from Xiph.Org
- Bluetooth stack: BlueZ (Linux), Windows Bluetooth API, CoreAudio (macOS)

---

## FAQ

**Q: Does it work with iPhone?**
A: Media (YouTube, music) works via Bluetooth. Calls do not — iOS doesn't allow third-party call interception.

**Q: Why not just use Bluetooth HFP?**
A: Windows and macOS cannot act as Bluetooth headsets (HFP Hands-Free role). Only Linux can. We use Wi-Fi as a workaround.

**Q: Is my data private?**
A: Yes. All audio stays on your local network. No cloud servers, no accounts.

**Q: Can I use it without Wi-Fi hotspot?**
A: Yes, if both devices are on the same LAN. Hotspot mode is for zero-config setup.

---

**Made with ❤️ for people who live at their desks.**
