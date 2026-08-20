# PhoneBridge Development Map

Last updated: 2026-08-20

## Goal

PhoneBridge is a local-first Android companion for **Windows, macOS and Linux**. The PC core remains platform-neutral; OS-specific integrations live behind platform interfaces.

## Current branch
`feature/tls-pairing-v1`

## Architecture
```text
Android
  ├── Discovery / PeerRegistry
  ├── TLS / FramedChannel
  ├── Pairing / TrustStore
  ├── ConnectionManager / heartbeat
  └── CallManager / InCallService / CallBridge
          │
          ▼
PC Rust Core
  ├── Discovery
  ├── TLS control plane
  ├── Protocol
  ├── Identity / TrustStore
  ├── ConnectionManager / ControlSession
  ├── PairingSession
  ├── CallController
  └── HfpBackend
        ├── Windows
        ├── macOS
        └── Linux
```

## Protocol v1
- TLS TCP `17591`.
- UDP discovery `17592`.
- Newline-delimited JSON.
- Pairing is bound to persistent device identity fingerprint plus human confirmation code.
- LAN discovery never authenticates a device.
- `Ping` / `Pong` heartbeat frames.
- `Disconnect { reason }` graceful close frame.

## Completed
- PC TLS server and Android TLS client foundations.
- Canonical newline-delimited framing on Rust/Android.
- Persistent identity and TrustStore foundations.
- Fingerprint-bound pairing state machine.
- Trusted-device fast path at PC session layer.
- Pre-authentication command rejection.
- Transport handshake timeout and idle timeout.
- Android reconnecting ConnectionManager with serialized writes.
- Android heartbeat every 15 seconds while connected.
- Android automatic `Pong` response.
- Android graceful `Disconnect` handling and reconnect suppression for that connection.
- PC graceful `Disconnect` handling.
- PC timeout sends `Disconnect` before socket shutdown.
- Android discovery peer model/registry.
- PC CallController and Android CallManager/CallBridge/InCallService foundation.
- Dedicated Windows/Linux/macOS HFP backend boundaries.

## P0 — connection foundation
- [ ] Move `HelloAck` construction completely into ControlSession.
- [ ] Complete trusted-device fast path on Android.
- [x] Reject stale/mismatched pairing state.
- [x] Enforce transport timeouts in session and TCP loop.
- [x] Connected transition depends on successful pairing/trust at PC session layer.
- [x] Graceful disconnect frame and peer-close handling.
- [ ] Android persistent trust data integration.
- [x] Keep Android feature writes serialized through ConnectionManager.
- [ ] PC-side periodic heartbeat sender (Android already sends heartbeat).

## P1 — calls / HFP
- [ ] Decouple `BridgeInCallService` lifecycle from CallBridge; use an application-level call gateway.
- [ ] Android PhoneBluetoothStatus capability reporting.
- [ ] PC PcBluetoothStatus after authentication.
- [ ] Wire CallController lifetime into live ControlSession.
- [ ] Windows native HFP capability detection/control.
- [ ] Linux native BlueZ D-Bus HFP capability detection/control.
- [ ] macOS native IOBluetooth HFP capability detection/control.
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
**Finish Android trusted-device persistence/fast path, wire PC HFP capability into authenticated sessions, then start real Windows HFP implementation.**
