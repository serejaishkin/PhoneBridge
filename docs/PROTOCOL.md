# PhoneBridge Protocol V1.3

## Packet Format (UDP Audio)

### Android → PC (port 5001)
```
+--------+--------------------------------+
|  seq   |          opus payload          |
| 2 bytes|          variable              |
+--------+--------------------------------+
```

### PC → Android (port 5003)
Same format: 2-byte sequence + Opus frame.

## Signaling (WebSocket JSON)

### Android → PC
```json
{"type":"register","device":"android"}
{"type":"incoming_call","data":{"number":"+1234567890"}}
{"type":"call_answered"}
{"type":"call_ended"}
```

### PC → Android
```json
{"type":"answer_call"}
{"type":"end_call"}
{"type":"mute_toggle"}
```

## BLE Discovery

### Advertisement Data
- Service UUID: `a1b2c3d4-e5f6-7890-abcd-ef1234567890`
- Manufacturer ID: `0xFFFF`
- Manufacturer Data (6 bytes):
  - `[0..3]` = IPv4 address (big-endian per octet)
  - `[4..5]` = UDP port (big-endian)

### Scanning
PC scans for BLE devices with name prefix `PhoneBridge`, extracts IP/port from manufacturer data, auto-connects UDP + WebSocket.
