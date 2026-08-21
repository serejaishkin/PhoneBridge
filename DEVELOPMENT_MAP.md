# PhoneBridge Development Map

Last updated: 2026-08-21

## Project direction
PhoneBridge is a free/open-source project intended for personal use and public sharing.

The project has pivoted toward a **KDE Connect based architecture** instead of continuing to grow a second custom device/pairing/protocol stack.

## License
- [x] Project license changed from MIT to GPL-3.0-only.
- [x] GPL-3.0 license notice added to `LICENSE`.
- [x] README/Cargo metadata updated for GPL-3.0-only.
- [ ] Add final third-party attribution/notices after KDE Connect/Sefirah source components are selected.
- [ ] Do not copy third-party GPL code into files that are not properly attributed until the migration boundary is documented.

## Current repository strategy
```text
PhoneBridge repository
│
├── main
│   └── public project branch
│
├── feature/tls-pairing-v1
│   └── frozen experimental custom transport/session stack
│
└── feature/kdeconnect-core
    └── active migration/integration branch
```

The old custom TLS/pairing implementation is retained for history and reference. Do not continue adding parallel protocol features there unless required to preserve compatibility during migration.

## Target architecture
```text
                    PhoneBridge
                         │
              KDE Connect protocol/core
                         │
        ┌────────────────┼────────────────┐
        │                │                │
      Android           PC              Platform
        │                │                │
        │          device/features       │
        │                │         ┌──────┼──────┐
        │                │         │      │      │
        │                │       Windows Linux  macOS
        │                │         │      │      │
        └────────────────┴─────────┴──────┴──────┘
                         │
              PhoneBridge extensions
                         │
          ┌──────────────┼──────────────┐
          │              │              │
      Bluetooth       PC Hotspot       Audio/HFP
```

## Reuse policy
### KDE Connect
Use as the primary reference/base for:
- device model;
- discovery/link abstraction;
- pairing/trust model;
- packet/protocol model;
- plugin/feature architecture;
- existing Linux/Windows/macOS desktop foundation;
- Android protocol implementation where appropriate.

The current KDE Connect protocol reference documents `kdeconnect.identity` and `kdeconnect.pair`; protocol version 8 is current in that reference. Pairing is explicit and devices must be paired before normal packets are accepted. The PhoneBridge compatibility layer follows this model.

### Sefirah
Use as a feature/UX reference and, where legally appropriate, as source for selected GPL components:
- calls;
- SMS;
- notifications;
- clipboard;
- media control;
- file/storage UX;
- screen mirroring;
- Windows desktop UX.

### PhoneBridge original code
Reuse only where it provides functionality not already better supplied by the selected base:
- Bluetooth transport abstraction/backends;
- PC hotspot management;
- cross-platform route selection/reconnect;
- phone audio streaming;
- HFP integration;
- PhoneBridge-specific desktop/mobile UI.

## Migration rules
1. Do not maintain two independent pairing/protocol stacks long-term.
2. Prefer KDE Connect's established device/protocol model over recreating equivalent infrastructure.
3. Keep PhoneBridge-specific functionality behind clean extension/backend boundaries.
4. Keep OS APIs inside platform-specific modules.
5. Preserve English comments in code.
6. Record third-party source origin and license before copying substantial source files.
7. Do not claim a native backend complete until it actually works with the common protocol/session path.
8. Refresh file SHA immediately before every GitHub update.
9. Do not mark tests as passing unless they were actually executed.
10. Keep the frozen custom stack available until the new implementation reaches feature parity for required workflows.

## Migration phases
### Phase 0 — licensing and audit
- [x] Decide on GPL-3.0-only.
- [x] Change project metadata/license notice.
- [x] Audit PhoneBridge custom protocol/session architecture.
- [x] Compare PhoneBridge with KDE Connect and Sefirah at architecture level.

### Phase 1 — KDE Connect integration map
- [x] Create migration branch `feature/kdeconnect-core`.
- [x] Add a small KDE Connect-compatible packet layer for `identity` and `pair`.
- [x] Add a UI-independent pairing state machine with explicit Allow/Reject decisions.
- [x] Add an interactive desktop pairing playground to make the new pairing flow tangible before network integration.
- [x] Keep the pairing playground isolated from the production daemon.
- [x] Fix duplicate Rust `protocol` module layout (`protocol.rs` vs `protocol/mod.rs`).
- [x] Restore PC dependencies required by existing audio/network modules (`cpal`, `opus`, `crossbeam-channel`, `parking_lot`, `tokio-tungstenite`).
- [x] Expose the existing `pairing` module from the PC library.
- [x] Remove the obsolete `SharedState` dependency from the WebSocket shell while the connection-state architecture is migrated.
- [x] Fix audio input type mismatches between CPAL PCM callbacks and the Opus channel.
- [ ] Pin exact KDE Connect Android/desktop versions or commits to use as reference/base.
- [ ] Map PhoneBridge files to KDE Connect equivalents.
- [ ] Identify code that can be reused directly.
- [ ] Identify code that must be rewritten/adapted.
- [ ] Identify code that should be removed after migration.
- [ ] Create third-party attribution inventory.

