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
  ├── Pairing / TrustStore
  ├── ConnectionManager
  └── Features
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
  ├── PairingSession
  ├── Call state
  └── Platform abstraction
        └── HfpBackend
             ├── Windows
             ├── macOS
             └── Linux
```

## Protocol v1

Transport is TLS over TCP port `17591`.

Discovery uses UDP port `17592`.

Expected initial flow:

```text
Discovery
  -> Hello
  <- HelloAck
  <- PairChallenge (when untrusted)
  -> PairConfirm
  <- PairResult
  -> Ping
  <- Pong
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
- Rust/Kotlin wire-format test foundations.
- Android pairing state-machine/UI wiring foundations.
- Platform-neutral PC `connection` module.
- Platform-neutral PC `PairingSession` state machine.
- PC connection timeout helper.
- Android bounded reconnect policy.
- Android `ConnectionManager` reconnect loop.
- Android `DiscoveredPeer` model with TTL.
- Android `PeerRegistry` for multiple discovered PCs.
- Cross-platform `HfpBackend` abstraction.
- Windows/Linux/macOS HFP backend slots isolated behind cfg modules.
- PC call state remains independent from native audio transport.

## Important unfinished work

### P0 — finish connection foundation

- [ ] Integrate PairingServer with ConnectionManager instead of keeping a separate message loop.
- [ ] Define one canonical framing implementation for Rust/Kotlin.
- [ ] Make already-trusted devices skip pairing cleanly.
- [ ] Reject stale/mismatched pairing state.
- [ ] Add actual connection timeout enforcement to PC/Android transport.
- [ ] Add explicit `Connected` transition only after authentication/trust is complete.
- [ ] Ensure both sides can initiate Ping/Pong.
- [ ] Handle graceful disconnect.
- [ ] Persist Android trust data safely.

### P1 — discovery

- [ ] Wire `DiscoveryClient` into `PeerRegistry` lifecycle.
- [ ] Periodic prune of expired peers.
- [ ] Validate discovery fields before inserting peers.
- [ ] Prefer discovered PC fingerprint for TLS pinning.
- [ ] Handle multiple PCs and selected peer persistence.
- [ ] Keep discovery independent from transport connection.

### P1 — calls / HFP

- [ ] Define call control messages in the shared protocol.
- [ ] Android call-state producer.
- [ ] Answer/decline/end commands.
- [ ] Wire PC call commands to `HfpBackend`.
- [ ] Windows HFP detection/control implementation.
- [ ] Linux BlueZ D-Bus HFP implementation.
- [ ] macOS IOBluetooth HFP implementation.
- [ ] Audio routing diagnostics.
- [ ] Restore audio state after call.

### P1 — platform abstraction

Core must not contain OS-specific APIs.

```text
PlatformBackend
  ├── WindowsBackend
  ├── MacOSBackend
  └── LinuxBackend
```

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

**P0 connection integration and canonical framing. After that, call-control protocol is the next feature.**

Development is intentionally proceeding without running builds/tests at this stage. Build failures are to be fixed during the dedicated stabilization pass.
