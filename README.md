# PhoneBridge2

Open-source PhoneBridge project for personal/local use: Android ↔ PC integration without a cloud service.

## License

PhoneBridge is released under the **GNU General Public License v3.0 (GPL-3.0-only)**.

The project is intentionally kept fully open source so that users can study, modify, build and share their own versions. GPLv3 permits personal use, modification and redistribution, including charging for copies, while requiring the corresponding GPL-covered source and license terms when covered works are distributed. urlGNU GPLv3https://www.gnu.org/licenses/gpl-3.0.html

Third-party components keep their own licenses and copyright notices. When code from another GPL-licensed project such as KDE Connect is incorporated, its original attribution and applicable license notices will be preserved.

## Current architecture direction

The repository is being moved toward a **KDE Connect-compatible foundation** instead of maintaining a second independent device/pairing protocol unnecessarily.

Planned/reused layers:

- device identity and pairing model;
- discovery and device connection management;
- common device/plugin protocol;
- Android device integration;
- Windows, Linux and macOS desktop integration.

PhoneBridge-specific work remains focused on functionality not provided by the base stack:

- direct Bluetooth transport where required;
- PC-created hotspot support;
- Bluetooth HFP integration;
- phone/PC audio transport;
- cross-platform route selection and reconnect behaviour.

## Current repository status

`feature/tls-pairing-v1` contains the earlier independent TLS/pairing implementation. It is being kept as a historical/experimental branch while the project moves toward the new foundation.

The current `main` branch is **not an end-to-end validated release**. Build and runtime testing must be performed before claiming a working MVP.

## Original PhoneBridge goals

- PC as a wireless companion for Android calls and media/audio use cases.
- No mandatory cloud service.
- Native Bluetooth Classic HFP for call audio where supported by the operating system.
- Network transport for control and other data.
- Cross-platform PC support: Windows, Linux and macOS.

## Development

Read `DEVELOPMENT_MAP.md` before continuing implementation. The map is the handoff source of truth and records which parts are implemented, planned or intentionally frozen.

The project follows these rules:

1. Keep platform-specific APIs behind platform backends.
2. Do not duplicate pairing/device/protocol logic when an existing compatible foundation can provide it.
3. Bluetooth transport and IP networking remain separate transport backends.
4. PC hotspot is a platform/network service, not a separate application protocol.
5. Code comments are written in English.
6. Do not mark code as tested unless a real build/test was run.
