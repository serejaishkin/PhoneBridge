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
Use as a feature/UX reference and, where legally appropriate, as source for selected GPL components.

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
- [x] Add KDE Connect-compatible `identity` and `pair` packet model.
- [x] Add UI-independent pairing state machine with explicit Allow/Reject decisions.
- [x] Add interactive desktop pairing playground.
- [x] Fix duplicate Rust `protocol` module layout.
- [x] Restore PC audio/network dependencies.
- [x] Expose the existing pairing module.
- [x] Remove obsolete `SharedState` dependency from the WebSocket shell.
- [x] Fix audio input type mismatches.
- [x] Export `PairingSession` from the KDE Connect module.
- [x] Add persistent PC TLS certificate/key identity.
- [x] Add SHA-256 certificate fingerprint display support.
- [x] Add persistent trusted-peer storage model.
- [x] Add a TLS pairing listener on port `1716`.
- [ ] Wire TLS listener decisions directly to the production pairing UI.
- [ ] Pin exact KDE Connect Android/desktop versions or commits.
- [ ] Map PhoneBridge files to KDE Connect equivalents.
- [ ] Identify reusable/adaptable/removable source components.
- [ ] Create final third-party attribution inventory.

### Phase 2 — protocol/device foundation
- [x] Establish initial KDE Connect-compatible packet model.
- [x] Establish initial KDE Connect-compatible pairing state model.
- [x] Establish PC TLS transport substrate.
- [ ] Replace old PhoneBridge Hello/HelloAck pairing protocol with KDE Connect-compatible path.
- [ ] Add Android packet/session implementation.
- [ ] Establish real Android ↔ desktop pairing.
- [ ] Establish discovery.
- [ ] Establish reconnect.
- [ ] Enable mutual peer certificate authentication after Android certificate support exists.

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
Run from `pc/`:

```text
cargo run --bin phonebridge-pairing-demo
```

The playground provides:
- simulated Android identity packet;
- visible pairing state;
- remote device details;
- Allow/Reject state-machine controls;
- KDE Connect pair response display.

The new TLS layer provides:
- persistent PC certificate/key;
- stable SHA-256 certificate fingerprint;
- TLS listener on TCP `1716`;
- KDE Connect identity packet sent after TLS connection;
- framed packet reception;
- persistent trust-store data model.

**Current limitation:** the TLS listener uses server-authenticated TLS only. Android peer certificate authentication and GUI-controlled network Allow/Reject response are the next security/pairing integration step. Do not call this production-secure pairing yet.

## Build/test status
### Latest user-provided Windows verification
User ran:
```text
cargo check
```
from `pc/` on `feature/kdeconnect-core` and confirmed that it **passes** after the previous pairing fixes.

The newly added TLS files have **not yet been locally verified after this latest block**.

Next verification:
```text
cd /d/GitHub/PhoneBridge/pc
cargo check
cargo test
cargo run --bin phonebridge-pairing-demo
```

Do not mark the new TLS block as build-verified until the user runs these commands successfully.

## Frozen custom stack
The previous implementation contains Identity, TrustStore, custom TLS server/client, custom pairing state, custom discovery, ConnectionManager, route persistence, Windows RFCOMM transport bridge, and desktop pairing GUI.

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

**Immediate next task:** locally build/test the TLS block, then connect Android identity/pair packets to the TLS session and route network pairing decisions through the desktop UI. After that implement persistent peer certificates and mutual TLS trust.
