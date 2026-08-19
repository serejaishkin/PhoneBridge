# PhoneBridge Development Map

Last updated: 2026-08-20

## Goal

PhoneBridge is a local-first Android companion for **Windows, macOS and Linux**. The PC core must remain platform-neutral; OS-specific integrations belong behind platform interfaces.

## Current branch

`feature/tls-pairing-v1`

## Current PR

Draft PR #1: `feat: implement TLS pairing protocol v1`

## Architecture

```text
Android
  ├── Discovery
  │    ├── DiscoveredPeer
  │    └── PeerRegistry
  ├── TLS client
  │    └── FramedChannel
  ├── Pairing / TrustStore
  ├── ConnectionManager
  └── CallManager / InCallService
          │
          │ local LAN / TLS
          ▼
PC Rust Core
  ├── Discovery
  ├── TLS control plane
  ├── Protocol
  ├── Identity
  ├── TrustStore
  ├── ConnectionManager
  ├── ControlSession
  ├── PairingSession
  ├── Call state / controller
  └── Platform abstraction
        └── HfpBackend
             ├── Windows
             ├── macOS
             └── Linux
```

## Protocol v1

Transport is TLS over TCP port `17591`.

Discovery uses UDP port `17592`.

Wire format is newline-delimited JSON. Variants with payload use `type` + `data`; payload-free messages use only `type`. Rust and Android now use the same serialization shape.

Expected initial flow:

```text
Discovery
  -> TLS
  -> Hello
  <- HelloAck
  <- PairChallenge (when untrusted)
  -> PairConfirm
  <- PairResult
  -> Ping
  <- Pong
```

Call flow:

```text
Android Telephony / InCallService
        │
        ├── IncomingCall
        ├── CallEnded
        │
        ▼
   TLS control channel
        │
        ▼
PC CallController
        │
        ▼
    HfpBackend
```

Pairing must verify both persistent identity fingerprints. TLS certificate pinning is used on Android. The human-readable pairing code is confirmation, not the cryptographic secret.

## Completed in v1 branch

- TLS server foundation on PC.
- Android TLS client foundation.
- Connection state model.
- PC/Android pairing message set.
- Protocol version field.
- Persistent identity fingerprints.
- PC TrustStore.
- Android TrustStore.
- Discovery announcement includes PC identity fingerprint.
- Human-readable pairing code.
- Rust wire framing via newline-delimited JSON.
- Android `ProtocolJson` mirror of Rust serde framing.
- Android `FramedChannel` owns one-frame-per-line I/O.
- Android `TlsClient` now uses `FramedChannel`.
- Android pairing state-machine/UI wiring foundations.
- Platform-neutral PC `connection` module.
- Platform-neutral PC `PairingSession` state machine.
- PC `ControlSession` combining connection and pairing state.
- `PairingServer` now routes received control messages through `ControlSession`.
- PC connection timeout helper.
- Android bounded reconnect policy.
- Android `ConnectionManager` reconnect loop.
- Android `DiscoveredPeer` model with TTL.
- Android `PeerRegistry` for multiple discovered PCs.
- Cross-platform `HfpBackend` abstraction.
- Windows/Linux/macOS HFP backend slots isolated behind cfg modules.
- PC call state remains independent from native audio transport.
- PC `CallController` translates call protocol commands to HFP operations.
- Android `CallManager` observes telephony state.
- Android `BridgeInCallService` exposes answer/reject call controls.

## Important unfinished work

### P0 — finish connection foundation

- [ ] Send `HelloAck` through the shared session layer rather than the server special case.
- [ ] Make already-trusted devices skip pairing cleanly on both sides.
- [ ] Reject stale/mismatched pairing state.
- [ ] Add actual connection timeout enforcement to PC/Android transport.
- [ ] Add explicit `Connected` transition only after authentication/trust is complete.
- [ ] Ensure both sides can initiate Ping/Pong.
- [ ] Handle graceful disconnect.
- [ ] Persist Android trust data safely.
- [ ] Move write serialization into ConnectionManager so multiple features cannot interleave frames.

### P1 — discovery

- [ ] Wire `DiscoveryClient` into `PeerRegistry` lifecycle.
- [ ] Periodic prune of expired peers.
- [ ] Validate discovery fields before inserting peers.
- [ ] Prefer discovered PC fingerprint for TLS pinning.
- [ ] Handle multiple PCs and selected peer persistence.
- [ ] Keep discovery independent from transport connection.

### P1 — calls / HFP

- [ ] Connect Android `CallManager` events to the active PhoneBridge connection.
- [ ] Connect `BridgeInCallService` events to the active PhoneBridge connection.
- [ ] Route incoming `CallAnswer` / `CallDecline` commands to `BridgeInCallService`.
- [ ] Send `PhoneBluetoothStatus` from Android.
- [ ] Send `PcBluetoothStatus` after connection authentication.
- [ ] Wire PC call controller into the live `ControlSession` loop.
- [ ] Windows HFP detection/control implementation.
- [ ] Linux BlueZ D-Bus HFP implementation.
- [ ] macOS IOBluetooth HFP implementation.
- [ ] Audio routing diagnostics.
- [ ] Restore audio state after call.

### P1 — media

- [ ] Play/pause.
- [ ] Next/previous.
- [ ] Volume.
- [ ] Playback metadata.
- [ ] Android media session integration.
- [ ] PC platform media backend.

### P2 — notifications

- [ ] Android notification bridge.
- [ ] PC notification backend.
- [ ] Notification actions.
- [ ] Per-device permissions.

### P2 — clipboard

- [ ] Bidirectional clipboard protocol.
- [ ] Loop prevention.
- [ ] Size limits.
- [ ] Permission/privacy controls.

### P2 — files

- [ ] Capability negotiation.
- [ ] Chunked transfer.
- [ ] Resume interrupted transfer.
- [ ] Hash verification.
- [ ] Progress reporting.

## Rules for future development

1. Do not put Windows APIs in core modules.
2. Do not make cloud services a dependency for LAN functionality.
3. Protocol changes must be reflected in both Rust and Kotlin.
4. Every protocol message gets a stable wire representation.
5. Pairing/trust is separate from discovery.
6. Discovery is not authentication.
7. Never trust a device only because it is on the same LAN.
8. Prefer additive protocol changes over breaking changes.
9. Keep Android and PC implementations in lockstep.
10. Do not merge unfinished architectural experiments into `main`.

## Handoff procedure

1. Read this file.
2. Read `NEXT_STEPS.md`.
3. Inspect latest commits on `feature/tls-pairing-v1`.
4. Continue from the first unchecked P0 item.
5. Do not start GUI/media/HFP native implementation until the P0 connection foundation is coherent.
6. Update this map after every major module.

## Current next coding target

**P0: finish trusted-session semantics and connect the existing Android call layer to the authenticated connection. Then wire PC CallController into ControlSession.**

Development is intentionally proceeding without running builds/tests at this stage. Build failures are to be fixed during the dedicated stabilization pass.
