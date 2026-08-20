# PhoneBridge Development Map

Last updated: 2026-08-20

## Goal
PhoneBridge is a local-first Android companion for **Windows, macOS and Linux**. The PC core remains platform-neutral; OS-specific integrations live behind platform interfaces.

## Current branch
`feature/tls-pairing-v1`

## Architecture
```text
Android
  ├── DiscoveryClient / PeerRegistry
  ├── TLS / FramedChannel
  ├── Pairing / TrustStore
  ├── ConnectionManager / authenticated handshake / heartbeat / reconnect
  ├── EndpointStore / PeerConnectionStore / PreferredRouteStore / RoutePlanner
  └── CallManager / InCallService / CallBridge
          │
          ▼
PC Rust Core
  ├── Discovery / UDP LAN + hotspot
  ├── PeerRegistry / TTL
  ├── RouteMemory / RouteStore / ConnectionCoordinator
  ├── TLS control plane
  ├── Protocol
  ├── Identity / TrustStore
  ├── ConnectionManager / ControlSession
  ├── PairingSession
  ├── CallController
  ├── UiBackend
  └── Platform
        ├── Bluetooth stream contract
        ├── OS Bluetooth backend selector
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
- Android does not enter `CONNECTED` until a trusted `HelloAck` or successful `PairResult` is received.

## Completed
- PC TLS server and Android TLS client foundations.
- Canonical newline-delimited framing on Rust/Android.
- Persistent identity and TrustStore foundations.
- Fingerprint-bound pairing state machine.
- Trusted-device fast path at PC and Android session layers.
- Android first-frame `HelloAck` validation.
- Android pre-authentication command gating.
- Pre-authentication command rejection on PC.
- Transport handshake timeout and idle timeout.
- Android reconnecting ConnectionManager with serialized writes.
- Android heartbeat every 15 seconds while connected.
- Android automatic `Pong` response.
- Android graceful `Disconnect` handling.
- PC graceful `Disconnect` handling.
- PC timeout sends `Disconnect` before socket shutdown.
- Android discovery peer model/registry.
- Android persistent PC endpoint store.
- Android selected-PC persistence.
- Android deterministic Wi-Fi/hotspot/Bluetooth-PAN route planner.
- Android preferred transport persistence and route prioritization.
- Android selected-PC → live ConnectionManager integration.
- Automatic multi-route reconnect coordinator foundation.
- PC Bluetooth native stream transport contract for RFCOMM/L2CAP.
- PC per-OS Bluetooth backend selector for Windows/Linux/macOS.
- PC preferred-route memory and route ordering.
- PC route persistence primitive.
- PC multi-route ConnectionCoordinator foundation for TCP routes.
- PC ConnectionCoordinator can load persisted route preference and persist it only after explicit authenticated-session confirmation.
- PC discovery peer registry with TTL.
- PC CallController and Android CallManager/CallBridge/InCallService foundation.
- Dedicated Windows/Linux/macOS HFP backend boundaries.

## P0 — connection foundation
- [ ] Move `HelloAck` construction completely into ControlSession.
- [x] Complete trusted-device fast path on Android.
- [x] Reject stale/mismatched pairing state.
- [x] Enforce transport timeouts in session and TCP loop.
- [x] Connected transition depends on successful pairing/trust at PC session layer.
- [x] Android Connected transition depends on authenticated HelloAck/PairResult.
- [x] Graceful disconnect frame and peer-close handling.
- [x] Android persistent trust data integration in PairingManager.
- [x] Keep Android feature writes serialized through ConnectionManager.
- [ ] PC-side periodic heartbeat sender.
- [x] Android can persist the last PC endpoint independently of discovery.
- [x] Android can persist the selected PC identity.
- [x] Persist preferred transport route metadata and prioritize it during reconnect.
- [x] PC route model can remember the last successful transport.
- [x] PC has a route-attempt coordinator for TCP transports.
- [x] Connect saved-PC selection directly to live Android ConnectionManager.
- [x] PC RouteStore is available to the live ConnectionCoordinator.
- [x] Route persistence is exposed only through `mark_authenticated()` on the PC coordinator.
- [ ] Wire `mark_authenticated()` into the actual authenticated ControlSession owner.
- [ ] Complete actual direct Bluetooth RFCOMM/L2CAP adapters per OS.

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
- [x] UDP discovery on normal Wi-Fi/LAN.
- [x] UDP discovery usable on a PC-created hotspot when broadcast is permitted by the OS/firewall.
- [x] Android PeerRegistry model and TTL pruning primitive.
- [x] PC PeerRegistry model and TTL pruning primitive.
- [x] DiscoveryClient persists announced PC endpoints.
- [ ] Wire DiscoveryClient to a dedicated Android PeerRegistry service.
- [ ] Validate announcements against expected protocol/schema constraints.
- [x] Discovered fingerprint carried into TLS pinning.
- [x] Persist selected PC endpoint.
- [ ] Advertise direct Bluetooth endpoint when native Bluetooth backend is available.

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
7. Bluetooth PAN is a network route; direct Bluetooth RFCOMM/L2CAP is a separate transport.
8. Code comments are written in English; development map is maintained as the handoff source of truth.
9. Do not mark a native OS backend complete until it actually opens/discovers the required Bluetooth transport.
10. Refresh the current file SHA immediately before every update; never reuse an older blob SHA.

## Handoff
Read this map first, then continue from the first unchecked P0 item. Do not run builds/tests until the planned stabilization pass unless explicitly requested.

## Next coding target
**Wire `ConnectionCoordinator::mark_authenticated()` into the real PC ControlSession owner, then implement the actual Windows Bluetooth transport, followed by Linux and macOS.**
