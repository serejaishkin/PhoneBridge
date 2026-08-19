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

class ConnectionManager(
    private val tlsClient: TlsClient,
    private val scope: CoroutineScope,
) {
    private val _state = MutableStateFlow(ConnectionState.DISCONNECTED)
    val state: StateFlow<ConnectionState> = _state.asStateFlow()
    private var job: Job? = null
    private var connection: TlsClient.Connection? = null

    fun connect(host: String, port: Int, fingerprint: String, hello: Message.Hello) {
        job?.cancel()
        job = scope.launch(Dispatchers.IO) {
            _state.value = ConnectionState.CONNECTING
            try {
                val c = tlsClient.connect(host, port, fingerprint, hello)
                connection = c
                _state.value = ConnectionState.CONNECTED

                while (isActive) {
                    val message = tlsClient.readMessage(c)
                    when (message) {
                        Message.Ping -> tlsClient.sendMessage(c, Message.Pong)
                        else -> Unit
                    }
                }
            } catch (_: Throwable) {
                _state.value = ConnectionState.RECONNECTING
                delay(1000)
                _state.value = ConnectionState.DISCONNECTED
            } finally {
                connection?.close()
                connection = null
            }
        }
    }

    fun send(message: Message) {
        val c = connection ?: return
        scope.launch(Dispatchers.IO) {
            runCatching { tlsClient.sendMessage(c, message) }
        }
    }

    fun disconnect() {
        job?.cancel()
        connection?.close()
        connection = null
        _state.value = ConnectionState.DISCONNECTED
    }
}