### Phase 2 — protocol/device foundation
- [x] Establish initial KDE Connect-compatible packet model.
- [x] Establish initial KDE Connect-compatible pairing state model.
- [ ] Replace the old PhoneBridge Hello/HelloAck pairing protocol with the KDE Connect-compatible path.
- [ ] Establish Android ↔ desktop pairing using the selected base.
- [ ] Establish discovery using the selected base.
- [ ] Establish reconnect using the selected base.
- [ ] Stop extending the old custom TLS/pairing protocol.

### Phase 3 — desktop platforms
- [ ] Windows desktop integration.
- [ ] Linux desktop integration.
- [ ] macOS desktop integration.
- [ ] Unified PhoneBridge desktop UI.

### Phase 4 — Bluetooth
- [ ] Android Bluetooth Classic transport.
- [ ] Windows Bluetooth transport.
- [ ] Linux BlueZ transport.
- [ ] macOS Bluetooth transport.
- [ ] Bluetooth reconnect/fallback policy.

Bluetooth is treated as a transport/backend problem, not as a replacement protocol.

### Phase 5 — PC hotspot
- [ ] Windows Mobile Hotspot backend.
- [ ] Linux NetworkManager hotspot backend.
- [ ] macOS Internet Sharing backend where supported.
- [ ] Android hotspot discovery/connection workflow.
- [ ] Route preference and fallback between LAN, hotspot and Bluetooth.

A PC hotspot remains an IP network path. It must not introduce a second application protocol.

### Phase 6 — PhoneBridge-specific features
- [ ] Phone call control.
- [ ] HFP state/control.
- [ ] Android → PC audio streaming.
- [ ] PC → Android microphone/audio path.
- [ ] Opus transport and recovery.
- [ ] Media integration.
- [ ] Notifications.
- [ ] Clipboard.
- [ ] Files.

## Current pairing playground
Run from `pc/` after dependencies are installed:

```text
cargo run --bin phonebridge-pairing-demo
```

The demo currently provides:
- simulated Android identity packet;
- visible pairing state;
- remote device details;
- Allow button;
- Reject button;
- displayed `kdeconnect.pair` response packet;
- no network/TLS side effects.

This is a **UX/protocol smoke playground**, not yet a real phone connection test.

## Build/test status
The latest local Windows `cargo check` exposed five migration-branch errors. They have now been addressed in commits:
- obsolete `crate::protocol::SharedState` dependency in the WebSocket shell;
- CPAL `f32 -> i16` conversion bound;
- incorrect Opus receiver type stored in `AudioInput`;
- ambiguous generic sample conversion;
- incorrect `recv_opus()` receiver type.

**These fixes have not yet been locally recompiled after the latest commits**, so the branch remains `build verification pending`.

Next local verification:
```text
cd pc
cargo check
cargo test
cargo run --bin phonebridge-pairing-demo
```

Do not mark the build/test gate as passed until these commands actually complete successfully.

## Frozen custom stack
The previous implementation contains:
- Identity;
- TrustStore;
- custom TLS server/client;
- custom pairing state;
- custom discovery;
- custom ConnectionManager;
- custom route persistence;
- Windows RFCOMM transport bridge;
- desktop pairing GUI.

These components are **reference/transition code**, not the long-term architecture.

## Testing gate
Before deleting or replacing old components:
1. Android build must pass.
2. Desktop build must pass on the affected platform.
3. Protocol/pairing tests must pass.
4. Discovery must work.
5. Reconnect must work.
6. Bluetooth transport must be tested on real hardware.
7. PC hotspot must be tested on the target OS.
8. No feature is marked complete from static inspection alone.

## Handoff
Read this file first before continuing development.

**Immediate next task:** run the fixed PC build/test locally, then wire the KDE Connect-compatible pairing model into the real PC TLS connection and add the matching Android packet/session implementation. Keep the desktop demo available until real pairing works.
