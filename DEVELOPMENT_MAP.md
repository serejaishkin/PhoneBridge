# PhoneBridge Development Map

Last updated: 2026-08-31

## Active branch
`feature/kdeconnect-core`

## Current state
Android compile errors reported by the developer were traced to missing/incorrect references. `MainActivity` already contains the microphone relay state and `AudioPlaybackService` exists; `CallManager` contains its `TelephonyManager`/`Intent` dependencies. The signaling client has been restored as a complete source file and now includes the JSON protocol types used by the active control path, including pairing challenge/result handling and `confirmPairing()`.

**Important:** the developer's local `./gradlew assembleDebug` run reported Kotlin compilation errors before this repair. No new build result is claimed until the developer runs the command again.

## Immediate verification
```text
cd /d/GitHub/PhoneBridge/android
./gradlew assembleDebug
```

If compilation advances, fix the next compiler error rather than adding speculative modules.

## Target architecture
```text
PhoneBridge
  │
  ├── KDE Connect compatible identity/pairing/protocol model
  │
  ├── Android
  │    ├── discovery
  │    ├── TLS/control session
  │    ├── pairing/trust
  │    ├── calls
  │    ├── media
  │    └── SMS
  │
  ├── PC
  │    ├── discovery
  │    ├── TLS/control session
  │    ├── pairing/trust
  │    ├── desktop GUI
  │    └── platform backends
  │         ├── Windows
  │         ├── Linux
  │         └── macOS
  │
  └── PhoneBridge extensions
       ├── Bluetooth
       ├── PC hotspot
       └── audio/HFP
```

## Pairing
```text
Android Hello
      ↓
PC TLS/control session
      ↓
PairChallenge
      ↓
Desktop Allow / Reject
      ↓
PairApprove / PairReject
      ↓
Android
      ↓
TrustStore
      ↓
Paired
```

The protocol must eventually converge on the KDE Connect-compatible path. The old custom TLS/pairing stack is transition/reference code only.

## Completed foundation
- GPL-3.0-only project direction.
- `feature/kdeconnect-core` migration branch.
- KDE Connect-compatible identity/pair packet model foundation.
- PC TLS listener/control substrate.
- PC certificate fingerprint support.
- Android persistent identity/certificate support.
- Android PC certificate pinning/TOFU model.
- PC trusted-peer storage model.
- Pairing state machine with explicit Allow/Reject.
- Desktop pairing UI foundation.
- Android pairing UI foundation.
- Android `SignalingClient` mutual-TLS client identity support.
- Android pairing challenge/result handling.
- Android `confirmPairing()` command.
- Android call manager and media/audio service foundation.
- Windows HFP detection foundation.
- Linux HFP detection foundation.
- macOS HFP detection foundation.
- Windows RFCOMM backend foundation in the transition stack.

## Current P0
- [ ] Re-run Android `assembleDebug` after signaling/client repair.
- [ ] Fix remaining Kotlin compiler errors one by one.
- [ ] Build-verify Android X509KeyManager/certificate identity path.
- [ ] Build-verify Android pairing/session implementation.
- [ ] Connect Android packet/session implementation to KDE Connect-compatible packet framing.
- [ ] Establish real Android ↔ PC pairing on hardware.
- [ ] Establish discovery.
- [ ] Establish reconnect.
- [ ] Establish common transport selection for LAN/hotspot/Bluetooth.
- [ ] Windows Bluetooth runtime test.
- [ ] Linux Bluetooth runtime test.
- [ ] macOS Bluetooth runtime test.

## Audio / calls
- [x] PC audio receiver source foundation.
- [x] Android media capture source foundation.
- [x] PC microphone → Android playback source foundation.
- [ ] End-to-end audio test.
- [ ] HFP call control end-to-end.
- [ ] Opus packet-loss/recovery.

## Desktop GUI
- [x] Cross-platform dashboard state.
- [x] Pairing screen.
- [x] Allow / Reject controls.
- [x] Forget-peer control foundation.
- [ ] Bind GUI commands to the actual live production session writer on `feature/kdeconnect-core`.
- [ ] Live pairing status updates from production session.
- [ ] Device list/discovery UI.
- [ ] Reconnect/fallback UI.

## Android GUI
- [x] Main PhoneBridge screen.
- [x] PC address connection control.
- [x] Media capture control.
- [x] PC microphone relay control.
- [x] Pairing protocol hooks.
- [ ] Discovered PC list.
- [ ] Pairing code dialog.
- [ ] Trust/Forget PC UI.
- [ ] Connection/reconnect state UI.

## Bluetooth
Bluetooth is a transport/backend, not a replacement protocol.

```text
Android Bluetooth
       │
       ├── Windows
       ├── Linux
       └── macOS
              ↓
       common byte stream
              ↓
       KDE Connect compatible control/session
```

## Hotspot
- [ ] Windows Mobile Hotspot.
- [ ] Linux NetworkManager hotspot.
- [ ] macOS Internet Sharing where supported.
- [ ] Android hotspot discovery/connection.
- [ ] Route preference/fallback.

## Development rules
1. Active work is on `feature/kdeconnect-core`.
2. Do not continue growing the frozen custom protocol unless migration compatibility requires it.
3. Rust and Kotlin protocol changes must stay aligned.
4. Keep OS APIs in platform-specific modules.
5. Code comments are in English.
6. Refresh the current file SHA immediately before every GitHub update.
7. Never claim a build/test passed unless it was actually executed.
8. Do not mark runtime transport complete from static inspection.
9. Prefer fixing the next real compiler error over speculative architecture changes.

## Handoff
The developer has now supplied a real Android Gradle compiler log. The signaling client was restored first. **Next action is a fresh `./gradlew assembleDebug` result from the developer; use the exact next compiler error to continue.**
