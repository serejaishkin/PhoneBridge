# Control connection

The connection module owns the lifecycle of one authenticated PhoneBridge control channel.

The current implementation provides:

- explicit connection states;
- one outbound message queue;
- concurrent inbound/outbound processing with `tokio::select!`;
- newline-delimited JSON through the existing `protocol::Message` codec.

The existing pairing server remains the TLS acceptor. Wiring the manager into the server is the next integration step, followed by a real Android pairing flow and interoperability tests.
