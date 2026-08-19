# PhoneBridge Development Map

Last updated: 2026-08-20

## Goal

PhoneBridge is a local-first Android companion for **Windows, macOS and Linux**. The PC core must remain platform-neutral; OS-specific integrations belong behind platform interfaces.

## Current branch

`feature/tls-pairing-v1`

## Architecture

```text
Android
  ├── Discovery / PeerRegistry
  ├── TLS / FramedChannel
  ├── Pairing / TrustStore
  ├── ConnectionManager
  └── CallManager / InCallService / CallBridge
          │
          ▼
PC Rust Core
  ├── Discovery
  ├── TLS control plane
  ├── Protocol
  ├── Identity / TrustStore
  ├── ConnectionManager
  ├── ControlSession / PairingSession
  ├── CallController
  └── HfpBackend
        ├── Windows
        ├── macOS
        └── Linux
```

## Protocol v1

- TLS TCP `17591`.
- UDP discovery `17592`.
- Newline-delimited JSON; Rust and Android use the same serialization shape.
- Pairing verifies persistent identity fingerprints; LAN discovery alone never authenticates a device.

## Completed

- PC TLS server and Android TLS client foundations.
- Canonical newline-delimited framing on Rust/Android.
- Persistent identity and TrustStore foundations.
- Pairing state machine and shared PC `ControlSession`.
- PairingServer routes control messages through ControlSession.
- Android reconnecting ConnectionManager with serialized writes.
- Android discovery peer model/registry.
- Cross-platform HfpBackend abstraction with Windows/Linux/macOS slots.
- PC CallController.
- Android CallManager + CallBridge + InCallService call controls.
- PC ControlSession call dispatch.
- Android CallBridge state stream is lifecycle-safe and deduplicates repeated call-state frames.

## P0 — connection foundation

- [ ] Move `HelloAck` construction into ControlSession.
- [ ] Complete trusted-device fast path on both sides.
- [ ] Reject stale/mismatched pairing state.
- [ ] Enforce transport timeouts.
- [ ] Make Connected transition depend strictly on authentication/trust.
- [ ] Graceful disconnect semantics.
- [ ] Android persistent trust data.
- [ ] Keep all writes serialized through ConnectionManager.

## P1 — calls / HFP

- [ ] Decouple `BridgeInCallService` lifecycle from CallBridge; use an application-level call gateway.
- [ ] Android PhoneBluetoothStatus capability reporting.
- [ ] PC PcBluetoothStatus after authentication.
- [ ] Wire CallController lifetime into live ControlSession.
- [ ] Windows HFP capability detection/control.
- [ ] Linux BlueZ D-Bus HFP capability detection/control.
- [ ] macOS IOBluetooth HFP capability detection/control.
- [ ] Audio routing diagnostics and restoration.

## P1 — discovery

- [ ] DiscoveryClient → PeerRegistry lifecycle.
- [ ] Periodic TTL pruning.
- [ ] Validate announcements.
- [ ] Use discovered fingerprint for TLS pinning.
- [ ] Persist selected PC.

## P1 — media

- [ ] Play/pause, next/previous, volume, metadata.
- [ ] Android MediaSession integration.
- [ ] Windows/macOS/Linux media backends.

## P2

- [ ] Notifications.
- [ ] Clipboard with loop prevention and size limits.
- [ ] Chunked/resumable files with hash verification.

## Rules

1. Core never directly calls OS APIs.
2. Protocol changes are implemented in Rust and Kotlin together.
3. Discovery is not authentication.
4. Prefer additive protocol changes.
5. Keep LAN functionality cloud-independent.
6. Do not merge unfinished experiments into main.

## Handoff

Read this map first, then continue from the first unchecked P0 item. Do not run builds/tests until the planned stabilization pass unless explicitly requested.

## Next coding target

**Finish connection trust semantics, then implement the first real Windows HFP backend.**
