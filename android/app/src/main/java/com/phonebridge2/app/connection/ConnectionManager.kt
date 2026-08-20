package com.phonebridge2.app.connection

import com.phonebridge2.app.pairing.Message
import com.phonebridge2.app.pairing.TlsClient
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/** Owns one authenticated PhoneBridge control connection and its reconnect loop. */
class ConnectionManager(
    private val tlsClient: TlsClient,
    private val scope: CoroutineScope,
    private val reconnectPolicy: ReconnectPolicy = ReconnectPolicy(),
) {
    private val _state = MutableStateFlow(ConnectionState.DISCONNECTED)
    val state: StateFlow<ConnectionState> = _state.asStateFlow()
    private var job: Job? = null
    private var heartbeatJob: Job? = null
    private var connection: TlsClient.Connection? = null
    private val writeMutex = Mutex()
    private var onMessage: suspend (Message) -> Unit = {}

    fun setMessageHandler(handler: suspend (Message) -> Unit) { onMessage = handler }

    fun connect(host: String, port: Int, fingerprint: String, hello: Message.Hello) {
        job?.cancel(); heartbeatJob?.cancel(); reconnectPolicy.reset()
        job = scope.launch(Dispatchers.IO) {
            var firstAttempt = true
            while (isActive) {
                _state.value = if (firstAttempt) ConnectionState.CONNECTING else ConnectionState.RECONNECTING
                firstAttempt = false
                try {
                    val c = tlsClient.connect(host, port, fingerprint, hello)
                    connection = c
                    _state.value = ConnectionState.HANDSHAKING

                    // HelloAck is the first authenticated application frame. TLS
                    // certificate pinning alone is not enough to enter CONNECTED.
                    val helloAck = tlsClient.readMessage(c)
                    if (helloAck !is Message.HelloAck) {
                        throw IllegalStateException("expected HelloAck as first control message")
                    }
                    onMessage(helloAck)
                    if (helloAck.data.protocol_version != 1) {
                        throw IllegalStateException("unsupported PC protocol version: ${helloAck.data.protocol_version}")
                    }

                    if (helloAck.data.trusted) {
                        _state.value = ConnectionState.CONNECTED
                        reconnectPolicy.reset()
                        startHeartbeat()
                    }

                    while (isActive) {
                        when (val message = tlsClient.readMessage(c)) {
                            is Message.Ping -> send(Message.Pong)
                            is Message.Pong -> Unit
                            is Message.PairResult -> {
                                onMessage(message)
                                if (message.data.trusted) {
                                    _state.value = ConnectionState.CONNECTED
                                    reconnectPolicy.reset()
                                    startHeartbeat()
                                }
                            }
                            is Message.Disconnect -> {
                                _state.value = ConnectionState.DISCONNECTED
                                break
                            }
                            else -> onMessage(message)
                        }
                    }
                } catch (_: Throwable) {
                    if (!isActive) break
                } finally {
                    heartbeatJob?.cancel(); heartbeatJob = null
                    connection?.close(); connection = null
                }
                if (!isActive) break
                delay(reconnectPolicy.nextDelay())
            }
            _state.value = ConnectionState.DISCONNECTED
        }
    }

    private fun startHeartbeat() {
        heartbeatJob?.cancel()
        heartbeatJob = scope.launch(Dispatchers.IO) {
            while (isActive) {
                delay(15_000)
                if (!isActive) break
                if (_state.value == ConnectionState.CONNECTED) send(Message.Ping)
            }
        }
    }

    fun send(message: Message) {
        scope.launch(Dispatchers.IO) {
            writeMutex.withLock {
                val c = connection ?: return@withLock
                val allowed = when (message) {
                    is Message.PairConfirm, is Message.Ping, is Message.Pong -> true
                    else -> _state.value == ConnectionState.CONNECTED
                }
                if (!allowed) return@withLock
                runCatching { tlsClient.sendMessage(c, message) }
            }
        }
    }

    fun disconnect(reason: String = "local shutdown") {
        job?.cancel(); heartbeatJob?.cancel()
        scope.launch(Dispatchers.IO) {
            writeMutex.withLock {
                connection?.let { runCatching { tlsClient.sendMessage(it, Message.Disconnect(Message.DisconnectData(reason))) } }
            }
            connection?.close(); connection = null
        }
        reconnectPolicy.reset(); _state.value = ConnectionState.DISCONNECTED
    }
}
