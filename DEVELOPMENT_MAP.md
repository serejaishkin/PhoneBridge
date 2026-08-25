# PhoneBridge Development Map

Last updated: 2026-08-25

## Current branch
`feature/tls-pairing-v1`

## Current state
GUI-to-session pairing command plumbing is now represented explicitly by `BasicUi::UiCommand` and `ControlSession::handle_ui_command()`. The next required step is runtime ownership/writer integration and then the stabilization build pass. No build/test result is claimed.

## Architecture
```text
Android
  ├── Discovery / PeerRegistry
  ├── TLS / FramedChannel
  ├── Pairing / TrustStore / PairingManager
  ├── ConnectionManager / reconnect / heartbeat
  ├── EndpointStore / PreferredRouteStore / RoutePlanner
  └── Pairing UI / CallBridge
          │
          ▼
PC Rust Core
  ├── Discovery / UDP LAN + hotspot
  ├── RouteMemory / RouteStore / ConnectionCoordinator
  ├── Common AsyncRead/AsyncWrite transport boundary
  ├── TLS acceptor
  ├── Protocol / ControlSession
  ├── PairingSession / TrustStore
  ├── Desktop UiState / UiCommand
  ├── Desktop GUI
  └── Platform
        ├── Windows WinRT RFCOMM
        ├── Linux BlueZ [planned]
        └── macOS IOBluetooth [planned]
```

## Pairing flow
```text
Android Hello
   ↓
PC PairingSession
   ↓
PairChallenge
   ↓
Desktop GUI
   ↓
Allow / Reject
   ↓
UiCommand channel
   ↓
ControlSession::handle_ui_command()
   ↓
PairApprove / PairReject
   ↓
TLS writer
   ↓
Android
   ↓
TrustStore
```

## Completed
- Persistent device identity and fingerprint-bound trust.
- Pairing short-code verification.
- Android pairing wizard and Forget action.
- PC pairing challenge/result UI state.
- Desktop Allow / Reject / Forget controls.
- `BasicUi::UiCommand` channel for pairing decisions.
- `ControlSession::handle_ui_command()` boundary for applying desktop commands.
- Android authenticated reconnect and preferred route persistence.
- PC route persistence only after authenticated session.
- Common PC async byte-stream boundary.
- Transport-independent TLS acceptor.
- Windows WinRT RFCOMM discovery/connect foundation.
- Windows RFCOMM → common byte-stream/TLS source-code bridge foundation.

## P0 — remaining
- [ ] Connect `BasicUi` command receiver to the actual live ControlSession owner and serialized TLS writer.
- [ ] Make PC Forget revoke the persistent TrustStore, not only the live pairing session.
- [ ] Make desktop pairing result update the live UI state after the writer completes.
- [ ] Run `cargo check` on PC.
- [ ] Run `cargo test` on PC.
- [ ] Run Android unit tests.
- [ ] Run Android debug compilation.
- [ ] Fix all compile/API errors found by the stabilization pass.
- [ ] Confirm Windows RFCOMM → TLS → ControlSession at runtime.
- [ ] Android direct RFCOMM client.
- [ ] Linux BlueZ RFCOMM/L2CAP transport.
- [ ] macOS IOBluetooth transport.

## P1
- [ ] HFP capability/control on Windows/Linux/macOS.
- [ ] Media controls and metadata.
- [ ] Notifications.
- [ ] Clipboard.
- [ ] Chunked/resumable file transfer.

## Rules
1. Core never directly calls OS APIs.
2. Protocol changes are implemented in Rust and Kotlin together.
3. Discovery is not authentication.
4. Bluetooth PAN is a network route; direct Bluetooth RFCOMM/L2CAP is a separate transport.
5. Code comments are in English.
6. Do not mark a transport complete before runtime integration.
7. Refresh the current file SHA immediately before every update.
8. Do not claim builds/tests passed unless actually executed.

## Handoff
Next work must start with the live `UiCommand` receiver/writer integration. Then run the requested stabilization build/test pass before adding another OS transport.
